use std::borrow::Cow;
use std::mem;
use std::ptr::NonNull;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use ohos_drawing_binding::{
    AlphaFormat, Bitmap, BitmapFormat, Canvas, ColorFormat, FontCollection, FontSlant, FontStyle,
    FontWeight, FontWidth, ImageInfo, TextStyle, Typography, TypographyBuilder, TypographyStyle,
};
use rustc_hash::FxHashMap;
use wgpu::rwh::{OhosDisplayHandle, OhosNdkWindowHandle, RawDisplayHandle, RawWindowHandle};

use crate::frame::{east_asian_width, CursorVisualStyle};
use crate::native_surface::NativeSurface;
use crate::worker::RenderPacket;

const ATLAS_SIZE: u32 = 1024;
const INITIAL_CELL_BUFFER: u64 = 64 * 1024;
// Packed bits must match `src/shaders/cell.wgsl`.
const FLAG_UNDERLINE: u32 = 1 << 0;
const FLAG_STRIKE: u32 = 1 << 1;
const FLAG_SPACER: u32 = 1 << 2;
const FLAG_WIDE: u32 = 1 << 3;
const FLAG_HAS_GLYPH: u32 = 1 << 4;
const FLAG_COLOR_GLYPH: u32 = 1 << 12;
const CURSOR_SHIFT: u32 = 8;
const COPY_ROW_ALIGN: u32 = 256;

const CELL_SHADER: &str = include_str!("shaders/cell.wgsl");

/// Owns the wgpu device/queue/surface, the GPU cell instance buffer, and the
/// glyph atlas. The UI thread only publishes the newest viewport snapshot.
pub(crate) struct TerminalRenderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<BoundSurface>,
    resources: GpuResources,
    glyphs: GlyphRegistry,
    rasterizer: GlyphRasterizer,
}

impl TerminalRenderer {
    pub(crate) fn new(surface: NativeSurface) -> Result<Self, String> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::GL;
        #[cfg(target_env = "ohos")]
        descriptor.flags.remove(wgpu::InstanceFlags::DEBUG);
        let instance = wgpu::Instance::new(descriptor);
        let raw_surface = create_surface(&instance, &surface)?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&raw_surface),
            ..Default::default()
        }))
        .map_err(|error| format!("request_adapter: {error}"))?;
        let required_limits = adapter.limits();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("arkit terminal device"),
            required_limits,
            ..Default::default()
        }))
        .map_err(|error| format!("request_device: {error}"))?;
        let bound = configure_surface(raw_surface, surface, &adapter, &device)?;
        let resources = GpuResources::new(&device, &queue, bound.config.format);

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: Some(bound),
            resources,
            glyphs: GlyphRegistry::default(),
            rasterizer: GlyphRasterizer::new(),
        })
    }

    pub(crate) fn bind_surface(&mut self, native: NativeSurface) -> Result<(), String> {
        let raw_surface = create_surface(&self.instance, &native)?;
        let bound = configure_surface(raw_surface, native, &self.adapter, &self.device)?;
        if bound.config.format != self.resources.format {
            self.resources = GpuResources::new(&self.device, &self.queue, bound.config.format);
            self.glyphs.clear();
        }
        self.surface = Some(bound);
        Ok(())
    }

    pub(crate) fn unbind_surface(&mut self) {
        self.surface = None;
    }

    pub(crate) fn render(&mut self, packet: &RenderPacket) -> Result<(), String> {
        let Some(surface) = self.surface.as_ref() else {
            return Ok(());
        };
        if self.glyphs.atlas_full {
            self.resources.reset_atlas(&self.device, &self.queue);
            self.glyphs.clear();
        }

        let width = surface.config.width;
        let height = surface.config.height;
        let srgb_target = self.resources.format.is_srgb();
        let cells = pack_cells(
            packet,
            srgb_target,
            &mut self.glyphs,
            &mut self.rasterizer,
            &self.queue,
            &mut self.resources.atlas,
        )?;
        self.resources
            .cells
            .upload(&self.device, &self.queue, &cells);
        self.resources.write_grid(
            &self.queue,
            packet.frame.cols,
            packet.frame.rows,
            width,
            height,
            packet.metrics.cell_width_px,
            packet.metrics.cell_height_px,
            packet.metrics.padding_vp as f32 * packet.metrics.scale() as f32,
            srgb_target,
        );

        let (frame, reconfigure) = match surface.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                surface.surface.configure(&self.device, &surface.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err("wgpu XComponent surface was lost".to_string());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("wgpu rejected the XComponent surface frame".to_string());
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("arkit terminal frame"),
            });
        {
            let clear = if packet.frame.default_bg != 0 {
                packet.frame.default_bg
            } else {
                packet.background_color
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("arkit terminal gpu cells"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(argb_to_wgpu(clear, srgb_target)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.resources.cells.draw(
                &mut pass,
                &self.resources.pipeline,
                &self.resources.bind_group,
                cells.len() as u32,
            );
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        if reconfigure {
            surface.surface.configure(&self.device, &surface.config);
        }
        Ok(())
    }
}

fn create_surface(
    instance: &wgpu::Instance,
    native: &NativeSurface,
) -> Result<wgpu::Surface<'static>, String> {
    let window = OhosNdkWindowHandle::new(
        NonNull::new(native.raw_window()).ok_or_else(|| "null OHNativeWindow".to_string())?,
    );
    let display = OhosDisplayHandle::new();
    // SAFETY: `NativeSurface` retains an OHNativeWindow reference and is stored
    // next to the returned wgpu surface. `BoundSurface` drops the wgpu surface
    // before releasing that native reference.
    unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(RawDisplayHandle::Ohos(display)),
            raw_window_handle: RawWindowHandle::OhosNdk(window),
        })
    }
    .map_err(|error| format!("create_surface: {error}"))
}

fn configure_surface(
    surface: wgpu::Surface<'static>,
    native: NativeSurface,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
) -> Result<BoundSurface, String> {
    let width =
        u32::try_from(native.width()).map_err(|_| "negative XComponent width".to_string())?;
    let height =
        u32::try_from(native.height()).map_err(|_| "negative XComponent height".to_string())?;
    let mut config = surface
        .get_default_config(adapter, width.max(1), height.max(1))
        .ok_or_else(|| "wgpu adapter does not support the XComponent surface".to_string())?;
    let capabilities = surface.get_capabilities(adapter);
    if capabilities
        .present_modes
        .contains(&wgpu::PresentMode::Fifo)
    {
        config.present_mode = wgpu::PresentMode::Fifo;
    }
    config.desired_maximum_frame_latency = 2;
    surface.configure(device, &config);
    Ok(BoundSurface {
        surface,
        _native: native,
        config,
    })
}

struct BoundSurface {
    surface: wgpu::Surface<'static>,
    _native: NativeSurface,
    config: wgpu::SurfaceConfiguration,
}

struct GpuResources {
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    grid_uniform: wgpu::Buffer,
    atlas: AtlasTexture,
    cells: CellBuffer,
}

impl GpuResources {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("arkit terminal cell shader"),
            source: wgpu::ShaderSource::Wgsl(CELL_SHADER.into()),
        });
        let grid_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("arkit terminal grid uniform"),
            size: mem::size_of::<GridUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let atlas = AtlasTexture::new(device, queue);
        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("arkit terminal cell binds"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("arkit terminal cell bind group"),
            layout: &bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: grid_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&atlas.sampler),
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("arkit terminal cell layout"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("arkit terminal cell pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<CellInstance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x4,
                        1 => Float32x4,
                        2 => Float32x4,
                        3 => Uint32
                    ],
                })],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            format,
            pipeline,
            bind_group,
            grid_uniform,
            atlas,
            cells: CellBuffer::new(device),
        }
    }

    fn reset_atlas(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.atlas = AtlasTexture::new(device, queue);
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("arkit terminal cell bind group"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.grid_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.atlas.sampler),
                },
            ],
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn write_grid(
        &self,
        queue: &wgpu::Queue,
        cols: u16,
        rows: u16,
        surface_w: u32,
        surface_h: u32,
        cell_w: f32,
        cell_h: f32,
        pad: f32,
        srgb_target: bool,
    ) {
        let uniform = GridUniform {
            cols: u32::from(cols.max(1)),
            rows: u32::from(rows.max(1)),
            surface_w: surface_w.max(1) as f32,
            surface_h: surface_h.max(1) as f32,
            cell_w: cell_w.max(f32::EPSILON),
            cell_h: cell_h.max(f32::EPSILON),
            pad,
            srgb_target: if srgb_target { 1.0 } else { 0.0 },
        };
        queue.write_buffer(&self.grid_uniform, 0, bytemuck::bytes_of(&uniform));
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GridUniform {
    cols: u32,
    rows: u32,
    surface_w: f32,
    surface_h: f32,
    cell_w: f32,
    cell_h: f32,
    pad: f32,
    srgb_target: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CellInstance {
    bg: [f32; 4],
    fg: [f32; 4],
    uv: [f32; 4],
    packed: u32,
    _pad: [u32; 3],
}

struct CellBuffer {
    buffer: wgpu::Buffer,
    capacity: u64,
}

impl CellBuffer {
    fn new(device: &wgpu::Device) -> Self {
        Self {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("arkit terminal cell instances"),
                size: INITIAL_CELL_BUFFER,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            capacity: INITIAL_CELL_BUFFER,
        }
    }

    fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, cells: &[CellInstance]) {
        if cells.is_empty() {
            return;
        }
        let bytes = bytemuck::cast_slice(cells);
        let required = bytes.len() as u64;
        if required > self.capacity {
            self.buffer.destroy();
            self.capacity = required.next_power_of_two().max(INITIAL_CELL_BUFFER);
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("arkit terminal cell instances"),
                size: self.capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.buffer, 0, bytes);
    }

    fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        pipeline: &'a wgpu::RenderPipeline,
        bind_group: &'a wgpu::BindGroup,
        count: u32,
    ) {
        if count == 0 {
            return;
        }
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.set_vertex_buffer(0, self.buffer.slice(..));
        pass.draw(0..6, 0..count);
    }
}

struct AtlasTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    x: u32,
    y: u32,
    row_h: u32,
}

impl AtlasTexture {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("arkit terminal glyph atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            &vec![0u8; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ATLAS_SIZE * 4),
                rows_per_image: Some(ATLAS_SIZE),
            },
            wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("arkit terminal atlas sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            x: 1,
            y: 1,
            row_h: 0,
        }
    }

    fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        let width = width.max(1);
        let height = height.max(1);
        if width + 1 >= ATLAS_SIZE || height + 1 >= ATLAS_SIZE {
            return None;
        }
        if self.x + width + 1 > ATLAS_SIZE {
            self.x = 1;
            self.y += self.row_h + 1;
            self.row_h = 0;
        }
        if self.y + height + 1 > ATLAS_SIZE {
            return None;
        }
        let origin = (self.x, self.y);
        self.x += width + 1;
        self.row_h = self.row_h.max(height);
        Some(origin)
    }

    fn upload(&self, queue: &wgpu::Queue, x: u32, y: u32, width: u32, height: u32, rgba: &[u8]) {
        let (padded, bytes_per_row) = pad_rgba_rows(rgba, width, height);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &padded,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

fn pack_cells(
    packet: &RenderPacket,
    srgb_target: bool,
    glyphs: &mut GlyphRegistry,
    rasterizer: &mut GlyphRasterizer,
    queue: &wgpu::Queue,
    atlas: &mut AtlasTexture,
) -> Result<Vec<CellInstance>, String> {
    let frame = &packet.frame;
    let cursor_visible = !(packet.cursor_blink && frame.cursor.blinking && !packet.cursor_phase);
    let count = frame.cols as usize * frame.rows as usize;
    let mut cells = vec![
        CellInstance {
            bg: [0.0; 4],
            fg: [0.0; 4],
            uv: [0.0; 4],
            packed: 0,
            _pad: [0; 3],
        };
        count
    ];
    let font_size = packet.metrics.font_size_fp * packet.metrics.scale();
    let letter_spacing = packet.metrics.letter_spacing_fp * packet.metrics.scale();
    let cell_w = packet.metrics.cell_width_px.max(1.0).round() as u32;
    let cell_h = packet.metrics.cell_height_px.max(1.0).round() as u32;

    for (index, cell) in frame.cells.iter().enumerate() {
        if index >= count {
            break;
        }
        let col = (index % frame.cols as usize) as u16;
        let row = (index / frame.cols as usize) as u16;
        let (mut fg, mut bg) = cell.paint_colors(frame.default_fg, frame.default_bg);
        let mut packed = 0u32;
        if cell.underline || cell.underline_kind > 0 {
            packed |= FLAG_UNDERLINE;
            let kind = if cell.underline_kind == 0 {
                1
            } else {
                u32::from(cell.underline_kind).min(7)
            };
            packed |= kind << 5;
        }
        if cell.strikethrough {
            packed |= FLAG_STRIKE;
        }
        if cell.is_spacer || cell.width == 0 {
            packed |= FLAG_SPACER;
        }
        if cell.width >= 2 {
            packed |= FLAG_WIDE;
        }

        let cursor_on_cell = cursor_visible
            && frame.cursor.visible
            && frame.cursor.row == row
            && (frame.cursor.col == col
                || (cell.width >= 2
                    && frame.cursor.col > col
                    && frame.cursor.col < col.saturating_add(u16::from(cell.width))));
        if cursor_on_cell {
            let cursor_color = frame.cursor.color.unwrap_or(fg);
            match frame.cursor.style {
                CursorVisualStyle::Block => {
                    bg = cursor_color;
                    fg = if cell.bg != 0 && cell.bg != cursor_color {
                        cell.bg
                    } else {
                        frame.default_bg
                    };
                    packed |= 1 << CURSOR_SHIFT;
                }
                CursorVisualStyle::Bar => {
                    fg = cursor_color;
                    packed |= 2 << CURSOR_SHIFT;
                }
                CursorVisualStyle::Underline => {
                    fg = cursor_color;
                    packed |= 3 << CURSOR_SHIFT;
                }
                CursorVisualStyle::BlockHollow => {
                    fg = cursor_color;
                    packed |= 4 << CURSOR_SHIFT;
                }
            }
        }

        let mut uv = [0.0f32; 4];
        let text = cell.grapheme.as_str();
        if !cell.is_spacer && text.chars().any(|ch| ch != ' ' && ch != '\0') {
            let width_px = if cell.width >= 2 { cell_w * 2 } else { cell_w };
            match glyphs.uv_for(
                text,
                RasterStyleKey {
                    font_size_bits: font_size.to_bits(),
                    letter_spacing_bits: letter_spacing.to_bits(),
                    bold: cell.bold,
                    italic: cell.italic,
                    width_px,
                    height_px: cell_h,
                },
                rasterizer,
                queue,
                atlas,
            ) {
                Ok(Some(cached)) => {
                    uv = cached.uv;
                    packed |= FLAG_HAS_GLYPH;
                    if cached.color {
                        packed |= FLAG_COLOR_GLYPH;
                    }
                }
                Ok(None) => glyphs.atlas_full = true,
                Err(error) => {
                    ohos_hilog_binding::error(format!(
                        "arkit_terminal: glyph rasterization failed: {error}"
                    ));
                }
            }
        }

        cells[index] = CellInstance {
            bg: argb_to_render_color(bg, srgb_target),
            fg: argb_to_render_color(fg, srgb_target),
            uv,
            packed,
            _pad: [0; 3],
        };
    }
    Ok(cells)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RasterStyleKey {
    font_size_bits: u64,
    letter_spacing_bits: u64,
    bold: bool,
    italic: bool,
    width_px: u32,
    height_px: u32,
}

#[derive(Clone, Copy)]
struct CachedGlyph {
    uv: [f32; 4],
    color: bool,
}

#[derive(Default)]
struct GlyphRegistry {
    by_style: FxHashMap<RasterStyleKey, FxHashMap<Arc<str>, CachedGlyph>>,
    atlas_full: bool,
}

impl GlyphRegistry {
    fn uv_for(
        &mut self,
        text: &str,
        style: RasterStyleKey,
        rasterizer: &mut GlyphRasterizer,
        queue: &wgpu::Queue,
        atlas: &mut AtlasTexture,
    ) -> Result<Option<CachedGlyph>, String> {
        let clean = if text.contains('\0') {
            Cow::Owned(text.replace('\0', "\u{fffd}"))
        } else {
            Cow::Borrowed(text)
        };
        if let Some(cached) = self
            .by_style
            .get(&style)
            .and_then(|entries| entries.get(clean.as_ref()))
            .copied()
        {
            return Ok(Some(cached));
        }
        let packed = rasterizer.rasterize(&clean, style)?;
        let color = bitmap_has_chroma(&packed);
        let Some((origin_x, origin_y)) = atlas.allocate(style.width_px, style.height_px) else {
            return Ok(None);
        };
        atlas.upload(
            queue,
            origin_x,
            origin_y,
            style.width_px,
            style.height_px,
            &packed,
        );
        let size = ATLAS_SIZE as f32;
        let cached = CachedGlyph {
            uv: [
                origin_x as f32 / size,
                origin_y as f32 / size,
                (origin_x + style.width_px) as f32 / size,
                (origin_y + style.height_px) as f32 / size,
            ],
            color,
        };
        let text: Arc<str> = Arc::from(clean.as_ref());
        self.by_style.entry(style).or_default().insert(text, cached);
        Ok(Some(cached))
    }

    fn clear(&mut self) {
        self.by_style.clear();
        self.atlas_full = false;
    }
}

struct GlyphRasterizer {
    fonts: FontCollection,
}

impl GlyphRasterizer {
    fn new() -> Self {
        Self {
            fonts: FontCollection::shared(),
        }
    }

    fn rasterize(&mut self, text: &str, style: RasterStyleKey) -> Result<Vec<u8>, String> {
        let width = style.width_px.max(1);
        let height = style.height_px.max(1);
        let bitmap = Bitmap::new(
            width,
            height,
            BitmapFormat {
                color: ColorFormat::Rgba8888,
                alpha: AlphaFormat::Premul,
            },
        );
        let canvas = Canvas::with_bitmap(bitmap);
        canvas.clear(0);
        let spec = GlyphSpec {
            text: Arc::from(text),
            style,
            emoji: is_emoji_text(text),
        };
        let mut typography = build_typography(&mut self.fonts, &spec);
        let measured_width = typography.longest_line().max(f64::EPSILON);
        let text_height = typography.height() as f32;
        let target_width = f64::from(width);
        let center = spec.emoji || spec.text.chars().any(|ch| east_asian_width(ch) > 1);
        let (offset_x, scale_x) = horizontal_glyph_placement(measured_width, target_width, center);
        let offset_y = (height as f32 - text_height) * 0.5;
        canvas.save();
        canvas.translate(offset_x as f32, offset_y);
        canvas.scale(scale_x as f32, 1.0);
        typography.paint(&canvas, 0.0, 0.0);
        canvas.restore();
        let bitmap = canvas
            .bitmap()
            .expect("Canvas::with_bitmap must retain its bitmap");
        let mut packed = vec![0u8; (width * height * 4) as usize];
        let ok = bitmap.read_pixels(
            ImageInfo::new(
                width as i32,
                height as i32,
                ColorFormat::Rgba8888,
                AlphaFormat::Premul,
            ),
            &mut packed,
            (width * 4) as usize,
            0,
            0,
        );
        if !ok {
            packed = bitmap.pixels().to_vec();
        }
        Ok(packed)
    }
}

struct GlyphSpec {
    text: Arc<str>,
    style: RasterStyleKey,
    emoji: bool,
}

fn pad_rgba_rows(src: &[u8], width: u32, height: u32) -> (Vec<u8>, u32) {
    let src_row = (width.saturating_mul(4)) as usize;
    let dst_row = src_row.div_ceil(COPY_ROW_ALIGN as usize) * COPY_ROW_ALIGN as usize;
    let dst_row = dst_row.max(src_row);
    if dst_row == src_row {
        return (src.to_vec(), dst_row as u32);
    }
    let mut out = vec![0u8; dst_row * height as usize];
    for y in 0..height as usize {
        let src_off = y * src_row;
        let dst_off = y * dst_row;
        if src_off + src_row <= src.len() {
            out[dst_off..dst_off + src_row].copy_from_slice(&src[src_off..src_off + src_row]);
        }
    }
    (out, dst_row as u32)
}

fn bitmap_has_chroma(rgba: &[u8]) -> bool {
    rgba.chunks_exact(4).any(|pixel| {
        if pixel[3] < 24 {
            return false;
        }
        let max = pixel[0].max(pixel[1]).max(pixel[2]);
        let min = pixel[0].min(pixel[1]).min(pixel[2]);
        max.saturating_sub(min) > 18
    })
}

fn is_emoji_text(text: &str) -> bool {
    text.chars().any(|ch| {
        let u = ch as u32;
        matches!(
            u,
            0x231A..=0x231B
                | 0x23E9..=0x23EC
                | 0x23F0
                | 0x23F3
                | 0x25FD..=0x25FE
                | 0x2600..=0x27BF
                | 0x2B1B..=0x2B1C
                | 0x2B50
                | 0x2B55
                | 0xFE0F
                | 0x200D
                | 0x1F000..=0x1FAFF
        )
    })
}

fn horizontal_glyph_placement(measured_width: f64, target_width: f64, center: bool) -> (f64, f64) {
    let measured_width = measured_width.max(f64::EPSILON);
    let target_width = target_width.max(1.0);
    let scale = (target_width / measured_width).min(1.0);
    let painted_width = measured_width * scale;
    let offset = if center {
        ((target_width - painted_width) * 0.5).max(0.0)
    } else {
        0.0
    };
    (offset, scale)
}

fn build_typography(fonts: &mut FontCollection, spec: &GlyphSpec) -> Typography {
    let mut typography_style = TypographyStyle::new();
    typography_style.set_max_lines(1);
    let mut text_style = TextStyle::new();
    text_style.set_color(0xFFFF_FFFF);
    text_style.set_font_size(f64::from_bits(spec.style.font_size_bits));
    text_style.set_font_style(FontStyle::new(
        if spec.style.bold && !spec.emoji {
            FontWeight::W700
        } else {
            FontWeight::W400
        },
        FontWidth::Normal,
        if spec.style.italic && !spec.emoji {
            FontSlant::Italic
        } else {
            FontSlant::Normal
        },
    ));
    // Shared collection still needs a family list that can fall through to
    // HarmonyOS color emoji. Monospace-only drops COLR/CBDT faces.
    text_style.set_font_families(&[
        "monospace",
        "HarmonyOS Sans",
        "HarmonyOS Sans SC",
        "Noto Color Emoji",
        "Noto Sans CJK SC",
    ]);
    if spec.emoji {
        text_style.set_letter_spacing(0.0);
    } else {
        text_style.set_letter_spacing(f64::from_bits(spec.style.letter_spacing_bits));
    }
    let mut builder = TypographyBuilder::new(&mut typography_style, fonts);
    builder.push_text_style(&mut text_style);
    builder.add_text(&spec.text);
    builder.pop_text_style();
    let mut typography = builder.build();
    typography.layout(1_000_000.0);
    typography
}

fn argb_to_render_color(color: u32, srgb_target: bool) -> [f32; 4] {
    let convert = |byte: u32| {
        let channel = byte as f32 / 255.0;
        if srgb_target {
            srgb_to_linear(channel)
        } else {
            channel
        }
    };
    [
        convert((color >> 16) & 0xFF),
        convert((color >> 8) & 0xFF),
        convert(color & 0xFF),
        ((color >> 24) & 0xFF) as f32 / 255.0,
    ]
}

fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn argb_to_wgpu(color: u32, srgb_target: bool) -> wgpu::Color {
    let rgba = argb_to_render_color(color, srgb_target);
    wgpu::Color {
        r: f64::from(rgba[0]),
        g: f64::from(rgba[1]),
        b: f64::from(rgba[2]),
        a: f64::from(rgba[3]),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        argb_to_render_color, bitmap_has_chroma, horizontal_glyph_placement, is_emoji_text,
        pad_rgba_rows, srgb_to_linear, COPY_ROW_ALIGN,
    };

    #[test]
    fn srgb_surface_colors_are_converted_to_linear_shader_values() {
        let dark = argb_to_render_color(0xFF0B_1220, true);
        assert!((dark[0] - srgb_to_linear(11.0 / 255.0)).abs() < 0.000_001);
        assert!((dark[1] - srgb_to_linear(18.0 / 255.0)).abs() < 0.000_001);
        assert!((dark[2] - srgb_to_linear(32.0 / 255.0)).abs() < 0.000_001);
        assert_eq!(dark[3], 1.0);
    }

    #[test]
    fn fullwidth_glyph_keeps_its_natural_aspect_and_is_centered() {
        let (offset, scale) = horizontal_glyph_placement(40.0, 48.0, true);
        assert_eq!(scale, 1.0);
        assert_eq!(offset, 4.0);
    }

    #[test]
    fn texture_uploads_pad_rows_to_copy_alignment() {
        let (padded, stride) = pad_rgba_rows(&[1, 2, 3, 4, 5, 6, 7, 8], 2, 1);
        assert_eq!(stride % COPY_ROW_ALIGN, 0);
        assert_eq!(&padded[..8], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(padded.len(), stride as usize);
    }

    #[test]
    fn chroma_detects_color_emoji_pixels_not_white_masks() {
        let mask = [255, 255, 255, 200, 0, 0, 0, 0];
        let emoji = [240, 40, 40, 200, 0, 0, 0, 0];
        assert!(!bitmap_has_chroma(&mask));
        assert!(bitmap_has_chroma(&emoji));
        assert!(is_emoji_text("🙂🚀"));
        assert!(!is_emoji_text("hello"));
    }

    #[test]
    fn oversized_glyph_is_reduced_but_not_offset() {
        let (offset, scale) = horizontal_glyph_placement(60.0, 48.0, true);
        assert_eq!(scale, 0.8);
        assert_eq!(offset, 0.0);
    }
}
