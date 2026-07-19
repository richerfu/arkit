use crate::context::CanvasSurface;
use crate::{
    CanvasError, CanvasImage, CanvasImageEncodeOptions, CanvasImageFormat,
    CanvasRenderingContext2D, CanvasRenderingContext2DSettings, CanvasResult,
};
use base64::Engine as _;

/// An owned Canvas 2D backing store that does not require an ArkUI node.
///
/// This is the native equivalent of the web platform's `OffscreenCanvas` and
/// is suitable for prerendering sprites, generated images, and animation
/// frames before they are copied into an on-screen [`crate::Canvas`].
pub struct OffscreenCanvas {
    width: u32,
    height: u32,
    settings: CanvasRenderingContext2DSettings,
    surface: CanvasSurface,
}

impl OffscreenCanvas {
    pub fn new(width: u32, height: u32) -> CanvasResult<Self> {
        Self::new_with_settings(width, height, CanvasRenderingContext2DSettings::default())
    }

    pub fn new_with_settings(
        width: u32,
        height: u32,
        settings: CanvasRenderingContext2DSettings,
    ) -> CanvasResult<Self> {
        let surface = Self::create_surface(width, height, settings)?;
        Ok(Self {
            width,
            height,
            settings,
            surface,
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Resizes and clears the backing store, matching assignment to the web
    /// `width` or `height` attribute.
    pub fn resize(&mut self, width: u32, height: u32) -> CanvasResult<()> {
        self.surface = Self::create_surface(width, height, self.settings)?;
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub fn get_context_2d(&mut self) -> CanvasRenderingContext2D<'_> {
        self.surface.context()
    }

    /// Copies the current backing store into an immutable image source.
    pub fn snapshot(&self) -> CanvasImage {
        self.surface
            .snapshot()
            .expect("offscreen canvas always owns a bitmap backing store")
    }

    /// W3C-style image-bitmap transfer. Native drawing surfaces cannot move
    /// their bitmap out of an active canvas, so this performs a bounded copy.
    pub fn transfer_to_image_bitmap(&self) -> CanvasImage {
        self.snapshot()
    }

    /// W3C-style encoded output corresponding to `convertToBlob()`.
    pub fn convert_to_blob(
        &self,
        format: CanvasImageFormat,
        options: CanvasImageEncodeOptions,
    ) -> CanvasResult<Vec<u8>> {
        self.snapshot().encode(format, options)
    }

    /// Produces a portable SVG document containing the current raster backing
    /// store. OHOS native drawing does not expose a vector recording surface,
    /// so this preserves exact pixels by embedding a PNG instead of pretending
    /// that text and filtered paths can be reconstructed as vectors.
    pub fn convert_to_svg(&self) -> CanvasResult<Vec<u8>> {
        let png =
            self.convert_to_blob(CanvasImageFormat::Png, CanvasImageEncodeOptions::default())?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(png);
        Ok(format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\"><image width=\"100%\" height=\"100%\" href=\"data:image/png;base64,{}\"/></svg>",
            self.width, self.height, self.width, self.height, encoded
        )
        .into_bytes())
    }

    fn create_surface(
        width: u32,
        height: u32,
        settings: CanvasRenderingContext2DSettings,
    ) -> CanvasResult<CanvasSurface> {
        if width == 0 || height == 0 {
            return Err(CanvasError::InvalidDimensions);
        }
        Ok(CanvasSurface::new(
            width as f32,
            height as f32,
            width,
            height,
            1.0,
            settings,
        ))
    }
}

impl std::fmt::Debug for OffscreenCanvas {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OffscreenCanvas")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("settings", &self.settings)
            .finish_non_exhaustive()
    }
}
