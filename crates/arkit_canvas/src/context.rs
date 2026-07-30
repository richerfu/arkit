use ohos_drawing_binding::{
    AlphaFormat, Bitmap, BitmapFormat, Brush, Canvas, ColorFormat, Path, PathEffect, Pen, Rect,
    SamplingOptions, ShadowLayer,
};

use crate::color_space::ColorSpaceTransform;
use crate::filter::PaintFilter;
use crate::native::{NativeCanvasExt as _, NativePenExt as _};
use crate::state::CanvasStyleState;
use crate::text::{TextLayout, TextPlacement};
use crate::{
    CanvasColor, CanvasFont, CanvasGradient, CanvasImage, CanvasImageSmoothingQuality,
    CanvasLineCap, CanvasLineJoin, CanvasPattern, CanvasPatternRepetition,
    CanvasRenderingContext2DSettings, CanvasResult, CanvasStyle, CanvasTextAlign,
    CanvasTextBaseline, CanvasTextMetrics, DomMatrix2D, FillRule, GlobalCompositeOperation,
    ImageData, ImageDataSettings, IntoCanvasFont, IntoCanvasRadii, IntoCanvasStyle, Path2D,
};

pub(crate) struct CanvasSurface {
    canvas: Canvas,
    width: f32,
    height: f32,
    pixel_width: u32,
    pixel_height: u32,
    device_pixel_ratio: f32,
    settings: CanvasRenderingContext2DSettings,
    state: CanvasStyleState,
    stack: Vec<CanvasStyleState>,
    current_path: Path2D,
}

impl CanvasSurface {
    pub(crate) fn display_pixel_ratio() -> f32 {
        let ratio = ohos_display_binding::default_display_virtual_pixel_ratio();
        if ratio.is_finite() && ratio > 0.0 {
            ratio
        } else {
            1.0
        }
    }

    pub(crate) fn new(
        width: f32,
        height: f32,
        pixel_width: u32,
        pixel_height: u32,
        device_pixel_ratio: f32,
        settings: CanvasRenderingContext2DSettings,
    ) -> Self {
        let settings = settings.resolved_for_native_bitmap();
        let bitmap = Bitmap::new(
            pixel_width.max(1),
            pixel_height.max(1),
            BitmapFormat {
                color: ColorFormat::Rgba8888,
                alpha: if settings.alpha {
                    AlphaFormat::Premul
                } else {
                    AlphaFormat::Opaque
                },
            },
        );
        let canvas = Canvas::with_bitmap(bitmap);
        canvas.clear(settings.blank_color());
        canvas.reset_dom_transform(device_pixel_ratio);
        // This save level owns the initial unclipped state. `reset()` restores
        // to it and immediately recreates it.
        canvas.save();
        Self {
            canvas,
            width,
            height,
            pixel_width,
            pixel_height,
            device_pixel_ratio,
            settings,
            state: CanvasStyleState::default(),
            stack: Vec::new(),
            current_path: Path2D::new(),
        }
    }

    pub(crate) fn matches(
        &self,
        pixel_width: u32,
        pixel_height: u32,
        device_pixel_ratio: f32,
        settings: CanvasRenderingContext2DSettings,
    ) -> bool {
        self.pixel_width == pixel_width.max(1)
            && self.pixel_height == pixel_height.max(1)
            && (self.device_pixel_ratio - device_pixel_ratio).abs() <= f32::EPSILON
            && self.settings == settings.resolved_for_native_bitmap()
    }

    pub(crate) fn clear_pixels(&self) {
        self.canvas.clear(self.settings.blank_color());
    }

    pub(crate) fn draw_to(&self, target: &Canvas) {
        if let Some(bitmap) = self.canvas.bitmap() {
            target.draw_bitmap(bitmap, 0.0, 0.0);
        }
    }

    pub(crate) fn snapshot(&self) -> Option<CanvasImage> {
        self.canvas
            .bitmap()
            .map(|bitmap| CanvasImage::from_canvas_bitmap(bitmap, self.settings.alpha))
    }

    pub(crate) fn context(&mut self) -> CanvasRenderingContext2D<'_> {
        CanvasRenderingContext2D::new(self)
    }
}

/// A synchronous Canvas 2D drawing context.
///
/// Coordinates use logical pixels with an origin at the top-left, positive x
/// to the right, and positive y downward. The context is valid only during a
/// [`crate::CanvasRenderer`] callback and cannot escape the native draw frame.
pub struct CanvasRenderingContext2D<'canvas> {
    canvas: &'canvas mut Canvas,
    width: f32,
    height: f32,
    device_pixel_ratio: f32,
    settings: CanvasRenderingContext2DSettings,
    state: &'canvas mut CanvasStyleState,
    stack: &'canvas mut Vec<CanvasStyleState>,
    current_path: &'canvas mut Path2D,
    brush: Brush,
    pen: Pen,
}

#[derive(Clone, Copy)]
struct PaintTransforms {
    native: DomMatrix2D,
    current: DomMatrix2D,
}

impl PaintTransforms {
    const fn at_base(current: DomMatrix2D) -> Self {
        Self {
            native: DomMatrix2D::IDENTITY,
            current,
        }
    }

    const fn on_current(current: DomMatrix2D) -> Self {
        Self {
            native: current,
            current,
        }
    }
}

impl<'canvas> CanvasRenderingContext2D<'canvas> {
    pub(crate) fn new(surface: &'canvas mut CanvasSurface) -> Self {
        Self {
            canvas: &mut surface.canvas,
            width: surface.width,
            height: surface.height,
            device_pixel_ratio: surface.device_pixel_ratio,
            settings: surface.settings,
            state: &mut surface.state,
            stack: &mut surface.stack,
            current_path: &mut surface.current_path,
            brush: Brush::new(),
            pen: Pen::new(),
        }
    }

    pub const fn width(&self) -> f32 {
        self.width
    }

    pub const fn height(&self) -> f32 {
        self.height
    }

    pub const fn device_pixel_ratio(&self) -> f32 {
        self.device_pixel_ratio
    }

    // --- Drawing state ----------------------------------------------------

    /// Push a copy of the complete drawing state and native clip/transform.
    pub fn save(&mut self) {
        self.canvas.save();
        self.stack.push(self.state.clone());
    }

    /// Restore the most recently saved state. An empty stack is a no-op.
    pub fn restore(&mut self) {
        if let Some(state) = self.stack.pop() {
            self.canvas.restore();
            *self.state = state;
        }
    }

    /// Reset state, clipping, current path, transform, and output bitmap.
    pub fn reset(&mut self) {
        while self.stack.pop().is_some() {
            self.canvas.restore();
        }
        self.canvas.restore();
        self.canvas.reset_dom_transform(self.device_pixel_ratio);
        self.canvas.save();
        self.canvas.clear(self.settings.blank_color());
        *self.state = CanvasStyleState::default();
        self.current_path.reset();
    }

    pub fn get_context_attributes(&self) -> CanvasRenderingContext2DSettings {
        self.settings
    }

    pub const fn is_context_lost(&self) -> bool {
        false
    }

    // --- Styles -----------------------------------------------------------

    pub fn fill_style(&self) -> CanvasStyle {
        self.state.fill_style.clone()
    }

    pub fn set_fill_style(&mut self, style: impl IntoCanvasStyle) {
        self.state.set_fill_style(style);
    }

    pub fn stroke_style(&self) -> CanvasStyle {
        self.state.stroke_style.clone()
    }

    pub fn set_stroke_style(&mut self, style: impl IntoCanvasStyle) {
        self.state.set_stroke_style(style);
    }

    pub const fn global_alpha(&self) -> f32 {
        self.state.global_alpha
    }

    pub fn set_global_alpha(&mut self, alpha: f32) {
        if alpha.is_finite() && (0.0..=1.0).contains(&alpha) {
            self.state.global_alpha = alpha;
        }
    }

    pub const fn global_composite_operation(&self) -> GlobalCompositeOperation {
        self.state.global_composite_operation
    }

    pub const fn set_global_composite_operation(&mut self, operation: GlobalCompositeOperation) {
        self.state.global_composite_operation = operation;
    }

    pub const fn line_width(&self) -> f32 {
        self.state.line_width
    }

    pub fn set_line_width(&mut self, width: f32) {
        if width.is_finite() && width > 0.0 {
            self.state.line_width = width;
        }
    }

    pub const fn line_cap(&self) -> CanvasLineCap {
        self.state.line_cap
    }

    pub const fn set_line_cap(&mut self, cap: CanvasLineCap) {
        self.state.line_cap = cap;
    }

    pub const fn line_join(&self) -> CanvasLineJoin {
        self.state.line_join
    }

    pub const fn set_line_join(&mut self, join: CanvasLineJoin) {
        self.state.line_join = join;
    }

    pub const fn miter_limit(&self) -> f32 {
        self.state.miter_limit
    }

    pub fn set_miter_limit(&mut self, limit: f32) {
        if limit.is_finite() && limit > 0.0 {
            self.state.miter_limit = limit;
        }
    }

    pub fn set_line_dash(&mut self, segments: &[f32]) {
        if segments
            .iter()
            .any(|segment| !segment.is_finite() || *segment < 0.0)
        {
            return;
        }
        self.state.line_dash.clear();
        self.state.line_dash.extend_from_slice(segments);
        if self.state.line_dash.len() % 2 == 1 {
            self.state.line_dash.extend_from_slice(segments);
        }
    }

    pub fn line_dash(&self) -> &[f32] {
        &self.state.line_dash
    }

    pub const fn line_dash_offset(&self) -> f32 {
        self.state.line_dash_offset
    }

    pub fn set_line_dash_offset(&mut self, offset: f32) {
        if offset.is_finite() {
            self.state.line_dash_offset = offset;
        }
    }

    pub const fn image_smoothing_enabled(&self) -> bool {
        self.state.image_smoothing_enabled
    }

    pub const fn set_image_smoothing_enabled(&mut self, enabled: bool) {
        self.state.image_smoothing_enabled = enabled;
    }

    pub const fn image_smoothing_quality(&self) -> CanvasImageSmoothingQuality {
        self.state.image_smoothing_quality
    }

    pub const fn set_image_smoothing_quality(&mut self, quality: CanvasImageSmoothingQuality) {
        self.state.image_smoothing_quality = quality;
    }

    pub const fn shadow_offset_x(&self) -> f32 {
        self.state.shadow_offset_x
    }

    pub fn set_shadow_offset_x(&mut self, value: f32) {
        if value.is_finite() {
            self.state.shadow_offset_x = value;
        }
    }

    pub const fn shadow_offset_y(&self) -> f32 {
        self.state.shadow_offset_y
    }

    pub fn set_shadow_offset_y(&mut self, value: f32) {
        if value.is_finite() {
            self.state.shadow_offset_y = value;
        }
    }

    pub const fn shadow_blur(&self) -> f32 {
        self.state.shadow_blur
    }

    pub fn set_shadow_blur(&mut self, value: f32) {
        if value.is_finite() && value >= 0.0 {
            self.state.shadow_blur = value;
        }
    }

    pub const fn shadow_color(&self) -> CanvasColor {
        self.state.shadow_color
    }

    pub fn set_shadow_color(&mut self, value: impl IntoCanvasStyle) {
        if let Some(CanvasStyle::Color(color)) = value.into_canvas_style() {
            self.state.shadow_color = color;
        }
    }

    pub fn filter(&self) -> &str {
        &self.state.filter
    }

    pub fn set_filter(&mut self, value: impl Into<Box<str>>) {
        let value = value.into();
        if PaintFilter::is_valid_css(&value) {
            self.state.filter = value;
        }
    }

    pub fn create_linear_gradient(
        &self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
    ) -> CanvasResult<CanvasGradient> {
        if ![x0, y0, x1, y1].into_iter().all(f32::is_finite) {
            return Err(crate::CanvasError::NonFinite);
        }
        Ok(CanvasGradient::linear(
            (x0, y0),
            (x1, y1),
            self.state.transform,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_radial_gradient(
        &self,
        x0: f32,
        y0: f32,
        radius0: f32,
        x1: f32,
        y1: f32,
        radius1: f32,
    ) -> CanvasResult<CanvasGradient> {
        if ![x0, y0, radius0, x1, y1, radius1]
            .into_iter()
            .all(f32::is_finite)
        {
            return Err(crate::CanvasError::NonFinite);
        }
        if radius0 < 0.0 || radius1 < 0.0 {
            return Err(crate::CanvasError::NegativeRadius);
        }
        Ok(CanvasGradient::radial(
            (x0, y0),
            radius0,
            (x1, y1),
            radius1,
            self.state.transform,
        ))
    }

    pub fn create_conic_gradient(
        &self,
        start_angle: f32,
        x: f32,
        y: f32,
    ) -> CanvasResult<CanvasGradient> {
        if ![start_angle, x, y].into_iter().all(f32::is_finite) {
            return Err(crate::CanvasError::NonFinite);
        }
        Ok(CanvasGradient::conic(
            start_angle,
            (x, y),
            self.state.transform,
        ))
    }

    pub fn create_pattern(
        &self,
        image: &CanvasImage,
        repetition: CanvasPatternRepetition,
    ) -> CanvasPattern {
        CanvasPattern::new(image.clone(), repetition)
    }

    // --- Rectangles -------------------------------------------------------

    pub fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        if ![x, y, width, height].into_iter().all(f32::is_finite) || width == 0.0 || height == 0.0 {
            return;
        }
        let mut path = Path2D::new();
        path.rect_transformed(x, y, width, height, self.state.transform);
        self.brush.reset();
        self.brush.set_anti_alias(false);
        self.brush.set_color(self.settings.blank_color());
        self.brush.set_blend_mode(if self.settings.alpha {
            ohos_drawing_binding::BlendMode::Clear
        } else {
            GlobalCompositeOperation::Copy.to_native_blend_mode()
        });
        self.canvas.attach_brush(&self.brush);
        self.draw_path_at_base(&path.inner);
        self.canvas.detach_brush();
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        if ![x, y, width, height].into_iter().all(f32::is_finite) || width == 0.0 || height == 0.0 {
            return;
        }
        let mut path = Path2D::new();
        path.rect_transformed(x, y, width, height, self.state.transform);
        self.fill_native_path(&path.inner);
    }

    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        if ![x, y, width, height].into_iter().all(f32::is_finite) || (width == 0.0 && height == 0.0)
        {
            return;
        }
        let mut path = Path2D::new();
        path.rect_transformed(x, y, width, height, self.state.transform);
        self.stroke_native_path(&path.inner);
    }

    // --- Current path -----------------------------------------------------

    pub fn begin_path(&mut self) {
        self.current_path.reset();
    }

    pub fn close_path(&mut self) {
        self.current_path.close_path();
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.current_path
            .move_to_transformed(x, y, self.state.transform);
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.current_path
            .line_to_transformed(x, y, self.state.transform);
    }

    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.current_path
            .rect_transformed(x, y, width, height, self.state.transform);
    }

    pub fn quadratic_curve_to(&mut self, control_x: f32, control_y: f32, x: f32, y: f32) {
        self.current_path.quadratic_curve_to_transformed(
            control_x,
            control_y,
            x,
            y,
            self.state.transform,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn bezier_curve_to(
        &mut self,
        control_x1: f32,
        control_y1: f32,
        control_x2: f32,
        control_y2: f32,
        x: f32,
        y: f32,
    ) {
        self.current_path.bezier_curve_to_transformed(
            control_x1,
            control_y1,
            control_x2,
            control_y2,
            x,
            y,
            self.state.transform,
        );
    }

    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) -> CanvasResult<()> {
        self.current_path
            .arc_to_transformed(x1, y1, x2, y2, radius, self.state.transform)
    }

    pub fn round_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radii: impl IntoCanvasRadii,
    ) -> CanvasResult<()> {
        self.current_path.round_rect_transformed(
            x,
            y,
            width,
            height,
            radii.into_canvas_radii(),
            self.state.transform,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn arc(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        counterclockwise: bool,
    ) -> CanvasResult<()> {
        self.current_path.ellipse_transformed(
            x,
            y,
            radius,
            radius,
            0.0,
            start_angle,
            end_angle,
            counterclockwise,
            self.state.transform,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(
        &mut self,
        x: f32,
        y: f32,
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        start_angle: f32,
        end_angle: f32,
        counterclockwise: bool,
    ) -> CanvasResult<()> {
        self.current_path.ellipse_transformed(
            x,
            y,
            radius_x,
            radius_y,
            rotation,
            start_angle,
            end_angle,
            counterclockwise,
            self.state.transform,
        )
    }

    pub fn fill(&mut self) {
        self.fill_with_rule(FillRule::NonZero);
    }

    pub fn fill_with_rule(&mut self, rule: FillRule) {
        let path = self.current_path.clone_with_fill_rule(rule);
        self.fill_native_path(&path);
    }

    pub fn fill_path(&mut self, path: &Path2D) {
        self.fill_path_with_rule(path, FillRule::NonZero);
    }

    pub fn fill_path_with_rule(&mut self, path: &Path2D, rule: FillRule) {
        let path = path.transformed_with_fill_rule(self.state.transform, rule);
        self.fill_native_path(&path);
    }

    pub fn stroke(&mut self) {
        let path = self.current_path.inner.clone_path();
        self.stroke_native_path(&path);
    }

    pub fn stroke_path(&mut self, path: &Path2D) {
        let path = path.transformed(self.state.transform);
        self.stroke_native_path(&path);
    }

    pub fn clip(&mut self) {
        self.clip_with_rule(FillRule::NonZero);
    }

    pub fn clip_with_rule(&mut self, rule: FillRule) {
        let path = self.current_path.clone_with_fill_rule(rule);
        self.clip_native_path_at_base(&path);
    }

    pub fn clip_path(&mut self, path: &Path2D) {
        self.clip_path_with_rule(path, FillRule::NonZero);
    }

    pub fn clip_path_with_rule(&mut self, path: &Path2D, rule: FillRule) {
        let path = path.transformed_with_fill_rule(self.state.transform, rule);
        self.clip_native_path_at_base(&path);
    }

    pub fn is_point_in_path(&self, x: f32, y: f32, rule: FillRule) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        self.current_path.contains(x, y, rule)
    }

    pub fn is_point_in_path2d(&self, path: &Path2D, x: f32, y: f32, rule: FillRule) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        path.transformed_with_fill_rule(self.state.transform, rule)
            .contains(x, y)
    }

    pub fn is_point_in_stroke(&mut self, x: f32, y: f32) -> bool {
        let path = self.current_path.inner.clone_path();
        self.is_point_in_stroke_native(&path, x, y)
    }

    pub fn is_point_in_stroke_path(&mut self, path: &Path2D, x: f32, y: f32) -> bool {
        let path = path.transformed(self.state.transform);
        self.is_point_in_stroke_native(&path, x, y)
    }

    // --- Transform --------------------------------------------------------

    pub fn scale(&mut self, x: f32, y: f32) {
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        self.canvas.scale(x, y);
        self.state.transform = self.state.transform.multiply(DomMatrix2D::scaling(x, y));
    }

    pub fn rotate(&mut self, angle: f32) {
        if !angle.is_finite() {
            return;
        }
        self.canvas.rotate(angle);
        self.state.transform = self.state.transform.multiply(DomMatrix2D::rotation(angle));
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        self.canvas.translate(x, y);
        self.state.transform = self
            .state
            .transform
            .multiply(DomMatrix2D::translation(x, y));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        let matrix = DomMatrix2D::new(a, b, c, d, e, f);
        if !matrix.is_finite() {
            return;
        }
        self.canvas.concat_dom_matrix(matrix);
        self.state.transform = self.state.transform.multiply(matrix);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.set_transform_matrix(DomMatrix2D::new(a, b, c, d, e, f));
    }

    pub fn set_transform_matrix(&mut self, matrix: DomMatrix2D) {
        if !matrix.is_finite() {
            return;
        }
        self.canvas
            .set_dom_transform(matrix, self.device_pixel_ratio);
        self.state.transform = matrix;
    }

    pub fn reset_transform(&mut self) {
        self.canvas.reset_dom_transform(self.device_pixel_ratio);
        self.state.transform = DomMatrix2D::IDENTITY;
    }

    pub const fn get_transform(&self) -> DomMatrix2D {
        self.state.transform
    }

    // --- Text -------------------------------------------------------------

    pub fn set_font(&mut self, font: impl IntoCanvasFont) {
        if let Some(font) = font.into_canvas_font() {
            self.state.font_stretch = font.stretch;
            self.state.font_variant_caps = font.variant_caps;
            self.state.font = font;
        }
    }

    pub fn font(&self) -> &CanvasFont {
        &self.state.font
    }

    pub const fn set_text_align(&mut self, align: CanvasTextAlign) {
        self.state.text_align = align;
    }

    pub const fn text_align(&self) -> CanvasTextAlign {
        self.state.text_align
    }

    pub const fn set_text_baseline(&mut self, baseline: CanvasTextBaseline) {
        self.state.text_baseline = baseline;
    }

    pub const fn text_baseline(&self) -> CanvasTextBaseline {
        self.state.text_baseline
    }

    pub const fn set_direction(&mut self, direction: crate::CanvasTextDirection) {
        self.state.direction = direction;
    }

    pub const fn direction(&self) -> crate::CanvasTextDirection {
        self.state.direction
    }

    pub const fn set_font_kerning(&mut self, value: crate::CanvasFontKerning) {
        self.state.font_kerning = value;
    }

    pub const fn font_kerning(&self) -> crate::CanvasFontKerning {
        self.state.font_kerning
    }

    pub const fn set_font_stretch(&mut self, value: crate::CanvasFontStretch) {
        self.state.font_stretch = value;
        self.state.font.stretch = value;
    }

    pub const fn font_stretch(&self) -> crate::CanvasFontStretch {
        self.state.font_stretch
    }

    pub const fn set_font_variant_caps(&mut self, value: crate::CanvasFontVariantCaps) {
        self.state.font_variant_caps = value;
        self.state.font.variant_caps = value;
    }

    pub const fn font_variant_caps(&self) -> crate::CanvasFontVariantCaps {
        self.state.font_variant_caps
    }

    pub const fn set_text_rendering(&mut self, value: crate::CanvasTextRendering) {
        self.state.text_rendering = value;
    }

    pub const fn text_rendering(&self) -> crate::CanvasTextRendering {
        self.state.text_rendering
    }

    pub fn set_letter_spacing(&mut self, value: impl crate::IntoCanvasTextSpacing) {
        if let Some(spacing) = value.into_canvas_text_spacing(self.state.font.size_px) {
            self.state.letter_spacing = spacing.pixels();
        }
    }

    pub const fn letter_spacing(&self) -> f32 {
        self.state.letter_spacing
    }

    pub fn set_word_spacing(&mut self, value: impl crate::IntoCanvasTextSpacing) {
        if let Some(spacing) = value.into_canvas_text_spacing(self.state.font.size_px) {
            self.state.word_spacing = spacing.pixels();
        }
    }

    pub const fn word_spacing(&self) -> f32 {
        self.state.word_spacing
    }

    pub fn set_lang(&mut self, lang: impl Into<Box<str>>) {
        let lang = lang.into();
        if !lang.contains('\0') {
            self.state.lang = lang;
        }
    }

    pub fn lang(&self) -> &str {
        &self.state.lang
    }

    pub fn fill_text(&mut self, text: &str, x: f32, y: f32) {
        self.fill_text_with_max_width(text, x, y, None);
    }

    pub fn fill_text_max_width(&mut self, text: &str, x: f32, y: f32, max_width: f32) {
        self.fill_text_with_max_width(text, x, y, Some(max_width));
    }

    fn fill_text_with_max_width(&mut self, text: &str, x: f32, y: f32, max_width: Option<f32>) {
        if text.is_empty()
            || !x.is_finite()
            || !y.is_finite()
            || max_width.is_some_and(|width| width.is_nan() || width <= 0.0)
        {
            return;
        }
        let style = self.state.fill_style.clone();
        let sampling = self.sampling_options();
        let (image, shader) = Self::configure_brush(
            &mut self.brush,
            &style,
            self.state.global_alpha,
            self.state.global_composite_operation,
            PaintTransforms::on_current(self.state.transform),
            &sampling,
        );
        let shadow = self.shadow_layer();
        self.brush.set_shadow_layer(shadow.as_ref());
        let filter = PaintFilter::from_css(&self.state.filter);
        if let Some(filter) = filter.as_ref() {
            filter.apply_brush(&mut self.brush);
        }
        let layout = TextLayout::new(text, self.state);
        self.canvas.attach_brush(&self.brush);
        layout.paint(
            self.canvas,
            TextPlacement {
                x,
                y,
                align: self.state.text_align,
                baseline: self.state.text_baseline,
                direction: self.state.direction,
                max_width,
            },
        );
        self.canvas.detach_brush();
        drop(shader);
        drop(image);
        drop(shadow);
        drop(filter);
    }

    pub fn stroke_text(&mut self, text: &str, x: f32, y: f32) {
        self.stroke_text_with_max_width(text, x, y, None);
    }

    pub fn stroke_text_max_width(&mut self, text: &str, x: f32, y: f32, max_width: f32) {
        self.stroke_text_with_max_width(text, x, y, Some(max_width));
    }

    fn stroke_text_with_max_width(&mut self, text: &str, x: f32, y: f32, max_width: Option<f32>) {
        if text.is_empty()
            || !x.is_finite()
            || !y.is_finite()
            || max_width.is_some_and(|width| width.is_nan() || width <= 0.0)
        {
            return;
        }
        let style = self.state.stroke_style.clone();
        let sampling = self.sampling_options();
        let (image, shader) = Self::configure_pen_paint(
            &mut self.pen,
            &style,
            self.state.global_alpha,
            self.state.global_composite_operation,
            PaintTransforms::on_current(self.state.transform),
            &sampling,
        );
        self.pen.set_width(self.state.line_width);
        self.pen.set_canvas_geometry(
            self.state.line_cap,
            self.state.line_join,
            self.state.miter_limit,
        );
        let dash = PathEffect::dash(&self.state.line_dash, self.state.line_dash_offset);
        self.pen.set_path_effect(dash.as_ref());
        let shadow = self.shadow_layer();
        self.pen.set_shadow_layer(shadow.as_ref());
        let filter = PaintFilter::from_css(&self.state.filter);
        if let Some(filter) = filter.as_ref() {
            filter.apply_pen(&mut self.pen);
        }
        let layout = TextLayout::new(text, self.state);
        self.canvas.attach_pen(&self.pen);
        layout.paint(
            self.canvas,
            TextPlacement {
                x,
                y,
                align: self.state.text_align,
                baseline: self.state.text_baseline,
                direction: self.state.direction,
                max_width,
            },
        );
        self.canvas.detach_pen();
        drop(dash);
        drop(shader);
        drop(image);
        drop(shadow);
        drop(filter);
    }

    pub fn measure_text(&self, text: &str) -> CanvasTextMetrics {
        TextLayout::new(text, self.state).metrics(
            self.state.text_align,
            self.state.text_baseline,
            self.state.direction,
        )
    }

    fn fill_native_path(&mut self, path: &ohos_drawing_binding::Path) {
        let style = self.state.fill_style.clone();
        let sampling = self.sampling_options();
        let (image, shader) = Self::configure_brush(
            &mut self.brush,
            &style,
            self.state.global_alpha,
            self.state.global_composite_operation,
            PaintTransforms::at_base(self.state.transform),
            &sampling,
        );
        let shadow = self.shadow_layer();
        self.brush.set_shadow_layer(shadow.as_ref());
        let filter = PaintFilter::from_css(&self.state.filter);
        if let Some(filter) = filter.as_ref() {
            filter.apply_brush(&mut self.brush);
        }
        self.canvas.attach_brush(&self.brush);
        self.draw_path_at_base(path);
        self.canvas.detach_brush();
        drop(shader);
        drop(image);
        drop(shadow);
        drop(filter);
    }

    fn stroke_native_path(&mut self, path: &ohos_drawing_binding::Path) {
        let Some(outline) = self.trace_stroke(path) else {
            return;
        };
        let style = self.state.stroke_style.clone();
        let sampling = self.sampling_options();
        let (image, shader) = Self::configure_brush(
            &mut self.brush,
            &style,
            self.state.global_alpha,
            self.state.global_composite_operation,
            PaintTransforms::at_base(self.state.transform),
            &sampling,
        );
        let shadow = self.shadow_layer();
        self.brush.set_shadow_layer(shadow.as_ref());
        let filter = PaintFilter::from_css(&self.state.filter);
        if let Some(filter) = filter.as_ref() {
            filter.apply_brush(&mut self.brush);
        }
        self.canvas.attach_brush(&self.brush);
        self.draw_path_at_base(&outline);
        self.canvas.detach_brush();
        drop(shader);
        drop(image);
        drop(shadow);
        drop(filter);
    }

    fn draw_path_at_base(&self, path: &Path) {
        let transform = self.canvas.total_matrix();
        self.canvas.reset_dom_transform(self.device_pixel_ratio);
        self.canvas.draw_path(path);
        self.canvas.set_matrix(&transform);
    }

    fn clip_native_path_at_base(&self, path: &Path) {
        let transform = self.canvas.total_matrix();
        self.canvas.reset_dom_transform(self.device_pixel_ratio);
        self.canvas
            .clip_path(path, ohos_drawing_binding::ClipOperation::Intersect, true);
        self.canvas.set_matrix(&transform);
    }

    fn configure_pen_geometry(&mut self) {
        self.pen.set_width(self.state.line_width);
        self.pen.set_canvas_geometry(
            self.state.line_cap,
            self.state.line_join,
            self.state.miter_limit,
        );
    }

    fn create_dash_effect(&self) -> Option<PathEffect> {
        PathEffect::dash(&self.state.line_dash, self.state.line_dash_offset)
    }

    fn trace_stroke(&mut self, path: &Path) -> Option<Path> {
        // Canvas stores its current default path in output coordinates as each
        // command is issued. The stroke outline, however, is traced in the
        // current user coordinate system and is itself transformed by the CTM.
        // Bringing the path through the inverse before tracing preserves both
        // rules, including non-uniform scale and skew.
        let inverse = self.state.transform.inverse()?.to_native_matrix();
        let transform = self.state.transform.to_native_matrix();
        let mut local_path = path.clone_path();
        local_path.transform(&inverse);

        self.pen.reset();
        self.configure_pen_geometry();
        let dash = self.create_dash_effect();
        self.pen.set_path_effect(dash.as_ref());
        let mut outline = Path::new();
        self.pen
            .fill_path(&local_path, &mut outline, None, None)
            .then(|| {
                outline.transform(&transform);
                outline
            })
    }

    fn is_point_in_stroke_native(&mut self, path: &Path, x: f32, y: f32) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        self.trace_stroke(path)
            .is_some_and(|outline| outline.contains(x, y))
    }

    pub fn draw_image(&mut self, image: &CanvasImage, dx: f32, dy: f32) {
        self.draw_image_rect(
            image,
            0.0,
            0.0,
            image.width() as f32,
            image.height() as f32,
            dx,
            dy,
            image.width() as f32,
            image.height() as f32,
        );
    }

    pub fn draw_image_scaled(
        &mut self,
        image: &CanvasImage,
        dx: f32,
        dy: f32,
        destination_width: f32,
        destination_height: f32,
    ) {
        self.draw_image_rect(
            image,
            0.0,
            0.0,
            image.width() as f32,
            image.height() as f32,
            dx,
            dy,
            destination_width,
            destination_height,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_image_rect(
        &mut self,
        image: &CanvasImage,
        source_x: f32,
        source_y: f32,
        source_width: f32,
        source_height: f32,
        destination_x: f32,
        destination_y: f32,
        destination_width: f32,
        destination_height: f32,
    ) {
        if ![
            source_x,
            source_y,
            source_width,
            source_height,
            destination_x,
            destination_y,
            destination_width,
            destination_height,
        ]
        .into_iter()
        .all(f32::is_finite)
            || source_width == 0.0
            || source_height == 0.0
            || destination_width == 0.0
            || destination_height == 0.0
        {
            return;
        }
        let Some(source) = Self::normalized_rect(source_x, source_y, source_width, source_height)
        else {
            return;
        };
        let Some(destination) = Self::normalized_rect(
            destination_x,
            destination_y,
            destination_width,
            destination_height,
        ) else {
            return;
        };
        let Some((source, destination)) = Self::clip_image_rects(
            &source,
            &destination,
            image.width() as f32,
            image.height() as f32,
        ) else {
            return;
        };
        let sampling = self.sampling_options();
        self.brush.reset();
        self.brush.set_anti_alias(true);
        self.brush
            .set_alpha((self.state.global_alpha * 255.0).round() as u8);
        self.brush
            .set_blend_mode(self.state.global_composite_operation.to_native_blend_mode());
        let shadow = self.shadow_layer();
        self.brush.set_shadow_layer(shadow.as_ref());
        let filter = PaintFilter::from_css(&self.state.filter);
        if let Some(filter) = filter.as_ref() {
            filter.apply_brush(&mut self.brush);
        }
        self.canvas.attach_brush(&self.brush);
        self.canvas
            .draw_bitmap_rect(image.bitmap(), Some(&source), &destination, &sampling);
        self.canvas.detach_brush();
    }

    pub fn create_image_data(&self, width: i32, height: i32) -> CanvasResult<ImageData> {
        self.create_image_data_with_settings(width, height, ImageDataSettings::default())
    }

    pub fn create_image_data_with_settings(
        &self,
        width: i32,
        height: i32,
        settings: ImageDataSettings,
    ) -> CanvasResult<ImageData> {
        let (width, height) = ImageData::normalized_dimensions(width, height)?;
        ImageData::new_with_settings(width, height, settings.resolved(self.settings.color_space))
    }

    pub fn create_image_data_like(&self, source: &ImageData) -> CanvasResult<ImageData> {
        ImageData::new_with_settings(
            source.width(),
            source.height(),
            ImageDataSettings {
                color_space: Some(source.color_space()),
                pixel_format: source.pixel_format(),
            },
        )
    }

    pub fn get_image_data(
        &self,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
    ) -> CanvasResult<ImageData> {
        self.get_image_data_with_settings(
            source_x,
            source_y,
            source_width,
            source_height,
            ImageDataSettings::default(),
        )
    }

    pub fn get_image_data_with_settings(
        &self,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        settings: ImageDataSettings,
    ) -> CanvasResult<ImageData> {
        let (source_x, source_width) = ImageData::normalized_axis(source_x, source_width)?;
        let (source_y, source_height) = ImageData::normalized_axis(source_y, source_height)?;
        let output_settings = settings.resolved(self.settings.color_space);
        let mut result =
            ImageData::new_with_settings(source_width, source_height, output_settings)?;
        let ratio = self.device_pixel_ratio;
        let Some(bitmap) = self.canvas.bitmap() else {
            return Ok(result);
        };
        let pixels = bitmap.pixels();
        let bitmap_width = i64::from(bitmap.width());
        let bitmap_height = i64::from(bitmap.height());
        for y in 0..source_height {
            for x in 0..source_width {
                let physical_x = (((source_x + i64::from(x)) as f64 * f64::from(ratio)
                    + f64::from(ratio) * 0.5)
                    .floor()) as i64;
                let physical_y = (((source_y + i64::from(y)) as f64 * f64::from(ratio)
                    + f64::from(ratio) * 0.5)
                    .floor()) as i64;
                if physical_x < 0
                    || physical_y < 0
                    || physical_x >= bitmap_width
                    || physical_y >= bitmap_height
                {
                    continue;
                }
                let source = usize::try_from((physical_y * bitmap_width + physical_x) * 4)
                    .expect("canvas bitmap offset fits usize");
                let destination =
                    usize::try_from((u64::from(y) * u64::from(source_width) + u64::from(x)) * 4)
                        .expect("validated ImageData offset fits usize");
                let mut rgba = ImageData::unpremultiply_pixel(&pixels[source..source + 4]);
                if !self.settings.alpha {
                    rgba[3] = 255;
                }
                let mut rgba = rgba.map(|channel| f32::from(channel) / 255.0);
                ColorSpaceTransform::convert(
                    &mut rgba,
                    self.settings.color_space,
                    output_settings.color_space.unwrap_or_default(),
                );
                result.write_pixel(destination, rgba);
            }
        }
        Ok(result)
    }

    pub fn put_image_data(
        &mut self,
        image_data: &ImageData,
        destination_x: i32,
        destination_y: i32,
    ) {
        self.put_image_data_dirty(
            image_data,
            destination_x,
            destination_y,
            0,
            0,
            i32::try_from(image_data.width()).unwrap_or(i32::MAX),
            i32::try_from(image_data.height()).unwrap_or(i32::MAX),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn put_image_data_dirty(
        &mut self,
        image_data: &ImageData,
        destination_x: i32,
        destination_y: i32,
        dirty_x: i32,
        dirty_y: i32,
        dirty_width: i32,
        dirty_height: i32,
    ) {
        if dirty_width == 0 || dirty_height == 0 {
            return;
        }
        let Ok((dirty_x, dirty_width)) = ImageData::normalized_axis(dirty_x, dirty_width) else {
            return;
        };
        let Ok((dirty_y, dirty_height)) = ImageData::normalized_axis(dirty_y, dirty_height) else {
            return;
        };
        let ratio = self.device_pixel_ratio;
        let Some(bitmap) = self.canvas.bitmap_mut() else {
            return;
        };
        let bitmap_width = i64::from(bitmap.width());
        let bitmap_height = i64::from(bitmap.height());
        let source_width = i64::from(image_data.width());
        let source_height = i64::from(image_data.height());
        let pixels = bitmap.pixels_mut();
        let dirty_right = dirty_x
            .saturating_add(i64::from(dirty_width))
            .min(source_width);
        let dirty_bottom = dirty_y
            .saturating_add(i64::from(dirty_height))
            .min(source_height);
        for source_y in dirty_y.max(0)..dirty_bottom {
            for source_x in dirty_x.max(0)..dirty_right {
                let logical_x = i64::from(destination_x).saturating_add(source_x);
                let logical_y = i64::from(destination_y).saturating_add(source_y);
                let physical_left = (logical_x as f64 * f64::from(ratio)).floor() as i64;
                let physical_top = (logical_y as f64 * f64::from(ratio)).floor() as i64;
                let physical_right = ((logical_x + 1) as f64 * f64::from(ratio)).ceil() as i64;
                let physical_bottom = ((logical_y + 1) as f64 * f64::from(ratio)).ceil() as i64;
                let source = usize::try_from((source_y * source_width + source_x) * 4)
                    .expect("validated ImageData offset fits usize");
                let mut rgba = image_data.read_pixel(source);
                ColorSpaceTransform::convert(
                    &mut rgba,
                    image_data.color_space(),
                    self.settings.color_space,
                );
                let premultiplied = ImageData::premultiply_pixel(rgba, !self.settings.alpha);
                for physical_y in physical_top.max(0)..physical_bottom.min(bitmap_height) {
                    for physical_x in physical_left.max(0)..physical_right.min(bitmap_width) {
                        let destination =
                            usize::try_from((physical_y * bitmap_width + physical_x) * 4)
                                .expect("canvas bitmap offset fits usize");
                        pixels[destination..destination + 4].copy_from_slice(&premultiplied);
                    }
                }
            }
        }
    }
}

impl<'canvas> CanvasRenderingContext2D<'canvas> {
    fn configure_brush<'style>(
        brush: &mut Brush,
        style: &'style CanvasStyle,
        global_alpha: f32,
        composite: GlobalCompositeOperation,
        transforms: PaintTransforms,
        sampling: &SamplingOptions,
    ) -> (
        Option<ohos_drawing_binding::Image<'style>>,
        Option<ohos_drawing_binding::ShaderEffect>,
    ) {
        brush.reset();
        brush.set_anti_alias(true);
        brush.set_blend_mode(composite.to_native_blend_mode());
        match style {
            CanvasStyle::Color(color) => {
                brush.set_color(color.with_global_alpha(global_alpha));
                (None, None)
            }
            CanvasStyle::Gradient(gradient) => {
                let shader = gradient.shader(global_alpha, transforms.native);
                brush.set_color(if shader.is_some() {
                    0xFFFF_FFFF
                } else {
                    0x0000_0000
                });
                brush.set_shader_effect(shader.as_ref());
                (None, shader)
            }
            CanvasStyle::Pattern(pattern) => {
                let (image, shader) = pattern
                    .shader(sampling, transforms.native, transforms.current)
                    .map_or((None, None), |(image, shader)| (Some(image), Some(shader)));
                brush.set_color(if shader.is_some() {
                    0xFFFF_FFFF
                } else {
                    0x0000_0000
                });
                brush.set_alpha((global_alpha * 255.0).round() as u8);
                brush.set_shader_effect(shader.as_ref());
                (image, shader)
            }
        }
    }

    fn configure_pen_paint<'style>(
        pen: &mut Pen,
        style: &'style CanvasStyle,
        global_alpha: f32,
        composite: GlobalCompositeOperation,
        transforms: PaintTransforms,
        sampling: &SamplingOptions,
    ) -> (
        Option<ohos_drawing_binding::Image<'style>>,
        Option<ohos_drawing_binding::ShaderEffect>,
    ) {
        pen.reset();
        pen.set_anti_alias(true);
        pen.set_blend_mode(composite.to_native_blend_mode());
        match style {
            CanvasStyle::Color(color) => {
                pen.set_color(color.with_global_alpha(global_alpha));
                (None, None)
            }
            CanvasStyle::Gradient(gradient) => {
                let shader = gradient.shader(global_alpha, transforms.native);
                pen.set_color(if shader.is_some() {
                    0xFFFF_FFFF
                } else {
                    0x0000_0000
                });
                pen.set_shader_effect(shader.as_ref());
                (None, shader)
            }
            CanvasStyle::Pattern(pattern) => {
                let (image, shader) = pattern
                    .shader(sampling, transforms.native, transforms.current)
                    .map_or((None, None), |(image, shader)| (Some(image), Some(shader)));
                pen.set_color(if shader.is_some() {
                    0xFFFF_FFFF
                } else {
                    0x0000_0000
                });
                pen.set_alpha((global_alpha * 255.0).round() as u8);
                pen.set_shader_effect(shader.as_ref());
                (image, shader)
            }
        }
    }

    fn shadow_layer(&self) -> Option<ShadowLayer> {
        if self.state.shadow_color.alpha() == 0 {
            return None;
        }
        ShadowLayer::new(
            self.state.shadow_blur * 0.5,
            self.state.shadow_offset_x,
            self.state.shadow_offset_y,
            self.state
                .shadow_color
                .with_global_alpha(self.state.global_alpha),
        )
    }

    fn sampling_options(&self) -> SamplingOptions {
        use ohos_drawing_binding::{FilterMode, MipmapMode};
        if !self.state.image_smoothing_enabled {
            SamplingOptions::new(FilterMode::Nearest, MipmapMode::None)
        } else {
            SamplingOptions::new(
                FilterMode::Linear,
                match self.state.image_smoothing_quality {
                    CanvasImageSmoothingQuality::Low => MipmapMode::None,
                    CanvasImageSmoothingQuality::Medium => MipmapMode::Nearest,
                    CanvasImageSmoothingQuality::High => MipmapMode::Linear,
                },
            )
        }
    }

    fn normalized_rect(x: f32, y: f32, width: f32, height: f32) -> Option<Rect> {
        if ![x, y, width, height].into_iter().all(f32::is_finite) {
            return None;
        }
        let x2 = x + width;
        let y2 = y + height;
        let (left, right) = if x <= x2 { (x, x2) } else { (x2, x) };
        let (top, bottom) = if y <= y2 { (y, y2) } else { (y2, y) };
        Some(Rect::new(left, top, right, bottom))
    }

    fn clip_image_rects(
        source: &Rect,
        destination: &Rect,
        image_width: f32,
        image_height: f32,
    ) -> Option<(Rect, Rect)> {
        let source_width = source.width();
        let source_height = source.height();
        if source_width <= 0.0 || source_height <= 0.0 {
            return None;
        }
        let left = source.left().clamp(0.0, image_width);
        let top = source.top().clamp(0.0, image_height);
        let right = source.right().clamp(0.0, image_width);
        let bottom = source.bottom().clamp(0.0, image_height);
        if left >= right || top >= bottom {
            return None;
        }
        let destination_left =
            destination.left() + (left - source.left()) / source_width * destination.width();
        let destination_top =
            destination.top() + (top - source.top()) / source_height * destination.height();
        let destination_right =
            destination.left() + (right - source.left()) / source_width * destination.width();
        let destination_bottom =
            destination.top() + (bottom - source.top()) / source_height * destination.height();
        Some((
            Rect::new(left, top, right, bottom),
            Rect::new(
                destination_left,
                destination_top,
                destination_right,
                destination_bottom,
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_negative_pixel_rect_axes_without_overflow() {
        assert_eq!(ImageData::normalized_axis(10, -4), Ok((6, 4)));
        assert_eq!(
            ImageData::normalized_axis(i32::MIN, i32::MIN),
            Ok((i64::from(i32::MIN) * 2, 2_147_483_648))
        );
        assert_eq!(
            ImageData::normalized_axis(10, 0),
            Err(crate::CanvasError::InvalidImageData)
        );
    }

    #[test]
    fn opaque_image_data_write_composites_alpha_against_black() {
        assert_eq!(
            ImageData::premultiply_pixel([1.0, 0.5, 0.0, 0.5], true),
            [128, 64, 0, 255]
        );
        assert_eq!(
            ImageData::premultiply_pixel([1.0, 0.5, 0.0, 0.5], false),
            [128, 64, 0, 128]
        );
    }

    #[test]
    fn clips_draw_image_source_and_destination_proportionally() {
        let source = Rect::new(-10.0, 10.0, 30.0, 70.0);
        let destination = Rect::new(100.0, 200.0, 300.0, 500.0);
        let (source, destination) =
            CanvasRenderingContext2D::clip_image_rects(&source, &destination, 20.0, 40.0)
                .expect("partly visible source");
        assert_eq!(
            (source.left(), source.top(), source.right(), source.bottom()),
            (0.0, 10.0, 20.0, 40.0)
        );
        assert_eq!(
            (
                destination.left(),
                destination.top(),
                destination.right(),
                destination.bottom()
            ),
            (150.0, 200.0, 250.0, 350.0)
        );
    }
}
