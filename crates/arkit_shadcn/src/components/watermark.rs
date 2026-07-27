//! High-density watermark overlay rendered as one native custom-draw node.
//!
//! The watermark text or image is rasterized once into a small transparent
//! tile. A native repeating image shader then covers the component with a
//! single draw call, so a tall document does not create one Dioxus/ArkUI node
//! or one layout per visible watermark.

use std::{
    cell::{Cell, RefCell},
    ptr::NonNull,
    rc::Rc,
    sync::Arc,
};

use arkit_arkui::ArkImageSource;
use arkit_hooks::use_ark_node;
use arkit_prelude::*;
use ohos_arkui_binding::{
    common::node::ArkUINode,
    component::attribute::{ArkUIAttributeBasic, ArkUIEvent},
    types::advanced::NodeDirtyFlag,
};
use ohos_drawing_binding::{
    Brush, Canvas, FontCollection, Rect, TextStyle, TypographyBuilder, TypographyStyle,
};
use ohos_native_drawing_sys::{
    OH_Drawing_AlphaFormat, OH_Drawing_AlphaFormat_ALPHA_FORMAT_OPAQUE,
    OH_Drawing_AlphaFormat_ALPHA_FORMAT_PREMUL, OH_Drawing_AlphaFormat_ALPHA_FORMAT_UNPREMUL,
    OH_Drawing_Bitmap, OH_Drawing_BitmapBuild, OH_Drawing_BitmapCreate,
    OH_Drawing_BitmapCreateFromPixels, OH_Drawing_BitmapDestroy, OH_Drawing_BitmapFormat,
    OH_Drawing_BrushSetShaderEffect, OH_Drawing_CanvasBind, OH_Drawing_CanvasDrawImageRect,
    OH_Drawing_CanvasRotate, OH_Drawing_ColorFormat_COLOR_FORMAT_RGBA_8888,
    OH_Drawing_FilterMode_FILTER_MODE_LINEAR, OH_Drawing_FontStyle_FONT_STYLE_ITALIC,
    OH_Drawing_FontStyle_FONT_STYLE_NORMAL, OH_Drawing_FontStyle_FONT_STYLE_OBLIQUE,
    OH_Drawing_Image, OH_Drawing_ImageBuildFromBitmap, OH_Drawing_ImageCreate,
    OH_Drawing_ImageDestroy, OH_Drawing_Image_Info, OH_Drawing_Matrix,
    OH_Drawing_MatrixCreateScale, OH_Drawing_MatrixDestroy, OH_Drawing_MipmapMode_MIPMAP_MODE_NONE,
    OH_Drawing_SamplingOptions, OH_Drawing_SamplingOptionsCreate,
    OH_Drawing_SamplingOptionsDestroy, OH_Drawing_ShaderEffect,
    OH_Drawing_ShaderEffectCreateImageShader, OH_Drawing_ShaderEffectDestroy,
    OH_Drawing_TileMode_REPEAT,
};

use crate::theme::use_theme;

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_OPACITY: f32 = 0.14;
const DEFAULT_ROTATION_DEGREES: f32 = -22.0;
const DEFAULT_GAP_X: f32 = 80.0;
const DEFAULT_GAP_Y: f32 = 64.0;
const MAX_GAP_VP: f32 = 100_000.0;
const MAX_IMAGE_MARK_EDGE_VP: f32 = 2_048.0;
const MAX_TILE_EDGE_PIXELS: u32 = 2_048;
const MAX_TILE_PIXELS: u64 = 2_097_152;
const TEXT_LAYOUT_WIDTH: f64 = 1_000_000.0;
const TEXT_PADDING: f32 = 2.0;

/// Font slant used by a text watermark.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WatermarkFontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl WatermarkFontStyle {
    fn native_value(self) -> i32 {
        match self {
            Self::Normal => OH_Drawing_FontStyle_FONT_STYLE_NORMAL as i32,
            Self::Italic => OH_Drawing_FontStyle_FONT_STYLE_ITALIC as i32,
            Self::Oblique => OH_Drawing_FontStyle_FONT_STYLE_OBLIQUE as i32,
        }
    }
}

/// Image watermark content and its logical display size.
#[derive(Debug, Clone, PartialEq)]
pub struct WatermarkImage {
    pub source: ArkImageSource,
    /// Display width in vp before rotation.
    pub width: f32,
    /// Display height in vp before rotation.
    pub height: f32,
}

impl WatermarkImage {
    pub fn new(source: ArkImageSource, width: f32, height: f32) -> Self {
        Self {
            source,
            width,
            height,
        }
    }
}

/// Content repeated by [`Watermark`].
#[derive(Debug, Clone, PartialEq)]
pub enum WatermarkSource {
    Text(String),
    Image(WatermarkImage),
}

impl WatermarkSource {
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    pub fn image(source: ArkImageSource, width: f32, height: f32) -> Self {
        Self::Image(WatermarkImage::new(source, width, height))
    }
}

/// Visual configuration for [`Watermark`].
#[derive(Debug, Clone, PartialEq)]
pub struct WatermarkStyle {
    /// Text color. `None` uses the active theme foreground; ignored for images.
    pub color: Option<u32>,
    /// Font size in vp.
    pub font_size: f32,
    /// CSS-like font weight (`100..=900`).
    pub font_weight: i32,
    /// Normal, italic, or oblique text.
    pub font_style: WatermarkFontStyle,
    /// Optional font family for text watermarks.
    pub font_family: Option<String>,
    /// Opacity applied to either text or image content (`0..=1`).
    pub opacity: f32,
    /// Clockwise rotation in degrees.
    pub rotation_degrees: f32,
    /// Empty horizontal space between repeated marks, in vp.
    pub gap_x: f32,
    /// Empty vertical space between repeated marks, in vp.
    pub gap_y: f32,
}

impl Default for WatermarkStyle {
    fn default() -> Self {
        Self {
            color: None,
            font_size: DEFAULT_FONT_SIZE,
            font_weight: 500,
            font_style: WatermarkFontStyle::Normal,
            font_family: None,
            opacity: DEFAULT_OPACITY,
            rotation_degrees: DEFAULT_ROTATION_DEGREES,
            gap_x: DEFAULT_GAP_X,
            gap_y: DEFAULT_GAP_Y,
        }
    }
}

/// Props for [`Watermark`].
#[derive(Props, Clone, PartialEq)]
pub struct WatermarkProps {
    /// Repeated text or image.
    pub source: WatermarkSource,
    /// Width of the watermark container.
    #[props(default = "100%".to_string())]
    pub width: String,
    /// Height of the watermark container. `auto` follows the wrapped content.
    #[props(default = "auto".to_string())]
    pub height: String,
    #[props(default)]
    pub style: WatermarkStyle,
    pub children: Element,
}

/// Overlays repeated text or an image without intercepting its children.
///
/// One cached native tile and one repeating-shader draw call are used
/// regardless of the wrapped content height. Tile raster memory is capped;
/// unusually large text or gaps reduce tile resolution instead of allocating
/// an unbounded bitmap.
#[component]
pub fn Watermark(props: WatermarkProps) -> Element {
    let theme = use_theme();
    let style = props.style.resolve(theme.colors.foreground);
    let config = WatermarkDrawConfig::new(props.source, style);

    rsx! {
        stack {
            width: props.width,
            height: props.height,
            clip: true,
            {props.children}
            WatermarkCanvas { config }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ResolvedWatermarkStyle {
    color: u32,
    font_size: f32,
    font_weight: i32,
    font_style: WatermarkFontStyle,
    font_family: Option<Arc<str>>,
    opacity: f32,
    rotation_degrees: f32,
    gap_x: f32,
    gap_y: f32,
}

impl WatermarkStyle {
    fn resolve(self, color: u32) -> ResolvedWatermarkStyle {
        ResolvedWatermarkStyle {
            color: self.color.unwrap_or(color),
            font_size: finite_or(self.font_size, DEFAULT_FONT_SIZE).clamp(1.0, 512.0),
            font_weight: self.font_weight.clamp(100, 900),
            font_style: self.font_style,
            font_family: self
                .font_family
                .filter(|family| !family.is_empty() && !family.contains('\0'))
                .map(Arc::from),
            opacity: finite_or(self.opacity, DEFAULT_OPACITY).clamp(0.0, 1.0),
            rotation_degrees: normalize_degrees(finite_or(
                self.rotation_degrees,
                DEFAULT_ROTATION_DEGREES,
            )),
            gap_x: finite_or(self.gap_x, DEFAULT_GAP_X).clamp(0.0, MAX_GAP_VP),
            gap_y: finite_or(self.gap_y, DEFAULT_GAP_Y).clamp(0.0, MAX_GAP_VP),
        }
    }
}

#[derive(Clone, PartialEq)]
struct WatermarkDrawConfig {
    source: ResolvedWatermarkSource,
    style: ResolvedWatermarkStyle,
}

impl WatermarkDrawConfig {
    fn new(source: WatermarkSource, style: ResolvedWatermarkStyle) -> Self {
        let source = match source {
            WatermarkSource::Text(content) => {
                let content = if content.contains('\0') {
                    content.replace('\0', "\u{fffd}")
                } else {
                    content
                };
                ResolvedWatermarkSource::Text(Arc::from(content))
            }
            WatermarkSource::Image(image) => {
                ResolvedWatermarkSource::Image(ResolvedWatermarkImage {
                    source: image.source,
                    width: finite_or(image.width, 1.0).clamp(1.0, MAX_IMAGE_MARK_EDGE_VP),
                    height: finite_or(image.height, 1.0).clamp(1.0, MAX_IMAGE_MARK_EDGE_VP),
                })
            }
        };
        Self { source, style }
    }
}

#[derive(Clone, PartialEq)]
enum ResolvedWatermarkSource {
    Text(Arc<str>),
    Image(ResolvedWatermarkImage),
}

#[derive(Clone, PartialEq)]
struct ResolvedWatermarkImage {
    source: ArkImageSource,
    width: f32,
    height: f32,
}

#[derive(Props, Clone, PartialEq)]
struct WatermarkCanvasProps {
    config: WatermarkDrawConfig,
}

#[component]
fn WatermarkCanvas(props: WatermarkCanvasProps) -> Element {
    let initial_config = props.config.clone();
    let state = use_hook(move || Rc::new(WatermarkRenderState::new(initial_config)));
    let changed = state.update(props.config);
    let node_ref = use_ark_node();
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<usize>)));

    if changed {
        if let Some(node) = node_ref.peek() {
            let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
        }
    }

    let draw_state = state.clone();
    let registered_for_effect = registered_node.clone();
    use_effect(move || {
        let Some(node) = node_ref.get() else {
            return;
        };
        let native_key = node.borrow().raw_handle() as usize;
        if registered_for_effect.get() != Some(native_key) {
            let draw_state = draw_state.clone();
            CustomEventNode(&mut node.borrow_mut()).on_custom_draw(move |event| {
                let Some(draw_context) = event.draw_context_in_draw() else {
                    return;
                };
                let Some(raw_canvas) = draw_context.canvas() else {
                    return;
                };
                let pixel_ratio = display_pixel_ratio();
                let size = draw_context.size();
                let width = size.width as f32 / pixel_ratio;
                let height = size.height as f32 / pixel_ratio;
                if width <= 0.0 || height <= 0.0 {
                    return;
                }
                // SAFETY: ArkUI owns the canvas for this synchronous callback.
                // The borrowed wrapper is destroyed without releasing it and
                // cannot escape this closure.
                let canvas = unsafe { Canvas::from_raw_borrowed(raw_canvas.as_ptr().cast()) };
                draw_state.paint(&canvas, width, height, pixel_ratio);
            });
            registered_for_effect.set(Some(native_key));
        }
        let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
    });

    rsx! {
        custom {
            width: "100%",
            height: "100%",
            hit_test_behavior: "none",
        }
    }
}

struct WatermarkRenderState {
    config: RefCell<WatermarkDrawConfig>,
    tile: RefCell<Option<WatermarkTile>>,
}

impl WatermarkRenderState {
    fn new(config: WatermarkDrawConfig) -> Self {
        Self {
            config: RefCell::new(config),
            tile: RefCell::new(None),
        }
    }

    fn update(&self, next: WatermarkDrawConfig) -> bool {
        let mut current = self.config.borrow_mut();
        if *current == next {
            return false;
        }
        *current = next;
        self.tile.borrow_mut().take();
        true
    }

    fn paint(&self, canvas: &Canvas, width: f32, height: f32, pixel_ratio: f32) {
        let config = self.config.borrow();
        let mut tile = self.tile.borrow_mut();
        if tile
            .as_ref()
            .is_none_or(|tile| !tile.matches_pixel_ratio(pixel_ratio))
        {
            *tile = Some(WatermarkTile::new(&config, pixel_ratio));
        }
        let Some(tile) = tile.as_ref() else {
            return;
        };

        canvas.save();
        canvas.scale(pixel_ratio, pixel_ratio);
        tile.paint(canvas, width, height);
        canvas.restore();
    }
}

struct WatermarkTile {
    resources: Option<WatermarkTileResources>,
    pixel_ratio: f32,
}

struct WatermarkTileResources {
    // Drop order matters: the brush references the shader, the shader
    // references the image and auxiliary objects, and the image may share the
    // bitmap's pixels.
    brush: Brush,
    _shader: OwnedNative<OH_Drawing_ShaderEffect>,
    _matrix: OwnedNative<OH_Drawing_Matrix>,
    _sampling: OwnedNative<OH_Drawing_SamplingOptions>,
    _image: OwnedNative<OH_Drawing_Image>,
    _bitmap: OwnedNative<OH_Drawing_Bitmap>,
}

impl WatermarkTile {
    fn new(config: &WatermarkDrawConfig, pixel_ratio: f32) -> Self {
        let resources = if config.style.opacity <= 0.0 {
            None
        } else {
            match &config.source {
                ResolvedWatermarkSource::Text(content) => {
                    Self::build_text(content, &config.style, pixel_ratio)
                }
                ResolvedWatermarkSource::Image(image) => {
                    Self::build_image(image, &config.style, pixel_ratio)
                }
            }
        };
        Self {
            resources,
            pixel_ratio,
        }
    }

    fn build_text(
        content: &str,
        style: &ResolvedWatermarkStyle,
        pixel_ratio: f32,
    ) -> Option<WatermarkTileResources> {
        if content.is_empty() {
            return None;
        }
        let mut typography_style = TypographyStyle::new();
        let mut text_style = TextStyle::new();
        text_style.set_color(style.color);
        text_style.set_font_size(style.font_size as f64);
        text_style.set_font_weight(style.font_weight);
        text_style.set_font_style(style.font_style.native_value());
        if let Some(font_family) = style.font_family.as_deref() {
            text_style.set_font_families(&[font_family]);
        }
        let mut fonts = FontCollection::global_instance().unwrap_or_default();
        let mut builder = TypographyBuilder::new(&mut typography_style, &mut fonts);
        builder.push_text_style(&mut text_style);
        builder.add_text(content);
        builder.pop_text_style();
        let mut typography = builder.build();
        typography.layout(TEXT_LAYOUT_WIDTH);

        let text_width = finite_positive(typography.longest_line() as f32, style.font_size);
        let text_height = finite_positive(typography.height() as f32, style.font_size);
        Self::build_resources(
            style,
            text_width,
            text_height,
            pixel_ratio,
            |canvas, tile_width, tile_height, _| {
                typography.paint(
                    canvas,
                    ((tile_width - text_width) / 2.0) as f64,
                    ((tile_height - text_height) / 2.0) as f64,
                );
                true
            },
        )
    }

    fn build_image(
        image: &ResolvedWatermarkImage,
        style: &ResolvedWatermarkStyle,
        pixel_ratio: f32,
    ) -> Option<WatermarkTileResources> {
        Self::build_resources(
            style,
            image.width,
            image.height,
            pixel_ratio,
            |canvas, tile_width, tile_height, raster| {
                paint_image_mark(canvas, image, tile_width, tile_height, raster)
            },
        )
    }

    fn build_resources(
        style: &ResolvedWatermarkStyle,
        mark_width: f32,
        mark_height: f32,
        pixel_ratio: f32,
        paint_mark: impl FnOnce(&Canvas, f32, f32, &TileRasterPlan) -> bool,
    ) -> Option<WatermarkTileResources> {
        let radians = style.rotation_degrees.to_radians();
        let sin = radians.sin().abs();
        let cos = radians.cos().abs();
        let rotated_width = mark_width * cos + mark_height * sin;
        let rotated_height = mark_width * sin + mark_height * cos;
        let tile_width = (rotated_width + style.gap_x + TEXT_PADDING * 2.0).max(1.0);
        let tile_height = (rotated_height + style.gap_y + TEXT_PADDING * 2.0).max(1.0);
        let raster = TileRasterPlan::new(tile_width, tile_height, pixel_ratio);

        let bitmap = OwnedNative::new(
            // SAFETY: The returned native owner is immediately wrapped and
            // released by `OwnedNative`.
            unsafe { OH_Drawing_BitmapCreate() },
            OH_Drawing_BitmapDestroy,
        )?;
        let bitmap_format = OH_Drawing_BitmapFormat {
            colorFormat: OH_Drawing_ColorFormat_COLOR_FORMAT_RGBA_8888,
            alphaFormat: OH_Drawing_AlphaFormat_ALPHA_FORMAT_PREMUL,
        };
        // SAFETY: `bitmap` and `bitmap_format` remain live for this call.
        unsafe {
            OH_Drawing_BitmapBuild(
                bitmap.as_ptr(),
                raster.width_pixels,
                raster.height_pixels,
                &bitmap_format,
            );
        }

        {
            let tile_canvas = Canvas::new();
            // SAFETY: Both native objects remain live throughout the scoped
            // CPU render. The canvas is dropped before the bitmap is reused.
            unsafe { OH_Drawing_CanvasBind(tile_canvas.as_ptr(), bitmap.as_ptr()) };
            tile_canvas.clear(0x00000000);
            tile_canvas.scale(raster.scale, raster.scale);
            // SAFETY: The transform is local to this live canvas and the
            // canvas is discarded after the synchronous tile render.
            unsafe {
                OH_Drawing_CanvasRotate(
                    tile_canvas.as_ptr(),
                    style.rotation_degrees,
                    tile_width / 2.0,
                    tile_height / 2.0,
                );
            }
            if !paint_mark(&tile_canvas, tile_width, tile_height, &raster) {
                return None;
            }
        }

        let image = OwnedNative::new(
            // SAFETY: The returned native owner is immediately wrapped.
            unsafe { OH_Drawing_ImageCreate() },
            OH_Drawing_ImageDestroy,
        )?;
        // SAFETY: The image and bitmap are live. They are retained in the tile
        // in dependency order in case the native image shares bitmap pixels.
        if !unsafe { OH_Drawing_ImageBuildFromBitmap(image.as_ptr(), bitmap.as_ptr()) } {
            return None;
        }
        let sampling = OwnedNative::new(
            // SAFETY: The returned native owner is immediately wrapped.
            unsafe {
                OH_Drawing_SamplingOptionsCreate(
                    OH_Drawing_FilterMode_FILTER_MODE_LINEAR,
                    OH_Drawing_MipmapMode_MIPMAP_MODE_NONE,
                )
            },
            OH_Drawing_SamplingOptionsDestroy,
        )?;
        // The shader matrix maps the high-density raster tile back to logical
        // canvas coordinates.
        let inverse_scale = raster.scale.recip();
        let matrix = OwnedNative::new(
            // SAFETY: The returned native owner is immediately wrapped.
            unsafe { OH_Drawing_MatrixCreateScale(inverse_scale, inverse_scale, 0.0, 0.0) },
            OH_Drawing_MatrixDestroy,
        )?;
        let shader = OwnedNative::new(
            // SAFETY: All referenced native resources are kept alive in the
            // returned tile until after the shader is destroyed.
            unsafe {
                OH_Drawing_ShaderEffectCreateImageShader(
                    image.as_ptr(),
                    OH_Drawing_TileMode_REPEAT,
                    OH_Drawing_TileMode_REPEAT,
                    sampling.as_ptr(),
                    matrix.as_ptr(),
                )
            },
            OH_Drawing_ShaderEffectDestroy,
        )?;
        let mut brush = Brush::new();
        brush.set_anti_alias(true);
        brush.set_alpha((style.opacity * 255.0).round() as u8);
        // SAFETY: `shader` outlives `brush` by the field declaration order.
        unsafe { OH_Drawing_BrushSetShaderEffect(brush.as_ptr(), shader.as_ptr()) };

        Some(WatermarkTileResources {
            brush,
            _shader: shader,
            _matrix: matrix,
            _sampling: sampling,
            _image: image,
            _bitmap: bitmap,
        })
    }

    fn matches_pixel_ratio(&self, pixel_ratio: f32) -> bool {
        (self.pixel_ratio - pixel_ratio).abs() <= f32::EPSILON
    }

    fn paint(&self, canvas: &Canvas, width: f32, height: f32) {
        let Some(resources) = &self.resources else {
            return;
        };
        let rect = Rect::new(0.0, 0.0, width, height);
        canvas.attach_brush(&resources.brush);
        canvas.draw_rect(&rect);
        canvas.detach_brush();
    }
}

fn paint_image_mark(
    canvas: &Canvas,
    image: &ResolvedWatermarkImage,
    tile_width: f32,
    tile_height: f32,
    raster: &TileRasterPlan,
) -> bool {
    let decode_width = (image.width * raster.scale)
        .ceil()
        .clamp(1.0, MAX_TILE_EDGE_PIXELS as f32) as u32;
    let decode_height = (image.height * raster.scale)
        .ceil()
        .clamp(1.0, MAX_TILE_EDGE_PIXELS as f32) as u32;
    let source = image.source.with_dimensions(decode_width, decode_height);
    let mut pixels = match source.rgba_pixels() {
        Ok(pixels) => pixels,
        Err(error) => {
            ohos_hilog_binding::warn(format!(
                "arkit_shadcn: watermark image decode failed: {error}"
            ));
            return false;
        }
    };
    let width = pixels.width();
    let height = pixels.height();
    let row_stride = pixels.row_stride();
    let alpha_format = drawing_alpha_format(pixels.alpha_type());
    let minimum_len = usize::try_from(row_stride)
        .ok()
        .and_then(|stride| {
            usize::try_from(height)
                .ok()
                .and_then(|height| stride.checked_mul(height))
        })
        .unwrap_or(usize::MAX);
    if width == 0
        || height == 0
        || row_stride < width.saturating_mul(4)
        || pixels.pixels_mut().len() < minimum_len
    {
        ohos_hilog_binding::warn("arkit_shadcn: watermark image pixels are invalid");
        return false;
    }

    let mut image_info = OH_Drawing_Image_Info {
        width: width as i32,
        height: height as i32,
        colorType: OH_Drawing_ColorFormat_COLOR_FORMAT_RGBA_8888,
        alphaType: alpha_format,
    };
    let source_bitmap = match OwnedNative::new(
        // SAFETY: The pixel buffer remains live until after the bitmap and
        // image are destroyed at the end of this function.
        unsafe {
            OH_Drawing_BitmapCreateFromPixels(
                &mut image_info,
                pixels.pixels_mut().as_mut_ptr().cast(),
                row_stride,
            )
        },
        OH_Drawing_BitmapDestroy,
    ) {
        Some(bitmap) => bitmap,
        None => return false,
    };
    let source_image = match OwnedNative::new(
        // SAFETY: The returned native owner is immediately wrapped.
        unsafe { OH_Drawing_ImageCreate() },
        OH_Drawing_ImageDestroy,
    ) {
        Some(image) => image,
        None => return false,
    };
    // SAFETY: Both objects are live through the synchronous image draw.
    if !unsafe { OH_Drawing_ImageBuildFromBitmap(source_image.as_ptr(), source_bitmap.as_ptr()) } {
        return false;
    }
    let sampling = match OwnedNative::new(
        // SAFETY: The returned native owner is immediately wrapped.
        unsafe {
            OH_Drawing_SamplingOptionsCreate(
                OH_Drawing_FilterMode_FILTER_MODE_LINEAR,
                OH_Drawing_MipmapMode_MIPMAP_MODE_NONE,
            )
        },
        OH_Drawing_SamplingOptionsDestroy,
    ) {
        Some(sampling) => sampling,
        None => return false,
    };
    let destination = Rect::new(
        (tile_width - image.width) / 2.0,
        (tile_height - image.height) / 2.0,
        (tile_width + image.width) / 2.0,
        (tile_height + image.height) / 2.0,
    );
    // SAFETY: Canvas, image, destination, and sampling options all remain
    // valid for this synchronous draw call.
    unsafe {
        OH_Drawing_CanvasDrawImageRect(
            canvas.as_ptr(),
            source_image.as_ptr(),
            destination.as_ptr(),
            sampling.as_ptr(),
        );
    }
    true
}

fn drawing_alpha_format(alpha_type: i32) -> OH_Drawing_AlphaFormat {
    match alpha_type {
        1 => OH_Drawing_AlphaFormat_ALPHA_FORMAT_OPAQUE,
        3 => OH_Drawing_AlphaFormat_ALPHA_FORMAT_UNPREMUL,
        _ => OH_Drawing_AlphaFormat_ALPHA_FORMAT_PREMUL,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TileRasterPlan {
    scale: f32,
    width_pixels: u32,
    height_pixels: u32,
}

impl TileRasterPlan {
    fn new(width: f32, height: f32, preferred_scale: f32) -> Self {
        let width = finite_positive(width, 1.0) as f64;
        let height = finite_positive(height, 1.0) as f64;
        let preferred_scale = finite_positive(preferred_scale, 1.0) as f64;
        let edge = MAX_TILE_EDGE_PIXELS as f64;
        let area_scale = ((MAX_TILE_PIXELS as f64) / (width * height)).sqrt();
        let scale = preferred_scale
            .min(edge / width)
            .min(edge / height)
            .min(area_scale)
            .max(f64::MIN_POSITIVE);
        let width_pixels = (width * scale)
            .floor()
            .clamp(1.0, MAX_TILE_EDGE_PIXELS as f64) as u32;
        let height_pixels = (height * scale)
            .floor()
            .clamp(1.0, MAX_TILE_EDGE_PIXELS as f64) as u32;

        Self {
            scale: scale as f32,
            width_pixels,
            height_pixels,
        }
    }
}

struct OwnedNative<T> {
    raw: NonNull<T>,
    destroy: unsafe extern "C" fn(*mut T),
}

impl<T> OwnedNative<T> {
    fn new(raw: *mut T, destroy: unsafe extern "C" fn(*mut T)) -> Option<Self> {
        NonNull::new(raw).map(|raw| Self { raw, destroy })
    }

    fn as_ptr(&self) -> *mut T {
        self.raw.as_ptr()
    }
}

impl<T> Drop for OwnedNative<T> {
    fn drop(&mut self) {
        // SAFETY: `raw` is uniquely owned by this wrapper and destroyed once.
        unsafe { (self.destroy)(self.raw.as_ptr()) };
    }
}

struct CustomEventNode<'a>(&'a mut ArkUINode);

impl ArkUIAttributeBasic for CustomEventNode<'_> {
    fn raw(&self) -> &ArkUINode {
        self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        self.0
    }
}

impl ArkUIEvent for CustomEventNode<'_> {}

fn display_pixel_ratio() -> f32 {
    finite_positive(
        ohos_display_binding::default_display_virtual_pixel_ratio(),
        1.0,
    )
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

fn finite_positive(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn normalize_degrees(degrees: f32) -> f32 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raster_plan_respects_edge_and_memory_budgets() {
        let plan = TileRasterPlan::new(40_000.0, 20_000.0, 4.0);
        assert!(plan.width_pixels <= MAX_TILE_EDGE_PIXELS);
        assert!(plan.height_pixels <= MAX_TILE_EDGE_PIXELS);
        assert!(u64::from(plan.width_pixels) * u64::from(plan.height_pixels) <= MAX_TILE_PIXELS);
        assert!(plan.scale < 1.0);
    }

    #[test]
    fn normal_tile_keeps_device_scale() {
        let plan = TileRasterPlan::new(180.0, 120.0, 3.0);
        assert_eq!(plan.scale, 3.0);
        assert_eq!(plan.width_pixels, 540);
        assert_eq!(plan.height_pixels, 360);
    }

    #[test]
    fn invalid_style_values_are_normalized() {
        let style = WatermarkStyle {
            font_size: f32::NAN,
            font_family: Some("bad\0family".to_string()),
            opacity: f32::INFINITY,
            rotation_degrees: f32::INFINITY,
            gap_x: -1.0,
            gap_y: f32::NAN,
            ..WatermarkStyle::default()
        }
        .resolve(0x22000000);

        assert_eq!(style.font_size, DEFAULT_FONT_SIZE);
        assert_eq!(style.font_family, None);
        assert_eq!(style.opacity, DEFAULT_OPACITY);
        assert_eq!(style.rotation_degrees, DEFAULT_ROTATION_DEGREES);
        assert_eq!(style.gap_x, 0.0);
        assert_eq!(style.gap_y, DEFAULT_GAP_Y);
    }

    #[test]
    fn source_values_are_sanitized_and_bounded() {
        let style = WatermarkStyle::default().resolve(0xFF000000);
        let text = WatermarkDrawConfig::new(WatermarkSource::text("A\0B"), style.clone());
        assert!(matches!(
            text.source,
            ResolvedWatermarkSource::Text(ref content) if content.as_ref() == "A\u{fffd}B"
        ));

        let image = WatermarkDrawConfig::new(
            WatermarkSource::image(
                ArkImageSource::svg("logo", "<svg/>", 1, 1),
                f32::INFINITY,
                -20.0,
            ),
            style,
        );
        assert!(matches!(
            image.source,
            ResolvedWatermarkSource::Image(ResolvedWatermarkImage {
                width: 1.0,
                height: 1.0,
                ..
            })
        ));
    }
}
