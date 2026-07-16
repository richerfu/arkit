use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use crate::{CameraFrame, CameraSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraScanFormat {
    QrCode,
    DataMatrix,
    Aztec,
    Pdf417,
    Code128,
    Code93,
    Code39,
    Codabar,
    Ean13,
    Ean8,
    UpcA,
    UpcE,
    Itf,
}

impl std::fmt::Display for CameraScanFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::QrCode => "QR Code",
            Self::DataMatrix => "Data Matrix",
            Self::Aztec => "Aztec",
            Self::Pdf417 => "PDF417",
            Self::Code128 => "Code 128",
            Self::Code93 => "Code 93",
            Self::Code39 => "Code 39",
            Self::Codabar => "Codabar",
            Self::Ean13 => "EAN-13",
            Self::Ean8 => "EAN-8",
            Self::UpcA => "UPC-A",
            Self::UpcE => "UPC-E",
            Self::Itf => "ITF",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraScanRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl CameraScanRegion {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn crop(self, size: CameraSize) -> Option<(u32, u32, u32, u32)> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || !self.width.is_finite()
            || !self.height.is_finite()
            || self.width <= 0.0
            || self.height <= 0.0
        {
            return None;
        }
        let x = self.x.clamp(0.0, 1.0);
        let y = self.y.clamp(0.0, 1.0);
        let right = (self.x + self.width).clamp(x, 1.0);
        let bottom = (self.y + self.height).clamp(y, 1.0);
        let left = (x * size.width as f32).floor() as u32;
        let top = (y * size.height as f32).floor() as u32;
        let width = ((right - x) * size.width as f32).floor().max(1.0) as u32;
        let height = ((bottom - y) * size.height as f32).floor().max(1.0) as u32;
        Some((
            left,
            top,
            width.min(size.width.saturating_sub(left)),
            height.min(size.height.saturating_sub(top)),
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraScanConfiguration {
    pub frame_size: Option<CameraSize>,
    pub max_frames_per_second: u8,
    pub formats: Arc<[CameraScanFormat]>,
    pub region: Option<CameraScanRegion>,
    pub try_harder: bool,
    pub continuous: bool,
    pub duplicate_timeout: Duration,
}

impl Default for CameraScanConfiguration {
    fn default() -> Self {
        Self {
            frame_size: None,
            max_frames_per_second: 10,
            formats: vec![
                CameraScanFormat::QrCode,
                CameraScanFormat::DataMatrix,
                CameraScanFormat::Code128,
                CameraScanFormat::Ean13,
            ]
            .into(),
            region: Some(CameraScanRegion::new(0.12, 0.2, 0.76, 0.6)),
            try_harder: false,
            continuous: false,
            duplicate_timeout: Duration::from_millis(1_500),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CameraScanResult {
    pub text: Arc<str>,
    pub raw_bytes: Arc<[u8]>,
    pub format: CameraScanFormat,
    pub timestamp_ns: i64,
}

impl std::fmt::Debug for CameraScanResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CameraScanResult")
            .field("text", &self.text)
            .field("raw_byte_len", &self.raw_bytes.len())
            .field("format", &self.format)
            .field("timestamp_ns", &self.timestamp_ns)
            .finish()
    }
}

pub(crate) fn decode_frame(
    frame: CameraFrame,
    configuration: &CameraScanConfiguration,
) -> Option<CameraScanResult> {
    let (luma, width, height) = crop_frame(&frame, configuration.region)?;
    let mut hints = rxing::DecodeHints {
        PossibleFormats: Some(
            configuration
                .formats
                .iter()
                .copied()
                .map(native_format)
                .collect::<HashSet<_>>(),
        ),
        TryHarder: Some(configuration.try_harder),
        AlsoInverted: Some(true),
        ..rxing::DecodeHints::default()
    };
    let decoded =
        rxing::helpers::detect_in_luma_with_hints(luma, width, height, None, &mut hints).ok()?;
    Some(CameraScanResult {
        text: Arc::from(decoded.getText()),
        raw_bytes: Arc::from(decoded.getRawBytes()),
        format: scan_format(*decoded.getBarcodeFormat())?,
        timestamp_ns: frame.timestamp_ns,
    })
}

fn crop_frame(
    frame: &CameraFrame,
    region: Option<CameraScanRegion>,
) -> Option<(Vec<u8>, u32, u32)> {
    let Some((left, top, width, height)) = region.and_then(|region| region.crop(frame.size)) else {
        return Some((frame.luma().to_vec(), frame.size.width, frame.size.height));
    };
    if width == 0 || height == 0 {
        return None;
    }
    let source_width = frame.size.width as usize;
    let mut cropped = Vec::with_capacity(width as usize * height as usize);
    for y in top..top + height {
        let start = y as usize * source_width + left as usize;
        let end = start + width as usize;
        cropped.extend_from_slice(frame.luma().get(start..end)?);
    }
    Some((cropped, width, height))
}

fn native_format(format: CameraScanFormat) -> rxing::BarcodeFormat {
    match format {
        CameraScanFormat::QrCode => rxing::BarcodeFormat::QR_CODE,
        CameraScanFormat::DataMatrix => rxing::BarcodeFormat::DATA_MATRIX,
        CameraScanFormat::Aztec => rxing::BarcodeFormat::AZTEC,
        CameraScanFormat::Pdf417 => rxing::BarcodeFormat::PDF_417,
        CameraScanFormat::Code128 => rxing::BarcodeFormat::CODE_128,
        CameraScanFormat::Code93 => rxing::BarcodeFormat::CODE_93,
        CameraScanFormat::Code39 => rxing::BarcodeFormat::CODE_39,
        CameraScanFormat::Codabar => rxing::BarcodeFormat::CODABAR,
        CameraScanFormat::Ean13 => rxing::BarcodeFormat::EAN_13,
        CameraScanFormat::Ean8 => rxing::BarcodeFormat::EAN_8,
        CameraScanFormat::UpcA => rxing::BarcodeFormat::UPC_A,
        CameraScanFormat::UpcE => rxing::BarcodeFormat::UPC_E,
        CameraScanFormat::Itf => rxing::BarcodeFormat::ITF,
    }
}

fn scan_format(format: rxing::BarcodeFormat) -> Option<CameraScanFormat> {
    match format {
        rxing::BarcodeFormat::QR_CODE
        | rxing::BarcodeFormat::MICRO_QR_CODE
        | rxing::BarcodeFormat::RECTANGULAR_MICRO_QR_CODE => Some(CameraScanFormat::QrCode),
        rxing::BarcodeFormat::DATA_MATRIX => Some(CameraScanFormat::DataMatrix),
        rxing::BarcodeFormat::AZTEC => Some(CameraScanFormat::Aztec),
        rxing::BarcodeFormat::PDF_417 => Some(CameraScanFormat::Pdf417),
        rxing::BarcodeFormat::CODE_128 => Some(CameraScanFormat::Code128),
        rxing::BarcodeFormat::CODE_93 => Some(CameraScanFormat::Code93),
        rxing::BarcodeFormat::CODE_39 => Some(CameraScanFormat::Code39),
        rxing::BarcodeFormat::CODABAR => Some(CameraScanFormat::Codabar),
        rxing::BarcodeFormat::EAN_13 => Some(CameraScanFormat::Ean13),
        rxing::BarcodeFormat::EAN_8 => Some(CameraScanFormat::Ean8),
        rxing::BarcodeFormat::UPC_A => Some(CameraScanFormat::UpcA),
        rxing::BarcodeFormat::UPC_E => Some(CameraScanFormat::UpcE),
        rxing::BarcodeFormat::ITF => Some(CameraScanFormat::Itf),
        _ => None,
    }
}
