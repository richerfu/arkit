//! Native chart export: render into a CPU bitmap, convert to PixelMap, and encode.

use std::collections::BTreeSet;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

use ohos_drawing_binding::Canvas;
use ohos_image_native_binding::{
    ImagePacker, ImageString, PackingOptions, PixelFormat, PixelMap, PixelMapInitializationOptions,
};
use ohos_native_drawing_sys::{
    OH_Drawing_AlphaFormat_ALPHA_FORMAT_PREMUL, OH_Drawing_Bitmap, OH_Drawing_BitmapBuild,
    OH_Drawing_BitmapCreate, OH_Drawing_BitmapDestroy, OH_Drawing_BitmapFormat,
    OH_Drawing_BitmapGetPixels, OH_Drawing_CanvasBind,
    OH_Drawing_ColorFormat_COLOR_FORMAT_BGRA_8888,
};

use crate::model::{ChartEvent, ChartOption};
use crate::render::{draw_option, ZoomWindow};

pub(crate) struct ExportContext<'a> {
    pub(crate) option: &'a ChartOption,
    pub(crate) selected: Option<&'a ChartEvent>,
    pub(crate) hidden_series: &'a BTreeSet<usize>,
    pub(crate) zoom_windows: &'a [ZoomWindow],
    pub(crate) selected_items: &'a BTreeSet<(usize, usize)>,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) device_pixel_ratio: f32,
}

pub(crate) fn save_chart_image(context: ExportContext<'_>) -> Result<PathBuf, String> {
    let feature = context
        .option
        .extra
        .get("toolbox")
        .and_then(serde_json::Value::as_object)
        .and_then(|toolbox| toolbox.get("feature"))
        .and_then(serde_json::Value::as_object)
        .and_then(|features| features.get("saveAsImage"))
        .and_then(serde_json::Value::as_object);
    let image_type = feature
        .and_then(|feature| feature.get("type"))
        .and_then(serde_json::Value::as_str)
        .filter(|value| matches!(*value, "png" | "jpeg" | "jpg"))
        .unwrap_or("png");
    let extension = if image_type == "jpg" {
        "jpeg"
    } else {
        image_type
    };
    let pixel_ratio = feature
        .and_then(|feature| feature.get("pixelRatio"))
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32)
        .unwrap_or(context.device_pixel_ratio)
        .clamp(0.5, 4.0);
    let width = (context.width.max(1.0) * pixel_ratio).round() as u32;
    let height = (context.height.max(1.0) * pixel_ratio).round() as u32;
    let byte_len = width as usize * height as usize * 4;
    let mut option = context.option.clone();
    let excludes_toolbox = feature
        .and_then(|feature| feature.get("excludeComponents"))
        .and_then(serde_json::Value::as_array)
        .map(|values| values.iter().any(|value| value.as_str() == Some("toolbox")))
        .unwrap_or(true);
    if excludes_toolbox {
        option.extra.remove("toolbox");
    }
    if let Some(background) = feature
        .and_then(|feature| feature.get("backgroundColor"))
        .and_then(crate::parser::parse_color)
    {
        option.visual_style.background_color = background;
    }
    crate::state::apply_states(&mut option, context.selected, context.selected_items);

    let bitmap = NativeBitmap::new(width, height)?;
    {
        let canvas = Canvas::new();
        unsafe { OH_Drawing_CanvasBind(canvas.as_ptr(), bitmap.raw) };
        canvas.save();
        canvas.scale(pixel_ratio, pixel_ratio);
        draw_option(
            &option,
            context.selected,
            context.hidden_series,
            context.zoom_windows,
            context.selected_items,
            Some(&canvas),
            context.width,
            context.height,
        );
        canvas.restore();
    }
    let pixels = bitmap.copy_pixels(byte_len)?;
    let mut pixels = pixels;
    let mut initialization = PixelMapInitializationOptions::new().map_err(image_error)?;
    initialization.set_width(width).map_err(image_error)?;
    initialization.set_height(height).map_err(image_error)?;
    initialization
        .set_pixel_format(PixelFormat::Bgra8888)
        .map_err(image_error)?;
    initialization
        .set_src_pixel_format(PixelFormat::Bgra8888)
        .map_err(image_error)?;
    initialization
        .set_row_stride((width * 4) as i32)
        .map_err(image_error)?;
    initialization.set_alpha_type(2).map_err(image_error)?;
    let pixelmap = PixelMap::create(&mut pixels, &mut initialization).map_err(image_error)?;

    let path = export_path(context.option, feature, extension);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create export directory: {error}"))?;
    }
    let file = std::fs::File::create(&path)
        .map_err(|error| format!("create {}: {error}", path.display()))?;
    let mut packing = PackingOptions::new().map_err(image_error)?;
    let mut mime = ImageString::from_str(if extension == "png" {
        "image/png"
    } else {
        "image/jpeg"
    })
    .map_err(image_error)?;
    packing.set_mime_type(&mut mime).map_err(image_error)?;
    packing.set_quality(92).map_err(image_error)?;
    let mut packer = ImagePacker::new().map_err(image_error)?;
    packer
        .pack_to_file_from_pixelmap(&mut packing, &pixelmap, file.as_raw_fd())
        .map_err(image_error)?;
    Ok(path)
}

fn export_path(
    option: &ChartOption,
    feature: Option<&serde_json::Map<String, serde_json::Value>>,
    extension: &str,
) -> PathBuf {
    if let Some(path) = feature
        .and_then(|feature| feature.get("path"))
        .and_then(serde_json::Value::as_str)
    {
        return PathBuf::from(path);
    }
    let name = feature
        .and_then(|feature| feature.get("name"))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| option.title.as_ref().map(|title| title.text.clone()))
        .unwrap_or_else(|| String::from("echarts"));
    let name = sanitize_filename(&name);
    Path::new("/data/storage/el2/base/files").join(format!("{name}.{extension}"))
}

fn sanitize_filename(value: &str) -> String {
    let value = value
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() || matches!(value, '-' | '_') {
                value
            } else {
                '_'
            }
        })
        .collect::<String>();
    let value = value.trim_matches('_');
    if value.is_empty() {
        String::from("echarts")
    } else {
        value.to_string()
    }
}

fn image_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

struct NativeBitmap {
    raw: *mut OH_Drawing_Bitmap,
}

impl NativeBitmap {
    fn new(width: u32, height: u32) -> Result<Self, String> {
        let raw = unsafe { OH_Drawing_BitmapCreate() };
        if raw.is_null() {
            return Err(String::from("OH_Drawing_BitmapCreate returned null"));
        }
        let format = OH_Drawing_BitmapFormat {
            colorFormat: OH_Drawing_ColorFormat_COLOR_FORMAT_BGRA_8888,
            alphaFormat: OH_Drawing_AlphaFormat_ALPHA_FORMAT_PREMUL,
        };
        unsafe { OH_Drawing_BitmapBuild(raw, width, height, &format) };
        Ok(Self { raw })
    }

    fn copy_pixels(&self, byte_len: usize) -> Result<Vec<u8>, String> {
        let pixels = unsafe { OH_Drawing_BitmapGetPixels(self.raw) }.cast::<u8>();
        if pixels.is_null() {
            return Err(String::from("OH_Drawing_BitmapGetPixels returned null"));
        }
        Ok(unsafe { std::slice::from_raw_parts(pixels, byte_len) }.to_vec())
    }
}

impl Drop for NativeBitmap {
    fn drop(&mut self) {
        unsafe { OH_Drawing_BitmapDestroy(self.raw) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_filename_is_safe_and_stable() {
        assert_eq!(sanitize_filename("Revenue / 2026"), "Revenue___2026");
        assert_eq!(sanitize_filename("***"), "echarts");
    }
}
