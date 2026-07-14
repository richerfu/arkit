//! Native chart export: render into a CPU bitmap, convert to PixelMap, and encode.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
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

const MAX_EXPORT_EDGE: u32 = 8_192;
const MAX_EXPORT_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug)]
pub(crate) enum ChartExportError {
    InvalidDimensions {
        width: f32,
        height: f32,
    },
    DimensionsTooLarge {
        width: u32,
        height: u32,
    },
    BufferSizeOverflow,
    Native(&'static str),
    Image(String),
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for ChartExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions { width, height } => {
                write!(formatter, "invalid chart dimensions {width}x{height}")
            }
            Self::DimensionsTooLarge { width, height } => write!(
                formatter,
                "export dimensions {width}x{height} exceed the supported limit"
            ),
            Self::BufferSizeOverflow => formatter.write_str("export pixel buffer is too large"),
            Self::Native(message) => formatter.write_str(message),
            Self::Image(message) => formatter.write_str(message),
            Self::Io {
                operation,
                path,
                source,
            } => write!(formatter, "{operation} {}: {source}", path.display()),
        }
    }
}

impl Error for ChartExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

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

pub(crate) fn save_chart_image(context: ExportContext<'_>) -> Result<PathBuf, ChartExportError> {
    if !context.width.is_finite()
        || !context.height.is_finite()
        || context.width <= 0.0
        || context.height <= 0.0
    {
        return Err(ChartExportError::InvalidDimensions {
            width: context.width,
            height: context.height,
        });
    }
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
    let requested_pixel_ratio = feature
        .and_then(|feature| feature.get("pixelRatio"))
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32)
        .unwrap_or(context.device_pixel_ratio);
    let pixel_ratio = if requested_pixel_ratio.is_finite() {
        requested_pixel_ratio.clamp(0.5, 4.0)
    } else {
        1.0
    };
    let width = (context.width.max(1.0) * pixel_ratio).round() as u32;
    let height = (context.height.max(1.0) * pixel_ratio).round() as u32;
    if width > MAX_EXPORT_EDGE || height > MAX_EXPORT_EDGE {
        return Err(ChartExportError::DimensionsTooLarge { width, height });
    }
    let byte_len = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .filter(|bytes| *bytes <= MAX_EXPORT_BYTES)
        .ok_or(ChartExportError::BufferSizeOverflow)?;
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
        // SAFETY: both handles are live for this scope; `bitmap` outlives the
        // borrowed canvas binding and is not accessed concurrently.
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
        .set_row_stride(
            width
                .checked_mul(4)
                .and_then(|value| i32::try_from(value).ok())
                .ok_or(ChartExportError::BufferSizeOverflow)?,
        )
        .map_err(image_error)?;
    initialization.set_alpha_type(2).map_err(image_error)?;
    let pixelmap = PixelMap::create(&mut pixels, &mut initialization).map_err(image_error)?;

    let path = export_path(context.option, feature, extension);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ChartExportError::Io {
            operation: "create export directory",
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let file = std::fs::File::create(&path).map_err(|source| ChartExportError::Io {
        operation: "create export file",
        path: path.clone(),
        source,
    })?;
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

fn image_error(error: impl std::fmt::Display) -> ChartExportError {
    ChartExportError::Image(error.to_string())
}

struct NativeBitmap {
    raw: *mut OH_Drawing_Bitmap,
}

impl NativeBitmap {
    fn new(width: u32, height: u32) -> Result<Self, ChartExportError> {
        // SAFETY: creates a fresh native bitmap handle with no aliases.
        let raw = unsafe { OH_Drawing_BitmapCreate() };
        if raw.is_null() {
            return Err(ChartExportError::Native(
                "OH_Drawing_BitmapCreate returned null",
            ));
        }
        let format = OH_Drawing_BitmapFormat {
            colorFormat: OH_Drawing_ColorFormat_COLOR_FORMAT_BGRA_8888,
            alphaFormat: OH_Drawing_AlphaFormat_ALPHA_FORMAT_PREMUL,
        };
        // SAFETY: `raw` is the live handle created above and `format` remains
        // valid for the duration of the synchronous build call.
        unsafe { OH_Drawing_BitmapBuild(raw, width, height, &format) };
        Ok(Self { raw })
    }

    fn copy_pixels(&self, byte_len: usize) -> Result<Vec<u8>, ChartExportError> {
        // SAFETY: the bitmap is live and fully built before its pixel pointer
        // is queried.
        let pixels = unsafe { OH_Drawing_BitmapGetPixels(self.raw) }.cast::<u8>();
        if pixels.is_null() {
            return Err(ChartExportError::Native(
                "OH_Drawing_BitmapGetPixels returned null",
            ));
        }
        // SAFETY: `byte_len` was checked from the exact bitmap dimensions and
        // BGRA8888 format (four bytes per pixel); the returned pointer remains
        // valid until `self` is dropped after this copy.
        Ok(unsafe { std::slice::from_raw_parts(pixels, byte_len) }.to_vec())
    }
}

impl Drop for NativeBitmap {
    fn drop(&mut self) {
        // SAFETY: `raw` is owned by this guard and destroyed exactly once.
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
