//! Barcode symbologies supported by the encoder.

/// Symbology used for generation.
///
/// Variants mirror the camera scan format set so applications can map
/// scan results to generators without a third shared crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BarcodeFormat {
    #[default]
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

impl BarcodeFormat {
    /// Every format this crate can encode (stable order for UI pickers).
    pub const ALL: &'static [Self] = &[
        Self::QrCode,
        Self::DataMatrix,
        Self::Aztec,
        Self::Pdf417,
        Self::Code128,
        Self::Code93,
        Self::Code39,
        Self::Codabar,
        Self::Ean13,
        Self::Ean8,
        Self::UpcA,
        Self::UpcE,
        Self::Itf,
    ];

    /// Human-readable label for UI.
    pub fn label(self) -> &'static str {
        match self {
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
        }
    }

    /// Whether the format is typically rendered as a square matrix.
    pub fn is_matrix(self) -> bool {
        matches!(
            self,
            Self::QrCode | Self::DataMatrix | Self::Aztec | Self::Pdf417
        )
    }

    pub(crate) fn to_rxing(self) -> rxing::BarcodeFormat {
        match self {
            Self::QrCode => rxing::BarcodeFormat::QR_CODE,
            Self::DataMatrix => rxing::BarcodeFormat::DATA_MATRIX,
            Self::Aztec => rxing::BarcodeFormat::AZTEC,
            Self::Pdf417 => rxing::BarcodeFormat::PDF_417,
            Self::Code128 => rxing::BarcodeFormat::CODE_128,
            Self::Code93 => rxing::BarcodeFormat::CODE_93,
            Self::Code39 => rxing::BarcodeFormat::CODE_39,
            Self::Codabar => rxing::BarcodeFormat::CODABAR,
            Self::Ean13 => rxing::BarcodeFormat::EAN_13,
            Self::Ean8 => rxing::BarcodeFormat::EAN_8,
            Self::UpcA => rxing::BarcodeFormat::UPC_A,
            Self::UpcE => rxing::BarcodeFormat::UPC_E,
            Self::Itf => rxing::BarcodeFormat::ITF,
        }
    }
}

impl fmt::Display for BarcodeFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

use std::fmt;

/// QR error-correction level (ignored for non-QR formats).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QrEcLevel {
    L,
    #[default]
    M,
    Q,
    H,
}

impl QrEcLevel {
    pub(crate) fn as_hint(self) -> &'static str {
        match self {
            Self::L => "L",
            Self::M => "M",
            Self::Q => "Q",
            Self::H => "H",
        }
    }
}
