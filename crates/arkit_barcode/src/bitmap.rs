//! Encoded barcode bitmap and export helpers.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use arkit_arkui::ArkImageSource;

use crate::error::{BarcodeError, BarcodeResult};
use crate::format::BarcodeFormat;
use crate::render::{png, svg};

/// Device-independent encoding result: a dark/light module grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarcodeBitmap {
    width: u32,
    height: u32,
    /// Row-major modules; `true` means dark.
    modules: Arc<[bool]>,
    format: BarcodeFormat,
    contents: Arc<str>,
    dark: u32,
    light: u32,
    fingerprint: Arc<str>,
}

impl BarcodeBitmap {
    pub(crate) fn new(
        width: u32,
        height: u32,
        modules: Vec<bool>,
        format: BarcodeFormat,
        contents: impl Into<Arc<str>>,
        dark: u32,
        light: u32,
        fingerprint: impl Into<Arc<str>>,
    ) -> Self {
        debug_assert_eq!(modules.len(), width as usize * height as usize);
        Self {
            width,
            height,
            modules: Arc::from(modules),
            format,
            contents: contents.into(),
            dark,
            light,
            fingerprint: fingerprint.into(),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> BarcodeFormat {
        self.format
    }

    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn dark(&self) -> u32 {
        self.dark
    }

    pub fn light(&self) -> u32 {
        self.light
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub fn modules(&self) -> &[bool] {
        &self.modules
    }

    pub fn is_dark(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.modules[(y * self.width + x) as usize]
    }

    /// SVG document (crispEdges). Prefer this for on-screen display.
    pub fn to_svg(&self) -> String {
        svg::render(self)
    }

    /// [`ArkImageSource`] suitable for `image { src: any_value(source) }`.
    pub fn to_ark_image_source(&self) -> ArkImageSource {
        let svg = self.to_svg();
        ArkImageSource::svg(self.fingerprint.to_string(), svg, self.width, self.height)
    }

    /// Packed RGBA8888 pixels (length = width * height * 4).
    pub fn to_rgba8888(&self) -> Vec<u8> {
        let mut pixels = Vec::with_capacity(self.modules.len() * 4);
        for &dark in self.modules.iter() {
            let color = if dark { self.dark } else { self.light };
            pixels.push(((color >> 16) & 0xFF) as u8); // R
            pixels.push(((color >> 8) & 0xFF) as u8); // G
            pixels.push((color & 0xFF) as u8); // B
            pixels.push(((color >> 24) & 0xFF) as u8); // A
        }
        pixels
    }

    pub fn to_png_bytes(&self) -> BarcodeResult<Vec<u8>> {
        png::encode_rgba(self.width, self.height, &self.to_rgba8888())
    }

    /// Raw base64 of PNG bytes (no `data:` prefix).
    pub fn to_base64_png(&self) -> BarcodeResult<String> {
        let bytes = self.to_png_bytes()?;
        Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            bytes,
        ))
    }

    pub fn to_data_uri_png(&self) -> BarcodeResult<String> {
        Ok(format!("data:image/png;base64,{}", self.to_base64_png()?))
    }

    /// Write PNG to `path` (parent directories must exist). Returns the path.
    pub fn write_png(&self, path: impl AsRef<Path>) -> BarcodeResult<PathBuf> {
        let path = path.as_ref();
        let bytes = self.to_png_bytes()?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|source| {
                    BarcodeError::io("create_dir_all", parent.to_path_buf(), source)
                })?;
            }
        }
        let tmp = path.with_extension("png.tmp");
        std::fs::write(&tmp, &bytes)
            .map_err(|source| BarcodeError::io("write", tmp.clone(), source))?;
        std::fs::rename(&tmp, path).map_err(|source| {
            let _ = std::fs::remove_file(&tmp);
            BarcodeError::io("rename", path.to_path_buf(), source)
        })?;
        Ok(path.to_path_buf())
    }
}
