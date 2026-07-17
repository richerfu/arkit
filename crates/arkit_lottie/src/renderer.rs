use ohos_native_window_binding::{NativeBufferFormat, NativeWindow};
use thorvg::{ColorSpace, EngineOption, LottieAnimation, Matrix, Paint, Picture, SwCanvas, Thorvg};

use crate::{
    LottieAlignment, LottieComposition, LottieError, LottieErrorKind, LottieFit, LottieResult,
    LottieSource,
};

pub(crate) struct LottieRenderer<'engine> {
    engine: &'engine Thorvg,
    canvas: SwCanvas<'engine>,
    animation: Option<LottieAnimation<'engine>>,
    composition: Option<LottieComposition>,
    fit: LottieFit,
    alignment: LottieAlignment,
    target_size: Option<(u32, u32)>,
    // A stable target keeps the canvas from retaining a pointer to a native
    // window mapping after that mapping is flushed and unmapped.
    fallback_target: Box<[u32; 1]>,
}

impl<'engine> LottieRenderer<'engine> {
    pub(crate) fn new(engine: &'engine Thorvg) -> LottieResult<Self> {
        // NativeWindow rotates through multiple buffers. Dirty-region rendering
        // is therefore incorrect because a newly dequeued buffer does not
        // necessarily contain the previous frame.
        let mut canvas = engine
            .sw_canvas(EngineOption::None)
            .map_err(|error| map_thorvg("Thorvg::sw_canvas", error))?;
        let mut fallback_target = Box::new([0_u32; 1]);
        // SAFETY: the boxed single-pixel target has a stable address and is
        // retained by this renderer for longer than the canvas.
        unsafe {
            canvas.set_target(
                fallback_target.as_mut_slice(),
                1,
                1,
                1,
                ColorSpace::ABGR8888,
            )
        }
        .map_err(|error| map_thorvg("SwCanvas::set_target", error))?;
        Ok(Self {
            engine,
            canvas,
            animation: None,
            composition: None,
            fit: LottieFit::default(),
            alignment: LottieAlignment::default(),
            target_size: None,
            fallback_target,
        })
    }

    pub(crate) fn load(
        &mut self,
        source: &LottieSource,
        quality: u8,
        fit: LottieFit,
        alignment: LottieAlignment,
    ) -> LottieResult<LottieComposition> {
        let Some(bytes) = source.inline_bytes() else {
            return Err(LottieError::invalid_source(
                "LottieRenderer::load",
                "network sources must be downloaded before rendering",
            ));
        };
        if bytes.is_empty() {
            return Err(LottieError::invalid_source(
                "LottieRenderer::load",
                "the Lottie JSON buffer is empty",
            ));
        }
        if source.key().is_empty() {
            return Err(LottieError::invalid_source(
                "LottieRenderer::load",
                "the Lottie source key is empty",
            ));
        }
        self.unload()?;

        let mut animation = self
            .engine
            .lottie_animation()
            .map_err(|error| map_thorvg("Thorvg::lottie_animation", error))?;
        animation
            .load_data(bytes)
            .map_err(|error| map_source_error(source, error))?;
        animation
            .set_quality(quality.min(100))
            .map_err(|error| map_thorvg("LottieAnimation::set_quality", error))?;

        let (width, height) = animation
            .picture()
            .size()
            .map_err(|error| map_thorvg("Picture::size", error))?;
        let frames = animation
            .total_frame()
            .map_err(|error| map_thorvg("Animation::total_frame", error))?;
        let duration_seconds = animation
            .duration()
            .map_err(|error| map_thorvg("Animation::duration", error))?;
        let composition = LottieComposition {
            width,
            height,
            frames,
            duration_seconds,
            frames_per_second: frames / duration_seconds,
        };
        if !composition.is_valid() {
            return Err(LottieError::invalid_source(
                "LottieRenderer::load",
                format!(
                    "composition '{}' has invalid dimensions or timing metadata",
                    source.key()
                ),
            ));
        }

        // ThorVG's animation owns its Picture while Canvas also needs a
        // reference to that same object. The safe wrapper intentionally does
        // not expose ref-counting, so this is the single contained FFI bridge:
        // increment once, wrap that reference, and transfer it to Canvas.
        add_animation_picture(&mut self.canvas, &animation)?;

        self.animation = Some(animation);
        self.composition = Some(composition);
        self.fit = fit;
        self.alignment = alignment;
        self.target_size = None;
        Ok(composition)
    }

    pub(crate) fn unload(&mut self) -> LottieResult<()> {
        self.canvas
            .clear()
            .map_err(|error| map_thorvg("SwCanvas::clear", error))?;
        self.animation.take();
        self.composition = None;
        self.target_size = None;
        Ok(())
    }

    pub(crate) fn configure_layout(&mut self, fit: LottieFit, alignment: LottieAlignment) {
        if self.fit != fit || self.alignment != alignment {
            self.fit = fit;
            self.alignment = alignment;
            self.target_size = None;
        }
    }

    pub(crate) fn set_quality(&mut self, quality: u8) -> LottieResult<()> {
        let Some(animation) = self.animation.as_mut() else {
            return Ok(());
        };
        animation
            .set_quality(quality.min(100))
            .map_err(|error| map_thorvg("LottieAnimation::set_quality", error))
    }

    pub(crate) fn set_frame(&mut self, frame: f32) -> LottieResult<()> {
        let Some(animation) = self.animation.as_mut() else {
            return Ok(());
        };
        match animation.set_frame(frame) {
            Ok(()) | Err(thorvg::Error::InsufficientCondition) => Ok(()),
            Err(error) => Err(map_thorvg("LottieAnimation::set_frame", error)),
        }
    }

    pub(crate) fn render(&mut self, window: &NativeWindow) -> LottieResult<()> {
        if self.animation.is_none() {
            return Ok(());
        }
        let mut buffer = window.request_buffer(None).map_err(|error| {
            LottieError::new(
                LottieErrorKind::SurfaceUnavailable,
                "NativeWindow::request_buffer",
                format!("{error:?}"),
            )
        })?;
        let width = u32::try_from(buffer.width()).map_err(|_| {
            LottieError::render("LottieRenderer::render", "surface width exceeds u32")
        })?;
        let height = u32::try_from(buffer.height()).map_err(|_| {
            LottieError::render("LottieRenderer::render", "surface height exceeds u32")
        })?;
        let stride = u32::try_from(buffer.stride()).map_err(|_| {
            LottieError::render("LottieRenderer::render", "surface stride exceeds u32")
        })?;
        if width == 0 || height == 0 || stride < width {
            return Err(LottieError::render(
                "LottieRenderer::render",
                "surface returned invalid buffer geometry",
            ));
        }
        let color_space = native_color_space(buffer.format())?;
        self.update_transform(width, height)?;

        let pixel_len = usize::try_from(u64::from(stride) * u64::from(height)).map_err(|_| {
            LottieError::render("LottieRenderer::render", "surface pixel count overflowed")
        })?;
        // SAFETY: NativeWindowBuffer owns a writable mmap whose documented
        // length is `stride * height * bytes_per_pixel`. Supported formats are
        // exactly four bytes per pixel, mmap is suitably aligned for u32, and
        // the slice is used only until draw+sync complete below.
        let pixels = unsafe { std::slice::from_raw_parts_mut(buffer.bits().cast(), pixel_len) };
        // SAFETY: `pixels` remains alive and exclusively borrowed until the
        // canvas is synchronized and retargeted to the stable fallback.
        unsafe {
            self.canvas
                .set_target(pixels, stride, width, height, color_space)
        }
        .map_err(|error| map_thorvg("SwCanvas::set_target", error))?;

        let render_result = self
            .canvas
            .render()
            .map_err(|error| map_thorvg("SwCanvas::render", error));
        // SAFETY: the fallback is boxed, remains stable for the renderer's
        // lifetime, and retargeting happens before NativeWindowBuffer unmaps.
        let reset_result = unsafe {
            self.canvas.set_target(
                self.fallback_target.as_mut_slice(),
                1,
                1,
                1,
                ColorSpace::ABGR8888,
            )
        }
        .map_err(|error| map_thorvg("SwCanvas::reset_target", error));
        render_result?;
        reset_result
    }

    fn update_transform(&mut self, width: u32, height: u32) -> LottieResult<()> {
        if self.target_size == Some((width, height)) {
            return Ok(());
        }
        let composition = self.composition.ok_or_else(|| {
            LottieError::render(
                "LottieRenderer::update_transform",
                "composition is not loaded",
            )
        })?;
        let target_width = width as f32;
        let target_height = height as f32;
        let scale_x = target_width / composition.width;
        let scale_y = target_height / composition.height;
        let (scale_x, scale_y) = match self.fit {
            LottieFit::Contain => {
                let scale = scale_x.min(scale_y);
                (scale, scale)
            }
            LottieFit::Cover => {
                let scale = scale_x.max(scale_y);
                (scale, scale)
            }
            LottieFit::Fill => (scale_x, scale_y),
            LottieFit::None => (1.0, 1.0),
        };
        let content_width = composition.width * scale_x;
        let content_height = composition.height * scale_y;
        let (align_x, align_y) = self.alignment.factors();
        let offset_x = (target_width - content_width) * align_x;
        let offset_y = (target_height - content_height) * align_y;
        let transform = Matrix {
            e11: scale_x,
            e12: 0.0,
            e13: offset_x,
            e21: 0.0,
            e22: scale_y,
            e23: offset_y,
            e31: 0.0,
            e32: 0.0,
            e33: 1.0,
        };
        self.animation
            .as_mut()
            .expect("animation checked before rendering")
            .picture_mut()
            .set_transform(&transform)
            .map_err(|error| map_thorvg("Picture::set_transform", error))?;
        self.target_size = Some((width, height));
        Ok(())
    }
}

impl Drop for LottieRenderer<'_> {
    fn drop(&mut self) {
        // Release Canvas' reference before the owning Animation is destroyed.
        let _ = self.canvas.clear();
    }
}

fn add_animation_picture(
    canvas: &mut SwCanvas<'_>,
    animation: &LottieAnimation<'_>,
) -> LottieResult<()> {
    let raw = animation.picture().raw();
    // SAFETY: `raw` is a live Picture owned by `animation`. Incrementing its
    // ThorVG reference count gives the wrapper below one owned reference.
    let references = unsafe { thorvg_sys::tvg_paint_ref(raw) };
    if references < 2 {
        // SAFETY: even an unexpected reference count still represents the
        // temporary reference requested above, so balance it before failing.
        unsafe {
            thorvg_sys::tvg_paint_unref(raw, false);
        }
        return Err(LottieError::render(
            "tvg_paint_ref",
            "failed to share the animation picture with the canvas",
        ));
    }
    // SAFETY: the preceding `tvg_paint_ref` transferred one valid owned
    // Picture reference to this wrapper, which Canvas consumes immediately.
    let picture = unsafe { <Picture<'_> as Paint>::from_raw_paint(raw) };
    let result = canvas.add(picture);
    // SAFETY: Canvas increments the Picture reference on successful add; on
    // failure it did not take ownership. In either case this balances only the
    // temporary reference created above, leaving Animation's reference intact.
    unsafe {
        thorvg_sys::tvg_paint_unref(raw, false);
    }
    result.map_err(|error| map_thorvg("SwCanvas::add", error))
}

fn native_color_space(format: NativeBufferFormat) -> LottieResult<ColorSpace> {
    match format {
        NativeBufferFormat::RGBA_8888 | NativeBufferFormat::RGBX_8888 => Ok(ColorSpace::ABGR8888),
        NativeBufferFormat::BGRA_8888 | NativeBufferFormat::BGRX_8888 => Ok(ColorSpace::ARGB8888),
        unsupported => Err(LottieError::new(
            LottieErrorKind::UnsupportedPixelFormat,
            "LottieRenderer::render",
            format!("native window format {unsupported:?} is not a 32-bit RGBA/BGRA buffer"),
        )),
    }
}

fn map_source_error(source: &LottieSource, error: thorvg::Error) -> LottieError {
    LottieError::invalid_source(
        "LottieAnimation::load_data",
        format!("could not parse composition '{}': {error}", source.key()),
    )
}

fn map_thorvg(operation: &'static str, error: thorvg::Error) -> LottieError {
    LottieError::render(operation, error.to_string())
}
