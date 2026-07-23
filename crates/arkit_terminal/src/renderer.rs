use std::borrow::Cow;
use std::ffi::c_void;
use std::mem;
use std::ptr::NonNull;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glyphon::{
    Buffer as GlyphBuffer, Cache as GlyphCache, Color as GlyphColor, ContentType, CustomGlyph,
    FontSystem, Metrics as GlyphMetrics, RasterizeCustomGlyphRequest, RasterizedCustomGlyph,
    Resolution, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use ohos_drawing_binding::{
    Canvas, FontCollection, TextStyle, Typography, TypographyBuilder, TypographyStyle,
};
use ohos_native_drawing_sys::{
    OH_Drawing_AlphaFormat_ALPHA_FORMAT_PREMUL, OH_Drawing_Bitmap,
    OH_Drawing_BitmapCreateFromPixels, OH_Drawing_BitmapDestroy, OH_Drawing_CanvasBind,
    OH_Drawing_ColorFormat_COLOR_FORMAT_RGBA_8888, OH_Drawing_FontStyle_FONT_STYLE_ITALIC,
    OH_Drawing_FontStyle_FONT_STYLE_NORMAL, OH_Drawing_FontWeight_FONT_WEIGHT_400,
    OH_Drawing_FontWeight_FONT_WEIGHT_700, OH_Drawing_Image_Info,
    OH_Drawing_SetTextStyleLetterSpacing,
};
use rustc_hash::FxHashMap;
use wgpu::rwh::{OhosDisplayHandle, OhosNdkWindowHandle, RawDisplayHandle, RawWindowHandle};

use crate::frame::{east_asian_width, CursorVisualStyle, TerminalRun};
use crate::native_surface::NativeSurface;
use crate::worker::RenderPacket;

const GLYPH_ID_RESET_AT: u16 = 60_000;
const INITIAL_RECT_BUFFER_SIZE: u64 = 16 * 1024;

const RECT_SHADER: &str = r#"
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) bounds: vec4<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );
    let corner = corners[vertex_index];
    var out: VertexOut;
    out.position = vec4<f32>(
        mix(bounds.x, bounds.z, corner.x),
        mix(bounds.y, bounds.w, corner.y),
        0.0,
        1.0,
    );
    out.color = color;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

/// Ghostty-style renderer ownership for one terminal.
///
/// The worker owns the wgpu instance/device/queue, the XComponent surface,
/// the glyph atlas, and all reusable GPU buffers. The UI thread only publishes
/// the newest immutable viewport snapshot.
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
        let instance = wgpu::Instance::new(descriptor);
        let raw_surface = create_surface(&instance, &surface)?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&raw_surface),
            ..Default::default()
        }))
        .map_err(|error| format!("request_adapter: {error}"))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("arkit terminal device"),
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
        if self.glyphs.next_id >= GLYPH_ID_RESET_AT {
            self.resources.reset_glyph_atlas(&self.device, &self.queue);
            self.glyphs.clear();
        }

        let width = surface.config.width;
        let height = surface.config.height;
        let srgb_target = self.resources.format.is_srgb();
        let scene = build_scene(packet, width, height, srgb_target, &mut self.glyphs)?;
        self.resources
            .rectangles
            .upload(&self.device, &self.queue, &scene.rectangles);
        self.resources
            .viewport
            .update(&self.queue, Resolution { width, height });

        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: i32::try_from(width).unwrap_or(i32::MAX),
            bottom: i32::try_from(height).unwrap_or(i32::MAX),
        };
        let area = TextArea {
            buffer: &self.resources.empty_text,
            left: 0.0,
            top: 0.0,
            scale: 1.0,
            bounds,
            default_color: GlyphColor::rgb(255, 255, 255),
            custom_glyphs: &scene.glyphs,
        };
        let glyphs = &self.glyphs;
        let rasterizer = &mut self.rasterizer;
        self.resources
            .text
            .prepare_with_custom(
                &self.device,
                &self.queue,
                &mut self.resources.font_system,
                &mut self.resources.atlas,
                &self.resources.viewport,
                [area],
                &mut self.resources.swash,
                |request| {
                    let spec = glyphs.spec(request.id)?;
                    rasterizer
                        .rasterize(spec, request)
                        .map_err(|error| {
                            ohos_hilog_binding::error(format!(
                                "arkit_terminal: glyph rasterization failed: {error}"
                            ));
                        })
                        .ok()
                },
            )
            .map_err(|error| format!("prepare glyph atlas: {error}"))?;

        let (frame, reconfigure) = match surface.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                self.resources.atlas.trim();
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                surface.surface.configure(&self.device, &surface.config);
                self.resources.atlas.trim();
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.resources.atlas.trim();
                return Err("wgpu XComponent surface was lost".to_string());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                self.resources.atlas.trim();
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
            let color_attachment = Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(argb_to_wgpu(scene.clear, srgb_target)),
                    store: wgpu::StoreOp::Store,
                },
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("arkit terminal render pass"),
                color_attachments: &[color_attachment],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.resources
                .rectangles
                .draw(&mut pass, 0..scene.background_count);
            self.resources
                .text
                .render(&self.resources.atlas, &self.resources.viewport, &mut pass)
                .map_err(|error| format!("render glyph atlas: {error}"))?;
            self.resources.rectangles.draw(
                &mut pass,
                scene.background_count..scene.rectangles.len() as u32,
            );
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        self.resources.atlas.trim();
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

/// Field order is intentional: the wgpu surface is dropped before the retained
/// OHNativeWindow reference.
struct BoundSurface {
    surface: wgpu::Surface<'static>,
    _native: NativeSurface,
    config: wgpu::SurfaceConfiguration,
}

struct GpuResources {
    format: wgpu::TextureFormat,
    rectangles: RectRenderer,
    glyph_cache: GlyphCache,
    atlas: TextAtlas,
    viewport: Viewport,
    text: TextRenderer,
    font_system: FontSystem,
    swash: SwashCache,
    empty_text: GlyphBuffer,
}

impl GpuResources {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let rectangles = RectRenderer::new(device, format);
        let glyph_cache = GlyphCache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &glyph_cache, format);
        let viewport = Viewport::new(device, &glyph_cache);
        let text = TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let font_system = FontSystem::new_with_fonts(std::iter::empty());
        let swash = SwashCache::new();
        let empty_text = GlyphBuffer::new_empty(GlyphMetrics::new(1.0, 1.0));
        Self {
            format,
            rectangles,
            glyph_cache,
            atlas,
            viewport,
            text,
            font_system,
            swash,
            empty_text,
        }
    }

    fn reset_glyph_atlas(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.atlas = TextAtlas::new(device, queue, &self.glyph_cache, self.format);
    }
}

#[derive(Debug)]
struct Scene {
    clear: u32,
    rectangles: Vec<RectInstance>,
    background_count: u32,
    glyphs: Vec<CustomGlyph>,
}

fn build_scene(
    packet: &RenderPacket,
    width: u32,
    height: u32,
    srgb_target: bool,
    glyphs: &mut GlyphRegistry,
) -> Result<Scene, String> {
    let frame = &packet.frame;
    let cursor_visible = !(packet.cursor_blink && frame.cursor.blinking && !packet.cursor_phase);
    let clear = if frame.default_bg != 0 {
        frame.default_bg
    } else {
        packet.background_color
    };
    let rows = frame.rows_as_runs_with_cursor(cursor_visible);
    let grid = PixelGrid::new(packet, width, height);
    let mut backgrounds = Vec::with_capacity(rows.len() * 4);
    let mut overlays = Vec::with_capacity(rows.len() * 2);
    let mut custom_glyphs = Vec::with_capacity(rows.len() * 4);

    for (row_index, runs) in rows.iter().enumerate() {
        if row_index >= frame.rows as usize {
            break;
        }
        let top = grid.row_edge(row_index as u16);
        let bottom = grid.row_edge(row_index as u16 + 1);
        if top >= height as f32 {
            break;
        }
        let mut col = 0u16;
        for run in runs {
            let run_cols = run.cols.max(1);
            let left = grid.col_edge(col);
            col = col.saturating_add(run_cols);
            let right = grid.col_edge(col);
            if left >= width as f32 {
                break;
            }
            if run.bg != clear {
                backgrounds.push(RectInstance::from_pixels(
                    left,
                    top,
                    right,
                    bottom,
                    run.bg,
                    width,
                    height,
                    srgb_target,
                ));
            }
            if !run.text.trim_matches(' ').is_empty() {
                let id = glyphs.id_for(
                    &run.text,
                    RasterStyleKey {
                        font_size_bits: grid.font_size.to_bits(),
                        letter_spacing_bits: grid.letter_spacing.to_bits(),
                        bold: run.bold,
                        italic: run.italic,
                    },
                )?;
                custom_glyphs.push(CustomGlyph {
                    id,
                    left,
                    top,
                    width: (right - left).max(1.0),
                    height: (bottom - top).max(1.0),
                    color: Some(argb_to_glyph(run.fg)),
                    snap_to_physical_pixel: true,
                    metadata: 0,
                });
            }
            push_decorations(
                &mut overlays,
                run,
                left,
                top,
                right,
                bottom,
                grid.cell_width,
                width,
                height,
                srgb_target,
            );
        }
    }
    let background_count = backgrounds.len() as u32;
    backgrounds.extend(overlays);
    Ok(Scene {
        clear,
        rectangles: backgrounds,
        background_count,
        glyphs: custom_glyphs,
    })
}

#[allow(clippy::too_many_arguments)]
fn push_decorations(
    overlays: &mut Vec<RectInstance>,
    run: &TerminalRun,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    cell_width: f32,
    surface_width: u32,
    surface_height: u32,
    srgb_target: bool,
) {
    let height = bottom - top;
    let decoration_width = (height * 0.065).clamp(1.0, 3.0);
    if run.underline {
        overlays.push(RectInstance::from_pixels(
            left,
            bottom - decoration_width,
            right,
            bottom,
            run.fg,
            surface_width,
            surface_height,
            srgb_target,
        ));
    }
    if run.strikethrough {
        let y = top + height * 0.55;
        overlays.push(RectInstance::from_pixels(
            left,
            y,
            right,
            y + decoration_width,
            run.fg,
            surface_width,
            surface_height,
            srgb_target,
        ));
    }

    let cursor_color = run.cursor_color.unwrap_or(run.fg);
    match run.cursor {
        Some(CursorVisualStyle::Underline) => overlays.push(RectInstance::from_pixels(
            left,
            bottom - decoration_width,
            right,
            bottom,
            cursor_color,
            surface_width,
            surface_height,
            srgb_target,
        )),
        Some(CursorVisualStyle::Bar) => overlays.push(RectInstance::from_pixels(
            left,
            top,
            left + (cell_width * 0.18).clamp(1.0, 3.0),
            bottom,
            cursor_color,
            surface_width,
            surface_height,
            srgb_target,
        )),
        Some(CursorVisualStyle::BlockHollow) => {
            let stroke = (cell_width * 0.12).clamp(1.0, 2.0);
            overlays.extend([
                RectInstance::from_pixels(
                    left,
                    top,
                    right,
                    top + stroke,
                    cursor_color,
                    surface_width,
                    surface_height,
                    srgb_target,
                ),
                RectInstance::from_pixels(
                    left,
                    bottom - stroke,
                    right,
                    bottom,
                    cursor_color,
                    surface_width,
                    surface_height,
                    srgb_target,
                ),
                RectInstance::from_pixels(
                    left,
                    top,
                    left + stroke,
                    bottom,
                    cursor_color,
                    surface_width,
                    surface_height,
                    srgb_target,
                ),
                RectInstance::from_pixels(
                    right - stroke,
                    top,
                    right,
                    bottom,
                    cursor_color,
                    surface_width,
                    surface_height,
                    srgb_target,
                ),
            ]);
        }
        Some(CursorVisualStyle::Block) | None => {}
    }
}

struct PixelGrid {
    padding: f32,
    cell_width: f32,
    cell_height: f32,
    font_size: f64,
    letter_spacing: f64,
}

impl PixelGrid {
    fn new(packet: &RenderPacket, _width: u32, _height: u32) -> Self {
        let metrics = packet.metrics;
        let scale = metrics.scale() as f32;
        Self {
            padding: metrics.padding_vp as f32 * scale,
            cell_width: metrics.cell_width_px.max(f32::EPSILON),
            cell_height: metrics.cell_height_px.max(f32::EPSILON),
            font_size: metrics.font_size_fp * f64::from(scale),
            letter_spacing: metrics.letter_spacing_fp * f64::from(scale),
        }
    }

    fn col_edge(&self, col: u16) -> f32 {
        (self.padding + f32::from(col) * self.cell_width).round()
    }

    fn row_edge(&self, row: u16) -> f32 {
        (self.padding + f32::from(row) * self.cell_height).round()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct RectInstance {
    bounds: [f32; 4],
    color: [f32; 4],
}

impl RectInstance {
    #[allow(clippy::too_many_arguments)]
    fn from_pixels(
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
        color: u32,
        width: u32,
        height: u32,
        srgb_target: bool,
    ) -> Self {
        let width = width.max(1) as f32;
        let height = height.max(1) as f32;
        Self {
            bounds: [
                left.mul_add(2.0 / width, -1.0),
                1.0 - top * 2.0 / height,
                right.mul_add(2.0 / width, -1.0),
                1.0 - bottom * 2.0 / height,
            ],
            color: argb_to_render_color(color, srgb_target),
        }
    }
}

struct RectRenderer {
    pipeline: wgpu::RenderPipeline,
    instances: wgpu::Buffer,
    capacity: u64,
}

impl RectRenderer {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("arkit terminal rectangle shader"),
            source: wgpu::ShaderSource::Wgsl(RECT_SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("arkit terminal rectangle pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<RectInstance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x4,
                        1 => Float32x4
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
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("arkit terminal rectangle instances"),
            size: INITIAL_RECT_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            instances,
            capacity: INITIAL_RECT_BUFFER_SIZE,
        }
    }

    fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, instances: &[RectInstance]) {
        if instances.is_empty() {
            return;
        }
        let bytes = bytemuck::cast_slice(instances);
        let required = bytes.len() as u64;
        if required > self.capacity {
            self.instances.destroy();
            self.capacity = required.next_power_of_two().max(INITIAL_RECT_BUFFER_SIZE);
            self.instances = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("arkit terminal rectangle instances"),
                size: self.capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.instances, 0, bytes);
    }

    fn draw(&self, pass: &mut wgpu::RenderPass<'_>, instances: std::ops::Range<u32>) {
        if instances.is_empty() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, instances);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RasterStyleKey {
    font_size_bits: u64,
    letter_spacing_bits: u64,
    bold: bool,
    italic: bool,
}

#[derive(Debug)]
struct GlyphSpec {
    text: Arc<str>,
    style: RasterStyleKey,
}

#[derive(Default)]
struct GlyphRegistry {
    by_style: FxHashMap<RasterStyleKey, FxHashMap<Arc<str>, u16>>,
    specs: Vec<Option<GlyphSpec>>,
    next_id: u16,
}

impl GlyphRegistry {
    fn id_for(&mut self, text: &str, style: RasterStyleKey) -> Result<u16, String> {
        let clean = if text.contains('\0') {
            Cow::Owned(text.replace('\0', "\u{fffd}"))
        } else {
            Cow::Borrowed(text)
        };
        if let Some(id) = self
            .by_style
            .get(&style)
            .and_then(|entries| entries.get(clean.as_ref()))
            .copied()
        {
            return Ok(id);
        }
        let id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| "terminal glyph id space exhausted".to_string())?;
        self.next_id = id;
        let text: Arc<str> = Arc::from(clean.as_ref());
        self.by_style
            .entry(style)
            .or_default()
            .insert(text.clone(), id);
        let index = id as usize;
        if self.specs.len() <= index {
            self.specs.resize_with(index + 1, || None);
        }
        self.specs[index] = Some(GlyphSpec { text, style });
        Ok(id)
    }

    fn spec(&self, id: u16) -> Option<&GlyphSpec> {
        self.specs.get(id as usize)?.as_ref()
    }

    fn clear(&mut self) {
        self.by_style.clear();
        self.specs.clear();
        self.next_id = 0;
    }
}

struct GlyphRasterizer {
    fonts: FontCollection,
}

impl GlyphRasterizer {
    fn new() -> Self {
        Self {
            // Shared is the worker-safe OHOS font collection and includes
            // HarmonyOS system fallback for CJK and emoji.
            fonts: FontCollection::shared(),
        }
    }

    fn rasterize(
        &mut self,
        spec: &GlyphSpec,
        request: RasterizeCustomGlyphRequest,
    ) -> Result<RasterizedCustomGlyph, String> {
        let width = usize::from(request.width);
        let height = usize::from(request.height);
        let byte_len = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| "glyph bitmap size overflowed".to_string())?;
        let mut rgba = vec![0u8; byte_len];
        let mut image_info = OH_Drawing_Image_Info {
            width: i32::from(request.width),
            height: i32::from(request.height),
            colorType: OH_Drawing_ColorFormat_COLOR_FORMAT_RGBA_8888,
            alphaType: OH_Drawing_AlphaFormat_ALPHA_FORMAT_PREMUL,
        };
        // SAFETY: `rgba` remains allocated and writable until the bound canvas
        // and bitmap are both destroyed below.
        let raw_bitmap = unsafe {
            OH_Drawing_BitmapCreateFromPixels(
                &mut image_info,
                rgba.as_mut_ptr().cast::<c_void>(),
                u32::from(request.width) * 4,
            )
        };
        let bitmap = BorrowedBitmap::new(raw_bitmap)
            .ok_or_else(|| "OH_Drawing_BitmapCreateFromPixels returned null".to_string())?;
        let canvas = Canvas::new();
        // SAFETY: the canvas and bitmap are live for all synchronous paint
        // operations, and the pixel buffer outlives both.
        unsafe { OH_Drawing_CanvasBind(canvas.as_ptr(), bitmap.as_ptr()) };
        canvas.clear(0);

        let mut typography = build_typography(&mut self.fonts, spec);
        let measured_width = typography.longest_line().max(f64::EPSILON);
        let text_height = typography.height() as f32;
        let target_width = f64::from(request.width.max(1));
        let center_horizontally = spec.text.chars().any(|ch| east_asian_width(ch) > 1);
        let (offset_x, scale_x) =
            horizontal_glyph_placement(measured_width, target_width, center_horizontally);
        let offset_y = (request.height as f32 - text_height) * 0.5;
        canvas.save();
        canvas.translate(offset_x as f32, offset_y);
        canvas.scale(scale_x as f32, 1.0);
        typography.paint(&canvas, 0.0, 0.0);
        canvas.restore();
        drop(canvas);
        drop(bitmap);

        let mask = rgba.chunks_exact(4).map(|pixel| pixel[3]).collect();
        Ok(RasterizedCustomGlyph {
            data: mask,
            content_type: ContentType::Mask,
        })
    }
}

/// Fit native typography into a terminal run without widening glyph outlines.
///
/// Ghostty positions a glyph inside its cell box; it does not stretch a
/// full-width fallback face to consume all two-column advance. Oversized text
/// is reduced to avoid clipping, while CJK/emoji is centered at its natural
/// aspect ratio inside the two-column box.
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
    text_style.set_font_weight(if spec.style.bold {
        OH_Drawing_FontWeight_FONT_WEIGHT_700 as i32
    } else {
        OH_Drawing_FontWeight_FONT_WEIGHT_400 as i32
    });
    text_style.set_font_style(if spec.style.italic {
        OH_Drawing_FontStyle_FONT_STYLE_ITALIC as i32
    } else {
        OH_Drawing_FontStyle_FONT_STYLE_NORMAL as i32
    });
    text_style.set_font_families(&["monospace"]);
    // The safe wrapper has not exposed letter spacing yet.
    // SAFETY: `text_style` owns a live native style for this call.
    unsafe {
        OH_Drawing_SetTextStyleLetterSpacing(
            text_style.as_ptr(),
            f64::from_bits(spec.style.letter_spacing_bits),
        );
    }
    let mut builder = TypographyBuilder::new(&mut typography_style, fonts);
    builder.push_text_style(&mut text_style);
    builder.add_text(&spec.text);
    builder.pop_text_style();
    let mut typography = builder.build();
    typography.layout(1_000_000.0);
    typography
}

fn argb_to_glyph(color: u32) -> GlyphColor {
    GlyphColor::rgba(
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
        ((color >> 24) & 0xFF) as u8,
    )
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

struct BorrowedBitmap(NonNull<OH_Drawing_Bitmap>);

impl BorrowedBitmap {
    fn new(raw: *mut OH_Drawing_Bitmap) -> Option<Self> {
        NonNull::new(raw).map(Self)
    }

    fn as_ptr(&self) -> *mut OH_Drawing_Bitmap {
        self.0.as_ptr()
    }
}

impl Drop for BorrowedBitmap {
    fn drop(&mut self) {
        // SAFETY: this wrapper owns the bitmap object but not its external
        // pixels, and destroys it exactly once before the pixel Vec is moved.
        unsafe { OH_Drawing_BitmapDestroy(self.0.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::{
        argb_to_render_color, horizontal_glyph_placement, srgb_to_linear, GlyphRegistry,
        RasterStyleKey, RectInstance,
    };

    #[test]
    fn glyph_registry_reuses_cached_run_without_changing_identity() {
        let mut registry = GlyphRegistry::default();
        let style = RasterStyleKey {
            font_size_bits: 16.0_f64.to_bits(),
            letter_spacing_bits: 0.0_f64.to_bits(),
            bold: false,
            italic: false,
        };
        let first = registry.id_for("terminal", style).unwrap();
        let second = registry.id_for("terminal", style).unwrap();

        assert_eq!(first, second);
        assert_eq!(registry.specs.iter().flatten().count(), 1);
    }

    #[test]
    fn srgb_surface_colors_are_converted_to_linear_shader_values() {
        let dark = argb_to_render_color(0xFF0B_1220, true);

        assert!((dark[0] - srgb_to_linear(11.0 / 255.0)).abs() < 0.000_001);
        assert!((dark[1] - srgb_to_linear(18.0 / 255.0)).abs() < 0.000_001);
        assert!((dark[2] - srgb_to_linear(32.0 / 255.0)).abs() < 0.000_001);
        assert_eq!(dark[3], 1.0);
    }

    #[test]
    fn pixel_rect_maps_top_left_and_bottom_right_to_clip_space() {
        let rect = RectInstance::from_pixels(0.0, 0.0, 200.0, 100.0, 0xFFFF_FFFF, 200, 100, false);

        assert_eq!(rect.bounds, [-1.0, 1.0, 1.0, -1.0]);
    }

    #[test]
    fn fullwidth_glyph_keeps_its_natural_aspect_and_is_centered() {
        let (offset, scale) = horizontal_glyph_placement(40.0, 48.0, true);

        assert_eq!(scale, 1.0);
        assert_eq!(offset, 4.0);
    }

    #[test]
    fn oversized_glyph_is_reduced_but_not_offset() {
        let (offset, scale) = horizontal_glyph_placement(60.0, 48.0, true);

        assert_eq!(scale, 0.8);
        assert_eq!(offset, 0.0);
    }
}
