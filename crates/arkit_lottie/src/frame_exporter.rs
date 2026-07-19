use std::ops::Range;

use thorvg::{ColorSpace, Thorvg};

use crate::renderer::LottieRenderer;
use crate::{
    LottieAlignment, LottieComposition, LottieError, LottieFit, LottieResult, LottieSource,
};

/// Controls deterministic, off-screen Lottie frame rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LottieFrameRenderOptions {
    /// Output size. `None` uses the rounded composition size.
    pub size: Option<[u32; 2]>,
    pub quality: u8,
    pub fit: LottieFit,
    pub alignment: LottieAlignment,
}

impl Default for LottieFrameRenderOptions {
    fn default() -> Self {
        Self {
            size: None,
            quality: 100,
            fit: LottieFit::Contain,
            alignment: LottieAlignment::Center,
        }
    }
}

/// A borrowed, tightly packed RGBA frame produced by [`LottieFrameRenderer`].
#[derive(Clone, Copy, Debug)]
pub struct LottieRenderedFrame<'pixels> {
    pub index: u32,
    pub timestamp_micros: u64,
    pub duration_micros: u64,
    pub width: u32,
    pub height: u32,
    pub rgba: &'pixels [u8],
}

/// High-throughput off-screen Lottie renderer for Canvas/image/video export.
///
/// One ThorVG engine, animation, target buffer, and transform are reused for
/// the complete requested range. The callback's pixel slice is valid only for
/// that callback; copy it only when frames must be retained concurrently.
#[derive(Clone, Debug)]
pub struct LottieFrameRenderer {
    source: LottieSource,
    options: LottieFrameRenderOptions,
}

impl LottieFrameRenderer {
    pub fn new(source: LottieSource) -> Self {
        Self {
            source,
            options: LottieFrameRenderOptions::default(),
        }
    }

    pub fn with_options(mut self, options: LottieFrameRenderOptions) -> Self {
        self.options = options;
        self
    }

    pub fn composition(&self) -> LottieResult<LottieComposition> {
        self.render_range(0..0, |_| {})
    }

    pub fn render_all(
        &self,
        on_frame: impl FnMut(LottieRenderedFrame<'_>),
    ) -> LottieResult<LottieComposition> {
        self.render_range(0..u32::MAX, on_frame)
    }

    pub fn render_range(
        &self,
        range: Range<u32>,
        mut on_frame: impl FnMut(LottieRenderedFrame<'_>),
    ) -> LottieResult<LottieComposition> {
        if self.source.inline_bytes().is_none() {
            return Err(LottieError::invalid_source(
                "LottieFrameRenderer::render_range",
                "network sources must be resolved to inline bytes before frame export",
            ));
        }
        if self
            .options
            .size
            .is_some_and(|[width, height]| width == 0 || height == 0)
        {
            return Err(LottieError::invalid_configuration(
                "LottieFrameRenderer::render_range",
                "output width and height must be non-zero",
            ));
        }

        let engine = Thorvg::init(render_thread_count())
            .map_err(|error| LottieError::render("Thorvg::init", error.to_string()))?;
        let mut renderer = LottieRenderer::new(&engine)?;
        let composition = renderer.load(
            &self.source,
            self.options.quality,
            self.options.fit,
            self.options.alignment,
        )?;
        let [width, height] = self.options.size.unwrap_or([
            composition.width.round().max(1.0) as u32,
            composition.height.round().max(1.0) as u32,
        ]);
        let frame_count = composition.frames.round().max(1.0) as u32;
        let start = range.start.min(frame_count);
        let end = range.end.min(frame_count);
        if start >= end {
            return Ok(composition);
        }
        let pixel_count = usize::try_from(u64::from(width) * u64::from(height)).map_err(|_| {
            LottieError::invalid_configuration(
                "LottieFrameRenderer::render_range",
                "output pixel count exceeds addressable memory",
            )
        })?;
        let mut pixels = Vec::new();
        pixels.try_reserve_exact(pixel_count).map_err(|_| {
            LottieError::render(
                "LottieFrameRenderer::render_range",
                "could not allocate the output frame",
            )
        })?;
        pixels.resize(pixel_count, 0_u32);
        let frame_duration = (1_000_000.0 / composition.frames_per_second).round() as u64;

        for index in start..end {
            renderer.set_frame(index as f32)?;
            renderer.render_to_pixels(&mut pixels, width, width, height, ColorSpace::ABGR8888)?;
            // SAFETY: every u32 target pixel is exactly four initialized bytes;
            // the borrowed byte view cannot outlive this callback invocation.
            let rgba = unsafe {
                std::slice::from_raw_parts(pixels.as_ptr().cast::<u8>(), pixels.len() * 4)
            };
            on_frame(LottieRenderedFrame {
                index,
                timestamp_micros: u64::from(index).saturating_mul(frame_duration),
                duration_micros: frame_duration,
                width,
                height,
                rgba,
            });
        }
        Ok(composition)
    }
}

fn render_thread_count() -> u32 {
    std::thread::available_parallelism()
        .map_or(1, usize::from)
        .saturating_sub(1)
        .clamp(1, 4) as u32
}
