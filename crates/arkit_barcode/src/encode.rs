//! Pure barcode encoding via rxing `MultiFormatWriter`.

use rxing::{EncodeHints, MultiFormatWriter, Writer};

use crate::bitmap::BarcodeBitmap;
use crate::error::{BarcodeError, BarcodeResult};
use crate::request::{BarcodeRequest, MAX_BARCODE_EDGE};

/// Encode `request` into a [`BarcodeBitmap`].
///
/// Does not touch the filesystem or UI. Safe to call from tests and workers.
pub fn encode_barcode(request: &BarcodeRequest) -> BarcodeResult<BarcodeBitmap> {
    let contents = request.contents.trim();
    if contents.is_empty() {
        return Err(BarcodeError::empty_contents());
    }
    if request.width == 0 || request.height == 0 {
        return Err(BarcodeError::invalid_dimensions(
            "barcode width and height must be >= 1",
        ));
    }
    if request.width > MAX_BARCODE_EDGE || request.height > MAX_BARCODE_EDGE {
        return Err(BarcodeError::invalid_dimensions(format!(
            "barcode dimensions {}x{} exceed max edge {MAX_BARCODE_EDGE}",
            request.width, request.height
        )));
    }

    let mut hints = EncodeHints {
        Margin: Some(request.margin.to_string()),
        ..EncodeHints::default()
    };
    if let Some(level) = request.qr_ec_level {
        if matches!(request.format, crate::format::BarcodeFormat::QrCode) {
            hints.ErrorCorrection = Some(level.as_hint().to_string());
        }
    }

    let writer = MultiFormatWriter;
    let matrix = writer
        .encode_with_hints(
            contents,
            &request.format.to_rxing(),
            request.width as i32,
            request.height as i32,
            &hints,
        )
        .map_err(|error| BarcodeError::encode_failed(error.to_string()))?;

    let width = matrix.width();
    let height = matrix.height();
    if width == 0 || height == 0 {
        return Err(BarcodeError::encode_failed(
            "encoder returned an empty bit matrix",
        ));
    }

    let mut modules = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            modules.push(matrix.get(x, y));
        }
    }

    Ok(BarcodeBitmap::new(
        width,
        height,
        modules,
        request.format,
        contents,
        request.dark,
        request.light,
        request.fingerprint(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{BarcodeFormat, QrEcLevel};
    use crate::request::BarcodeOptions;

    #[test]
    fn encodes_qr_code() {
        let bitmap =
            encode_barcode(&BarcodeRequest::qr("https://example.com", 256)).expect("qr encode");
        assert!(bitmap.width() >= 21);
        assert_eq!(bitmap.width(), bitmap.height());
        assert!(bitmap.modules().iter().any(|dark| *dark));
        assert!(bitmap.modules().iter().any(|dark| !*dark));
        let svg = bitmap.to_svg();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("rect"));
        let png = bitmap.to_png_bytes().expect("png");
        assert!(png.starts_with(b"\x89PNG"));
        let b64 = bitmap.to_base64_png().expect("b64");
        assert!(!b64.is_empty());
    }

    #[test]
    fn encodes_code128() {
        let bitmap =
            encode_barcode(&BarcodeRequest::code128("ARKIT-12345", 320, 96)).expect("code128");
        assert!(bitmap.width() > bitmap.height());
        assert_eq!(bitmap.format(), BarcodeFormat::Code128);
    }

    #[test]
    fn empty_contents_error() {
        let err = encode_barcode(&BarcodeRequest::qr("   ", 128)).unwrap_err();
        assert_eq!(err.kind, crate::error::BarcodeErrorKind::EmptyContents);
    }

    #[test]
    fn options_to_request_roundtrip() {
        let options = BarcodeOptions::qr(200)
            .with_qr_ec_level(QrEcLevel::H)
            .with_colors(0xFF11_2233, 0xFFEE_EEEE);
        let request = options.to_request("payload");
        assert_eq!(request.width, 200);
        assert_eq!(request.qr_ec_level, Some(QrEcLevel::H));
        let bitmap = encode_barcode(&request).expect("encode");
        assert!(bitmap.fingerprint().contains("ecH"));
    }

    #[test]
    fn oversized_dimensions_rejected() {
        let mut request = BarcodeRequest::qr("x", 64);
        request.width = MAX_BARCODE_EDGE + 1;
        let err = encode_barcode(&request).unwrap_err();
        assert_eq!(err.kind, crate::error::BarcodeErrorKind::InvalidDimensions);
    }

    #[test]
    fn all_public_formats_encode_sample_payloads() {
        // Keep payloads aligned with examples/barcode sample_payload.
        let samples: &[(BarcodeFormat, &str, u32, u32)] = &[
            (BarcodeFormat::QrCode, "https://example.com/arkit", 128, 128),
            (BarcodeFormat::DataMatrix, "ARKIT-DM-001", 128, 128),
            (BarcodeFormat::Aztec, "ARKIT-AZTEC", 128, 128),
            (BarcodeFormat::Pdf417, "PDF417 sample", 240, 96),
            (BarcodeFormat::Code128, "ARKIT-2026", 240, 80),
            (BarcodeFormat::Code93, "ARKIT93", 240, 80),
            (BarcodeFormat::Code39, "ARKIT-39", 240, 80),
            (BarcodeFormat::Codabar, "A123456A", 240, 80),
            (BarcodeFormat::Ean13, "5901234123457", 240, 96),
            (BarcodeFormat::Ean8, "96385074", 200, 80),
            (BarcodeFormat::UpcA, "042100005264", 240, 96),
            (BarcodeFormat::UpcE, "01234565", 200, 80),
            (BarcodeFormat::Itf, "123456789012", 240, 80),
        ];
        assert_eq!(samples.len(), BarcodeFormat::ALL.len());
        for (format, payload, width, height) in samples {
            let request = BarcodeRequest {
                contents: (*payload).to_string(),
                format: *format,
                width: *width,
                height: *height,
                margin: 2,
                dark: 0xFF00_0000,
                light: 0xFFFF_FFFF,
                qr_ec_level: if matches!(format, BarcodeFormat::QrCode) {
                    Some(QrEcLevel::M)
                } else {
                    None
                },
            };
            encode_barcode(&request).unwrap_or_else(|error| {
                panic!(
                    "format {} payload {payload:?} failed: {}",
                    format.label(),
                    error.message()
                )
            });
        }
    }
}
