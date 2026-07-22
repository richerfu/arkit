//! PNG encoding for barcode RGBA buffers.

use crate::error::{BarcodeError, BarcodeResult};

pub(crate) fn encode_rgba(width: u32, height: u32, rgba: &[u8]) -> BarcodeResult<Vec<u8>> {
    let expected = width as usize * height as usize * 4;
    if rgba.len() != expected {
        return Err(BarcodeError::render_failed(format!(
            "RGBA buffer length {} does not match {width}x{height}",
            rgba.len()
        )));
    }

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut encoder = png::Encoder::new(&mut cursor, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|error| BarcodeError::render_failed(error.to_string()))?;
        writer
            .write_image_data(rgba)
            .map_err(|error| BarcodeError::render_failed(error.to_string()))?;
    }
    Ok(cursor.into_inner())
}
