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
use arkit_hooks::{use_mounted_node, use_native_element_ref};
use arkit_prelude::*;
use ohos_arkui_binding::{
    common::node::ArkUINode,
    component::attribute::{ArkUIAttributeBasic, ArkUIEvent},
    types::advanced::NodeDirtyFlag,
};
use ohos_drawing_binding::{
    AlphaFormat, Bitmap, BitmapFormat, BlendMode, Brush, Canvas, ColorFormat, FilterMode,
    FontCollection, FontSlant, FontStyle, FontWeight, FontWidth, Image, Matrix, MipmapMode, Pen,
    Rect, SamplingOptions, ShaderEffect, ShadowLayer, TextStyle, TileMode, TypographyBuilder,
    TypographyStyle,
};
use ohos_native_drawing_sys::{
    OH_Drawing_CreateTextShadow, OH_Drawing_DestroyTextShadow, OH_Drawing_PointCreate,
    OH_Drawing_PointDestroy, OH_Drawing_SetTextShadow, OH_Drawing_TextStyleAddShadow,
};

use crate::theme::use_theme;

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_OPACITY: f32 = 0.14;
const DEFAULT_ROTATION_DEGREES: f32 = -22.0;
const DEFAULT_GAP_X: f32 = 80.0;
const DEFAULT_GAP_Y: f32 = 64.0;
const MAX_GAP_VP: f32 = 100_000.0;
const MAX_OFFSET_VP: f32 = 100_000.0;
const MAX_STROKE_WIDTH_VP: f32 = 64.0;
const MAX_SHADOW_BLUR_VP: f32 = 256.0;
const MAX_SHADOW_OFFSET_VP: f32 = 2_048.0;
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
    fn native_value(self) -> FontSlant {
        match self {
            Self::Normal => FontSlant::Normal,
            Self::Italic => FontSlant::Italic,
            Self::Oblique => FontSlant::Oblique,
        }
    }
}

/// Blend operation used when compositing the watermark over its content.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WatermarkBlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    Plus,
}

impl WatermarkBlendMode {
    fn native_value(self) -> BlendMode {
        match self {
            Self::Normal => BlendMode::SrcOver,
            Self::Multiply => BlendMode::Multiply,
            Self::Screen => BlendMode::Screen,
            Self::Overlay => BlendMode::Overlay,
            Self::Darken => BlendMode::Darken,
            Self::Lighten => BlendMode::Lighten,
            Self::ColorDodge => BlendMode::ColorDodge,
            Self::ColorBurn => BlendMode::ColorBurn,
            Self::HardLight => BlendMode::HardLight,
            Self::SoftLight => BlendMode::SoftLight,
            Self::Difference => BlendMode::Difference,
            Self::Exclusion => BlendMode::Exclusion,
            Self::Hue => BlendMode::Hue,
            Self::Saturation => BlendMode::Saturation,
            Self::Color => BlendMode::Color,
            Self::Luminosity => BlendMode::Luminosity,
            Self::Plus => BlendMode::Plus,
        }
    }
}

/// Outline applied to text watermark glyphs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WatermarkStroke {
    pub color: u32,
    /// Stroke width in vp.
    pub width: f32,
}

impl WatermarkStroke {
    pub fn new(color: u32, width: f32) -> Self {
        Self { color, width }
    }
}

/// Drop shadow applied to text or image watermark content.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WatermarkShadow {
    pub color: u32,
    /// Blur radius in vp.
    pub blur_radius: f32,
    /// Horizontal shadow offset in vp.
    pub offset_x: f32,
    /// Vertical shadow offset in vp.
    pub offset_y: f32,
}

impl WatermarkShadow {
    pub fn new(color: u32, blur_radius: f32, offset_x: f32, offset_y: f32) -> Self {
        Self {
            color,
            blur_radius,
            offset_x,
            offset_y,
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
    /// Horizontal displacement of each mark inside its repeating cell, in vp.
    pub offset_x: f32,
    /// Vertical displacement of each mark inside its repeating cell, in vp.
    pub offset_y: f32,
    /// Horizontal origin of the complete repeating grid, in vp.
    pub repeat_origin_x: f32,
    /// Vertical origin of the complete repeating grid, in vp.
    pub repeat_origin_y: f32,
    /// Blend operation used to composite the watermark over its children.
    pub blend_mode: WatermarkBlendMode,
    /// Optional text outline. Image sources ignore this field.
    pub stroke: Option<WatermarkStroke>,
    /// Optional drop shadow for text or image sources.
    pub shadow: Option<WatermarkShadow>,
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
            offset_x: 0.0,
            offset_y: 0.0,
            repeat_origin_x: 0.0,
            repeat_origin_y: 0.0,
            blend_mode: WatermarkBlendMode::Normal,
            stroke: None,
            shadow: None,
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
    offset_x: f32,
    offset_y: f32,
    repeat_origin_x: f32,
    repeat_origin_y: f32,
    blend_mode: WatermarkBlendMode,
    stroke: Option<WatermarkStroke>,
    shadow: Option<WatermarkShadow>,
}

impl WatermarkStyle {
    fn resolve(self, color: u32) -> ResolvedWatermarkStyle {
        let stroke = self.stroke.and_then(|stroke| {
            let width = finite_or(stroke.width, 0.0).clamp(0.0, MAX_STROKE_WIDTH_VP);
            (width > 0.0).then_some(WatermarkStroke {
                color: stroke.color,
                width,
            })
        });
        let shadow = self.shadow.map(|shadow| WatermarkShadow {
            color: shadow.color,
            blur_radius: finite_or(shadow.blur_radius, 0.0).clamp(0.0, MAX_SHADOW_BLUR_VP),
            offset_x: finite_or(shadow.offset_x, 0.0)
                .clamp(-MAX_SHADOW_OFFSET_VP, MAX_SHADOW_OFFSET_VP),
            offset_y: finite_or(shadow.offset_y, 0.0)
                .clamp(-MAX_SHADOW_OFFSET_VP, MAX_SHADOW_OFFSET_VP),
        });
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
            offset_x: finite_or(self.offset_x, 0.0).clamp(-MAX_OFFSET_VP, MAX_OFFSET_VP),
            offset_y: finite_or(self.offset_y, 0.0).clamp(-MAX_OFFSET_VP, MAX_OFFSET_VP),
            repeat_origin_x: finite_or(self.repeat_origin_x, 0.0)
                .clamp(-MAX_OFFSET_VP, MAX_OFFSET_VP),
            repeat_origin_y: finite_or(self.repeat_origin_y, 0.0)
                .clamp(-MAX_OFFSET_VP, MAX_OFFSET_VP),
            blend_mode: self.blend_mode,
            stroke,
            shadow,
        }
    }
}

impl ResolvedWatermarkStyle {
    fn raster_eq(&self, other: &Self) -> bool {
        self.color == other.color
            && self.font_size == other.font_size
            && self.font_weight == other.font_weight
            && self.font_style == other.font_style
            && self.font_family == other.font_family
            && self.rotation_degrees == other.rotation_degrees
            && self.gap_x == other.gap_x
            && self.gap_y == other.gap_y
            && self.offset_x == other.offset_x
            && self.offset_y == other.offset_y
            && self.stroke == other.stroke
            && self.shadow == other.shadow
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

    fn raster_eq(&self, other: &Self) -> bool {
        self.source == other.source && self.style.raster_eq(&other.style)
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
    let node_ref = use_native_element_ref();
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<u64>)));

    if changed {
        if let Some(node) = node_ref.current() {
            // SAFETY: dirty marking neither changes ownership nor event
            // routing, and the native borrow does not escape.
            let _ = unsafe { node.with_native(|node| node.mark_dirty(NodeDirtyFlag::NeedRender)) };
        }
    }

    let draw_state = state.clone();
    let registered_for_effect = registered_node.clone();
    use_mounted_node(node_ref.clone(), move |node| {
        let Some(node) = node else {
            registered_for_effect.set(None);
            return;
        };
        let native_key = node.epoch();
        if registered_for_effect.get() != Some(native_key) {
            let draw_state = draw_state.clone();
            // SAFETY: custom-draw is separate from renderer-owned normal node
            // events. The callback belongs to this mounted Custom node.
            let _ = unsafe {
                node.with_native_mut(|node| {
                    CustomEventNode(node).on_custom_draw(move |event| {
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
                        let canvas = Canvas::from_raw_borrowed(raw_canvas.cast());
                        draw_state.paint(&canvas, width, height, pixel_ratio);
                    });
                })
            };
            registered_for_effect.set(Some(native_key));
        }
        // SAFETY: dirty marking neither changes ownership nor event routing.
        let _ = unsafe { node.with_native(|node| node.mark_dirty(NodeDirtyFlag::NeedRender)) };
    });

    rsx! {
        custom {
            native_ref: node_ref,
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
        let raster_changed = !current.raster_eq(&next);
        *current = next;
        if raster_changed {
            self.tile.borrow_mut().take();
        }
        true
    }

    fn paint(&self, canvas: &Canvas, width: f32, height: f32, pixel_ratio: f32) {
        let config = self.config.borrow();
        if config.style.opacity <= 0.0 {
            return;
        }
        let mut tile = self.tile.borrow_mut();
        if tile
            .as_ref()
            .is_none_or(|tile| !tile.matches_pixel_ratio(pixel_ratio))
        {
            *tile = Some(WatermarkTile::new(&config, pixel_ratio));
        }
        let Some(tile) = tile.as_mut() else {
            return;
        };

        canvas.save();
        canvas.scale(pixel_ratio, pixel_ratio);
        tile.paint(canvas, width, height, &config.style);
        canvas.restore();
    }
}

struct WatermarkTile {
    resources: Option<WatermarkTileResources>,
    pixel_ratio: f32,
}

struct WatermarkTileResources {
    // Drop order matters: the brush references the shader.
    brush: Brush,
    _shader: ShaderEffect,
    tile_width: f32,
    tile_height: f32,
}

impl WatermarkTile {
    fn new(config: &WatermarkDrawConfig, pixel_ratio: f32) -> Self {
        let resources = match &config.source {
            ResolvedWatermarkSource::Text(content) => {
                Self::build_text(content, &config.style, pixel_ratio)
            }
            ResolvedWatermarkSource::Image(image) => {
                Self::build_image(image, &config.style, pixel_ratio)
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
        text_style.set_font_style(FontStyle::new(
            FontWeight::from_css(style.font_weight as u16),
            FontWidth::Normal,
            style.font_style.native_value(),
        ));
        if let Some(font_family) = style.font_family.as_deref() {
            text_style.set_font_families(&[font_family]);
        }
        let mut fill_brush = None;
        let mut stroke_pen = None;
        if let Some(stroke) = style.stroke {
            let mut brush = Brush::new();
            brush.set_anti_alias(true);
            brush.set_color(style.color);
            text_style.set_foreground_brush(&brush);
            fill_brush = Some(brush);

            let mut pen = Pen::new();
            pen.set_anti_alias(true);
            pen.set_color(stroke.color);
            pen.set_width(stroke.width);
            text_style.set_foreground_pen(&pen);
            stroke_pen = Some(pen);
        }
        let text_shadow = style.shadow.and_then(|shadow| {
            let native_shadow = OwnedNative::new(
                // SAFETY: The returned native owner is immediately wrapped.
                unsafe { OH_Drawing_CreateTextShadow() },
                OH_Drawing_DestroyTextShadow,
            )?;
            let offset = OwnedNative::new(
                // SAFETY: The returned native owner is immediately wrapped.
                unsafe { OH_Drawing_PointCreate(shadow.offset_x, shadow.offset_y) },
                OH_Drawing_PointDestroy,
            )?;
            // SAFETY: All arguments remain live through this call. Adding the
            // shadow copies its values into the native text style.
            unsafe {
                OH_Drawing_SetTextShadow(
                    native_shadow.as_ptr(),
                    shadow.color,
                    offset.as_ptr(),
                    shadow.blur_radius as f64,
                );
                OH_Drawing_TextStyleAddShadow(text_style.as_ptr(), native_shadow.as_ptr());
            }
            Some((native_shadow, offset))
        });
        let mut fonts = FontCollection::global_instance().unwrap_or_default();
        let mut builder = TypographyBuilder::new(&mut typography_style, &mut fonts);
        builder.push_text_style(&mut text_style);
        builder.add_text(content);
        builder.pop_text_style();
        let mut typography = builder.build();
        typography.layout(TEXT_LAYOUT_WIDTH);

        let text_width = finite_positive(typography.longest_line() as f32, style.font_size);
        let text_height = finite_positive(typography.height() as f32, style.font_size);
        let resources = Self::build_resources(
            style,
            text_width,
            text_height,
            MarkEffectOutsets::for_text(style),
            pixel_ratio,
            |canvas, left, top, _| {
                typography.paint(canvas, left as f64, top as f64);
                true
            },
        );
        drop(text_shadow);
        drop(stroke_pen);
        drop(fill_brush);
        resources
    }

    fn build_image(
        image: &ResolvedWatermarkImage,
        style: &ResolvedWatermarkStyle,
        pixel_ratio: f32,
    ) -> Option<WatermarkTileResources> {
        let mut painter_initialized = false;
        let mut painter = None;
        Self::build_resources(
            style,
            image.width,
            image.height,
            MarkEffectOutsets::for_image(style),
            pixel_ratio,
            move |canvas, left, top, raster| {
                if !painter_initialized {
                    painter = ImageMarkPainter::new(image, raster, style.shadow);
                    painter_initialized = true;
                }
                let Some(painter) = painter.as_ref() else {
                    return false;
                };
                painter.paint(canvas, left, top, image.width, image.height);
                true
            },
        )
    }

    fn build_resources(
        style: &ResolvedWatermarkStyle,
        mark_width: f32,
        mark_height: f32,
        effect_outsets: MarkEffectOutsets,
        pixel_ratio: f32,
        mut paint_mark: impl FnMut(&Canvas, f32, f32, &TileRasterPlan) -> bool,
    ) -> Option<WatermarkTileResources> {
        let layout = MarkTileLayout::new(mark_width, mark_height, effect_outsets, style);
        let raster = TileRasterPlan::new(layout.tile_width, layout.tile_height, pixel_ratio);

        let bitmap = Bitmap::new(
            raster.width_pixels,
            raster.height_pixels,
            BitmapFormat {
                color: ColorFormat::Rgba8888,
                alpha: AlphaFormat::Premul,
            },
        );
        let tile_canvas = Canvas::with_bitmap(bitmap);
        tile_canvas.clear(0x00000000);
        tile_canvas.scale(raster.scale, raster.scale);
        if !layout.paint_marks(&tile_canvas, style.rotation_degrees, |canvas, left, top| {
            paint_mark(canvas, left, top, &raster)
        }) {
            return None;
        }

        let image = Image::from_bitmap(tile_canvas.bitmap()?)?;
        let sampling = SamplingOptions::new(FilterMode::Linear, MipmapMode::None);
        // The shader matrix maps the high-density raster tile back to logical
        // canvas coordinates.
        let inverse_scale = raster.scale.recip();
        let matrix = Matrix::from_affine(inverse_scale, 0.0, 0.0, inverse_scale, 0.0, 0.0);
        let shader = ShaderEffect::image(
            &image,
            TileMode::Repeat,
            TileMode::Repeat,
            &sampling,
            Some(&matrix),
        )?;
        let mut brush = Brush::new();
        brush.set_anti_alias(true);
        brush.set_shader_effect(Some(&shader));

        Some(WatermarkTileResources {
            brush,
            _shader: shader,
            tile_width: layout.tile_width,
            tile_height: layout.tile_height,
        })
    }

    fn matches_pixel_ratio(&self, pixel_ratio: f32) -> bool {
        (self.pixel_ratio - pixel_ratio).abs() <= f32::EPSILON
    }

    fn paint(&mut self, canvas: &Canvas, width: f32, height: f32, style: &ResolvedWatermarkStyle) {
        let Some(resources) = &mut self.resources else {
            return;
        };
        resources
            .brush
            .set_alpha((style.opacity * 255.0).round() as u8);
        resources
            .brush
            .set_blend_mode(style.blend_mode.native_value());
        let origin_x = repeat_phase(style.repeat_origin_x, resources.tile_width);
        let origin_y = repeat_phase(style.repeat_origin_y, resources.tile_height);
        let rect = Rect::new(-origin_x, -origin_y, width - origin_x, height - origin_y);
        canvas.save();
        canvas.translate(origin_x, origin_y);
        canvas.attach_brush(&resources.brush);
        canvas.draw_rect(&rect);
        canvas.detach_brush();
        canvas.restore();
    }
}

struct ImageMarkPainter {
    // Drop order matters: the brush references the shadow layer.
    shadow_brush: Option<Brush>,
    _shadow_layer: Option<ShadowLayer>,
    bitmap: Bitmap,
    sampling: SamplingOptions,
}

impl ImageMarkPainter {
    fn new(
        image: &ResolvedWatermarkImage,
        raster: &TileRasterPlan,
        shadow: Option<WatermarkShadow>,
    ) -> Option<Self> {
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
                return None;
            }
        };
        let width = pixels.width();
        let height = pixels.height();
        let row_stride = pixels.row_stride();
        let bitmap_format = BitmapFormat {
            color: ColorFormat::Rgba8888,
            alpha: drawing_alpha_format(pixels.alpha_type()),
        };
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
            return None;
        }

        let mut bitmap = Bitmap::new(width, height, bitmap_format);
        let tight_stride = usize::try_from(width).ok()?.checked_mul(4)?;
        let source_stride = usize::try_from(row_stride).ok()?;
        for row in 0..usize::try_from(height).ok()? {
            let source_start = row.checked_mul(source_stride)?;
            let source_end = source_start.checked_add(tight_stride)?;
            let destination_start = row.checked_mul(tight_stride)?;
            let destination_end = destination_start.checked_add(tight_stride)?;
            bitmap.pixels_mut()[destination_start..destination_end]
                .copy_from_slice(&pixels.pixels_mut()[source_start..source_end]);
        }
        let sampling = SamplingOptions::new(FilterMode::Linear, MipmapMode::None);
        let shadow_layer = shadow.and_then(|shadow| {
            ShadowLayer::new(
                shadow.blur_radius,
                shadow.offset_x,
                shadow.offset_y,
                shadow.color,
            )
        });
        let shadow_brush = shadow_layer.as_ref().map(|shadow_layer| {
            let mut brush = Brush::new();
            brush.set_anti_alias(true);
            brush.set_shadow_layer(Some(shadow_layer));
            brush
        });

        Some(Self {
            shadow_brush,
            _shadow_layer: shadow_layer,
            bitmap,
            sampling,
        })
    }

    fn paint(&self, canvas: &Canvas, left: f32, top: f32, width: f32, height: f32) {
        let Some(image) = Image::from_bitmap(&self.bitmap) else {
            return;
        };
        let source = Rect::new(
            0.0,
            0.0,
            self.bitmap.width() as f32,
            self.bitmap.height() as f32,
        );
        let destination = Rect::new(left, top, left + width, top + height);
        if let Some(brush) = &self.shadow_brush {
            canvas.attach_brush(brush);
        }
        canvas.draw_image_rect(&image, &source, &destination, &self.sampling);
        if self.shadow_brush.is_some() {
            canvas.detach_brush();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MarkEffectOutsets {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl MarkEffectOutsets {
    fn for_text(style: &ResolvedWatermarkStyle) -> Self {
        Self::new(
            style.stroke.map_or(0.0, |stroke| stroke.width / 2.0),
            style.shadow,
        )
    }

    fn for_image(style: &ResolvedWatermarkStyle) -> Self {
        Self::new(0.0, style.shadow)
    }

    fn new(stroke: f32, shadow: Option<WatermarkShadow>) -> Self {
        let Some(shadow) = shadow else {
            return Self {
                left: stroke,
                top: stroke,
                right: stroke,
                bottom: stroke,
            };
        };
        let blur = shadow.blur_radius * 2.0;
        Self {
            left: stroke + blur + (-shadow.offset_x).max(0.0),
            top: stroke + blur + (-shadow.offset_y).max(0.0),
            right: stroke + blur + shadow.offset_x.max(0.0),
            bottom: stroke + blur + shadow.offset_y.max(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RotatedMarkBounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl RotatedMarkBounds {
    fn new(
        mark_width: f32,
        mark_height: f32,
        outsets: MarkEffectOutsets,
        rotation_degrees: f32,
    ) -> Self {
        let left = -mark_width / 2.0 - outsets.left;
        let top = -mark_height / 2.0 - outsets.top;
        let right = mark_width / 2.0 + outsets.right;
        let bottom = mark_height / 2.0 + outsets.bottom;
        let radians = rotation_degrees.to_radians();
        let sin = radians.sin();
        let cos = radians.cos();
        let mut bounds = Self {
            min_x: f32::INFINITY,
            min_y: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            max_y: f32::NEG_INFINITY,
        };
        for (x, y) in [(left, top), (right, top), (right, bottom), (left, bottom)] {
            let rotated_x = x * cos - y * sin;
            let rotated_y = x * sin + y * cos;
            bounds.min_x = bounds.min_x.min(rotated_x);
            bounds.min_y = bounds.min_y.min(rotated_y);
            bounds.max_x = bounds.max_x.max(rotated_x);
            bounds.max_y = bounds.max_y.max(rotated_y);
        }
        bounds
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MarkTileLayout {
    mark_width: f32,
    mark_height: f32,
    bounds: RotatedMarkBounds,
    tile_width: f32,
    tile_height: f32,
    center_x: f32,
    center_y: f32,
}

impl MarkTileLayout {
    fn new(
        mark_width: f32,
        mark_height: f32,
        effect_outsets: MarkEffectOutsets,
        style: &ResolvedWatermarkStyle,
    ) -> Self {
        let bounds = RotatedMarkBounds::new(
            mark_width,
            mark_height,
            effect_outsets,
            style.rotation_degrees,
        );
        let tile_width = (bounds.max_x - bounds.min_x + style.gap_x + TEXT_PADDING * 2.0).max(1.0);
        let tile_height = (bounds.max_y - bounds.min_y + style.gap_y + TEXT_PADDING * 2.0).max(1.0);
        let center_x = (tile_width - bounds.min_x - bounds.max_x) / 2.0
            + signed_repeat_phase(style.offset_x, tile_width);
        let center_y = (tile_height - bounds.min_y - bounds.max_y) / 2.0
            + signed_repeat_phase(style.offset_y, tile_height);
        Self {
            mark_width,
            mark_height,
            bounds,
            tile_width,
            tile_height,
            center_x,
            center_y,
        }
    }

    fn paint_marks(
        &self,
        canvas: &Canvas,
        rotation_degrees: f32,
        mut paint: impl FnMut(&Canvas, f32, f32) -> bool,
    ) -> bool {
        for row in -1_i32..=1 {
            for column in -1_i32..=1 {
                let center_x = self.center_x + column as f32 * self.tile_width;
                let center_y = self.center_y + row as f32 * self.tile_height;
                if center_x + self.bounds.max_x <= 0.0
                    || center_x + self.bounds.min_x >= self.tile_width
                    || center_y + self.bounds.max_y <= 0.0
                    || center_y + self.bounds.min_y >= self.tile_height
                {
                    continue;
                }
                canvas.save();
                canvas.rotate_degrees_around(rotation_degrees, center_x, center_y);
                let painted = paint(
                    canvas,
                    center_x - self.mark_width / 2.0,
                    center_y - self.mark_height / 2.0,
                );
                canvas.restore();
                if !painted {
                    return false;
                }
            }
        }
        true
    }
}

fn drawing_alpha_format(alpha_type: i32) -> AlphaFormat {
    match alpha_type {
        1 => AlphaFormat::Opaque,
        3 => AlphaFormat::Unpremul,
        _ => AlphaFormat::Premul,
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

fn repeat_phase(value: f32, period: f32) -> f32 {
    if value.is_finite() && period.is_finite() && period > 0.0 {
        value.rem_euclid(period)
    } else {
        0.0
    }
}

fn signed_repeat_phase(value: f32, period: f32) -> f32 {
    if value.is_finite() && period.is_finite() && period > 0.0 {
        (value + period / 2.0).rem_euclid(period) - period / 2.0
    } else {
        0.0
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
            offset_x: f32::INFINITY,
            offset_y: f32::NEG_INFINITY,
            repeat_origin_x: f32::NAN,
            repeat_origin_y: f32::INFINITY,
            stroke: Some(WatermarkStroke::new(0xFFFFFFFF, f32::NAN)),
            shadow: Some(WatermarkShadow::new(
                0x88000000,
                f32::INFINITY,
                4_000.0,
                -4_000.0,
            )),
            ..WatermarkStyle::default()
        }
        .resolve(0x22000000);

        assert_eq!(style.font_size, DEFAULT_FONT_SIZE);
        assert_eq!(style.font_family, None);
        assert_eq!(style.opacity, DEFAULT_OPACITY);
        assert_eq!(style.rotation_degrees, DEFAULT_ROTATION_DEGREES);
        assert_eq!(style.gap_x, 0.0);
        assert_eq!(style.gap_y, DEFAULT_GAP_Y);
        assert_eq!(style.offset_x, 0.0);
        assert_eq!(style.offset_y, 0.0);
        assert_eq!(style.repeat_origin_x, 0.0);
        assert_eq!(style.repeat_origin_y, 0.0);
        assert_eq!(style.stroke, None);
        assert_eq!(
            style.shadow,
            Some(WatermarkShadow::new(
                0x88000000,
                0.0,
                MAX_SHADOW_OFFSET_VP,
                -MAX_SHADOW_OFFSET_VP,
            ))
        );
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

    #[test]
    fn composite_only_changes_reuse_the_raster_tile() {
        let base = WatermarkStyle::default().resolve(0xFF000000);
        let mut composite = base.clone();
        composite.opacity = 0.8;
        composite.blend_mode = WatermarkBlendMode::Difference;
        composite.repeat_origin_x = 28.0;
        composite.repeat_origin_y = -16.0;
        assert!(base.raster_eq(&composite));

        composite.offset_x = 1.0;
        assert!(!base.raster_eq(&composite));
    }

    #[test]
    fn repeat_phases_wrap_without_changing_the_period() {
        assert_eq!(repeat_phase(-10.0, 100.0), 90.0);
        assert_eq!(repeat_phase(210.0, 100.0), 10.0);
        assert_eq!(signed_repeat_phase(60.0, 100.0), -40.0);
        assert_eq!(signed_repeat_phase(-60.0, 100.0), 40.0);
    }

    #[test]
    fn stroke_and_shadow_expand_the_cached_tile_bounds() {
        let plain_style = WatermarkStyle {
            rotation_degrees: 0.0,
            ..WatermarkStyle::default()
        }
        .resolve(0xFF000000);
        let effect_style = WatermarkStyle {
            rotation_degrees: 0.0,
            stroke: Some(WatermarkStroke::new(0xFFFFFFFF, 4.0)),
            shadow: Some(WatermarkShadow::new(0x88000000, 8.0, 6.0, -4.0)),
            ..WatermarkStyle::default()
        }
        .resolve(0xFF000000);
        let plain = MarkTileLayout::new(
            120.0,
            32.0,
            MarkEffectOutsets::for_text(&plain_style),
            &plain_style,
        );
        let effected = MarkTileLayout::new(
            120.0,
            32.0,
            MarkEffectOutsets::for_text(&effect_style),
            &effect_style,
        );

        assert!(effected.tile_width > plain.tile_width);
        assert!(effected.tile_height > plain.tile_height);
    }
}
