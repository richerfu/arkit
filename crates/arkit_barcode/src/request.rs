//! Encoding request and shared option structs.

use crate::format::{BarcodeFormat, QrEcLevel};

/// Maximum edge length accepted by the encoder (pixels).
pub const MAX_BARCODE_EDGE: u32 = 4_096;

/// Default quiet-zone / margin modules passed to the writer.
pub const DEFAULT_MARGIN: u32 = 2;

/// Pure encoding parameters (no UI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarcodeRequest {
    pub contents: String,
    pub format: BarcodeFormat,
    pub width: u32,
    pub height: u32,
    pub margin: u32,
    pub dark: u32,
    pub light: u32,
    pub qr_ec_level: Option<QrEcLevel>,
}

impl BarcodeRequest {
    /// Square QR helper.
    pub fn qr(contents: impl Into<String>, size: u32) -> Self {
        Self {
            contents: contents.into(),
            format: BarcodeFormat::QrCode,
            width: size,
            height: size,
            margin: DEFAULT_MARGIN,
            dark: 0xFF00_0000,
            light: 0xFFFF_FFFF,
            qr_ec_level: Some(QrEcLevel::M),
        }
    }

    /// Code 128 strip helper.
    pub fn code128(contents: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            contents: contents.into(),
            format: BarcodeFormat::Code128,
            width,
            height,
            margin: DEFAULT_MARGIN,
            dark: 0xFF00_0000,
            light: 0xFFFF_FFFF,
            qr_ec_level: None,
        }
    }

    /// Stable cache key for image sources and export memoization.
    pub fn fingerprint(&self) -> String {
        let ec = self.qr_ec_level.map(|level| level.as_hint()).unwrap_or("-");
        format!(
            "barcode:{}:{}:{}x{}:m{}:{:08x}:{:08x}:ec{}",
            self.format.label(),
            simple_hash(&self.contents),
            self.width,
            self.height,
            self.margin,
            self.dark,
            self.light,
            ec,
        )
    }
}

/// Copyable options used by hooks and the declarative component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarcodeOptions {
    pub format: BarcodeFormat,
    pub width: u32,
    pub height: u32,
    pub margin: u32,
    pub dark: u32,
    pub light: u32,
    pub qr_ec_level: Option<QrEcLevel>,
}

impl Default for BarcodeOptions {
    fn default() -> Self {
        Self::qr(256)
    }
}

impl BarcodeOptions {
    /// Square QR options at `size` CSS/layout pixels (also encode pixels).
    pub fn qr(size: u32) -> Self {
        Self {
            format: BarcodeFormat::QrCode,
            width: size.max(1),
            height: size.max(1),
            margin: DEFAULT_MARGIN,
            dark: 0xFF00_0000,
            light: 0xFFFF_FFFF,
            qr_ec_level: Some(QrEcLevel::M),
        }
    }

    pub fn code128(width: u32, height: u32) -> Self {
        Self {
            format: BarcodeFormat::Code128,
            width: width.max(1),
            height: height.max(1),
            margin: DEFAULT_MARGIN,
            dark: 0xFF00_0000,
            light: 0xFFFF_FFFF,
            qr_ec_level: None,
        }
    }

    pub fn with_format(mut self, format: BarcodeFormat) -> Self {
        self.format = format;
        if format.is_matrix() {
            let edge = self.width.max(self.height);
            self.width = edge;
            self.height = edge;
            if self.qr_ec_level.is_none() && matches!(format, BarcodeFormat::QrCode) {
                self.qr_ec_level = Some(QrEcLevel::M);
            }
        }
        self
    }

    pub fn with_colors(mut self, dark: u32, light: u32) -> Self {
        self.dark = dark;
        self.light = light;
        self
    }

    pub fn with_margin(mut self, margin: u32) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_qr_ec_level(mut self, level: QrEcLevel) -> Self {
        self.qr_ec_level = Some(level);
        self
    }

    pub fn to_request(&self, contents: impl Into<String>) -> BarcodeRequest {
        BarcodeRequest {
            contents: contents.into(),
            format: self.format,
            width: self.width,
            height: self.height,
            margin: self.margin,
            dark: self.dark,
            light: self.light,
            qr_ec_level: self.qr_ec_level,
        }
    }
}

fn simple_hash(input: &str) -> u64 {
    // FNV-1a 64 — stable, dependency-free fingerprint for cache keys.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}
