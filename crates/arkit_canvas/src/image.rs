use std::rc::Rc;

use half::f16;
use ohos_drawing_binding::{AlphaFormat, Bitmap, BitmapFormat, ColorFormat};
use ohos_image_native_binding::{
    DecodingOptions, ImagePacker, ImageSource, ImageString, PackingOptions, PixelFormat, PixelMap,
    PixelMapAlphaType, PixelMapInitializationOptions,
};

use crate::color_space::ColorSpaceTransform;
use crate::{CanvasColorSpace, CanvasError, CanvasResult};

/// IEEE 754 binary16 channel value used by `rgba-float16` image data.
pub type Float16 = f16;

/// Image formats accepted by [`CanvasImage::encode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasImageFormat {
    Png,
    Jpeg,
    WebP,
    Avif,
    Heif,
}

impl CanvasImageFormat {
    pub const fn mime_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::WebP => "image/webp",
            Self::Avif => "image/avif",
            Self::Heif => "image/heif",
        }
    }
}

/// Decode controls corresponding to `createImageBitmap()`'s frame and resize
/// inputs. A missing size preserves the source dimensions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CanvasImageDecodeOptions {
    pub frame_index: u32,
    pub desired_size: Option<[u32; 2]>,
}

/// Options used by [`CanvasImage::encode`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasImageEncodeOptions {
    /// Lossy quality in the inclusive range 0..=1.
    pub quality: f32,
}

impl Default for CanvasImageEncodeOptions {
    fn default() -> Self {
        Self { quality: 0.92 }
    }
}

/// Storage behind an [`ImageData`] object, matching the web `ImageDataArray`
/// union.
#[derive(Clone, Debug, PartialEq)]
pub enum ImageDataArray {
    /// Four clamped 8-bit channels per pixel in RGBA order.
    RgbaUnorm8(Vec<u8>),
    /// Four IEEE 754 binary16 channels per pixel in RGBA order.
    RgbaFloat16(Vec<Float16>),
}

impl ImageDataArray {
    pub fn as_rgba_unorm8(&self) -> Option<&[u8]> {
        match self {
            Self::RgbaUnorm8(data) => Some(data),
            Self::RgbaFloat16(_) => None,
        }
    }

    pub fn as_rgba_unorm8_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            Self::RgbaUnorm8(data) => Some(data),
            Self::RgbaFloat16(_) => None,
        }
    }

    pub fn as_rgba_float16(&self) -> Option<&[Float16]> {
        match self {
            Self::RgbaUnorm8(_) => None,
            Self::RgbaFloat16(data) => Some(data),
        }
    }

    pub fn as_rgba_float16_mut(&mut self) -> Option<&mut [Float16]> {
        match self {
            Self::RgbaUnorm8(_) => None,
            Self::RgbaFloat16(data) => Some(data),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::RgbaUnorm8(data) => data.len(),
            Self::RgbaFloat16(data) => data.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageDataPixelFormat {
    #[default]
    RgbaUnorm8,
    RgbaFloat16,
}

/// Optional settings used to construct or read [`ImageData`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageDataSettings {
    /// `None` inherits the rendering context's color space.
    pub color_space: Option<CanvasColorSpace>,
    pub pixel_format: ImageDataPixelFormat,
}

impl ImageDataSettings {
    pub(crate) fn resolved(self, default_color_space: CanvasColorSpace) -> Self {
        Self {
            color_space: Some(self.color_space.unwrap_or(default_color_space)),
            ..self
        }
    }
}

/// Unpremultiplied RGBA pixels and their interpretation metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageData {
    width: u32,
    height: u32,
    color_space: CanvasColorSpace,
    pixel_format: ImageDataPixelFormat,
    data: ImageDataArray,
}

impl ImageData {
    pub fn new(width: u32, height: u32) -> CanvasResult<Self> {
        Self::new_with_settings(width, height, ImageDataSettings::default())
    }

    pub fn new_with_settings(
        width: u32,
        height: u32,
        settings: ImageDataSettings,
    ) -> CanvasResult<Self> {
        Self::new_resolved(
            width,
            height,
            settings.color_space.unwrap_or_default(),
            settings.pixel_format,
        )
    }

    pub fn from_rgba(data: Vec<u8>, width: u32, height: u32) -> CanvasResult<Self> {
        Self::from_rgba_with_color_space(data, width, height, CanvasColorSpace::Srgb)
    }

    pub fn from_rgba_with_color_space(
        data: Vec<u8>,
        width: u32,
        height: u32,
        color_space: CanvasColorSpace,
    ) -> CanvasResult<Self> {
        if data.len() != Self::component_length(width, height)? {
            return Err(CanvasError::InvalidImageData);
        }
        Ok(Self {
            width,
            height,
            color_space,
            pixel_format: ImageDataPixelFormat::RgbaUnorm8,
            data: ImageDataArray::RgbaUnorm8(data),
        })
    }

    pub fn from_rgba_float16(
        data: Vec<Float16>,
        width: u32,
        height: u32,
        color_space: CanvasColorSpace,
    ) -> CanvasResult<Self> {
        if data.len() != Self::component_length(width, height)? {
            return Err(CanvasError::InvalidImageData);
        }
        Ok(Self {
            width,
            height,
            color_space,
            pixel_format: ImageDataPixelFormat::RgbaFloat16,
            data: ImageDataArray::RgbaFloat16(data),
        })
    }

    fn new_resolved(
        width: u32,
        height: u32,
        color_space: CanvasColorSpace,
        pixel_format: ImageDataPixelFormat,
    ) -> CanvasResult<Self> {
        let length = Self::component_length(width, height)?;
        let data = match pixel_format {
            ImageDataPixelFormat::RgbaUnorm8 => {
                ImageDataArray::RgbaUnorm8(Self::allocate_zeroed(length)?)
            }
            ImageDataPixelFormat::RgbaFloat16 => {
                ImageDataArray::RgbaFloat16(Self::allocate_zeroed(length)?)
            }
        };
        Ok(Self {
            width,
            height,
            color_space,
            pixel_format,
            data,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn color_space(&self) -> CanvasColorSpace {
        self.color_space
    }

    pub fn pixel_format(&self) -> ImageDataPixelFormat {
        self.pixel_format
    }

    pub fn data(&self) -> &ImageDataArray {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut ImageDataArray {
        &mut self.data
    }

    pub fn rgba_unorm8(&self) -> Option<&[u8]> {
        self.data.as_rgba_unorm8()
    }

    pub fn rgba_unorm8_mut(&mut self) -> Option<&mut [u8]> {
        self.data.as_rgba_unorm8_mut()
    }

    pub fn rgba_float16(&self) -> Option<&[Float16]> {
        self.data.as_rgba_float16()
    }

    pub fn rgba_float16_mut(&mut self) -> Option<&mut [Float16]> {
        self.data.as_rgba_float16_mut()
    }

    pub fn into_data(self) -> ImageDataArray {
        self.data
    }

    pub(crate) fn read_pixel(&self, component_offset: usize) -> [f32; 4] {
        match &self.data {
            ImageDataArray::RgbaUnorm8(data) => [
                f32::from(data[component_offset]) / 255.0,
                f32::from(data[component_offset + 1]) / 255.0,
                f32::from(data[component_offset + 2]) / 255.0,
                f32::from(data[component_offset + 3]) / 255.0,
            ],
            ImageDataArray::RgbaFloat16(data) => [
                data[component_offset].to_f32(),
                data[component_offset + 1].to_f32(),
                data[component_offset + 2].to_f32(),
                data[component_offset + 3].to_f32(),
            ],
        }
    }

    pub(crate) fn write_pixel(&mut self, component_offset: usize, rgba: [f32; 4]) {
        match &mut self.data {
            ImageDataArray::RgbaUnorm8(data) => {
                for (destination, value) in data[component_offset..component_offset + 4]
                    .iter_mut()
                    .zip(rgba)
                {
                    *destination = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
            ImageDataArray::RgbaFloat16(data) => {
                for (destination, value) in data[component_offset..component_offset + 4]
                    .iter_mut()
                    .zip(rgba)
                {
                    *destination = Float16::from_f32(value);
                }
            }
        }
    }

    fn to_srgb_unorm8(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.data.len());
        for offset in (0..self.data.len()).step_by(4) {
            let mut pixel = self.read_pixel(offset);
            ColorSpaceTransform::convert(&mut pixel, self.color_space, CanvasColorSpace::Srgb);
            result.extend(pixel.map(Self::float_to_unorm8));
        }
        result
    }

    pub(crate) fn normalized_dimensions(width: i32, height: i32) -> CanvasResult<(u32, u32)> {
        if width == 0 || height == 0 {
            return Err(CanvasError::InvalidImageData);
        }
        Ok((width.unsigned_abs(), height.unsigned_abs()))
    }

    pub(crate) fn normalized_axis(origin: i32, size: i32) -> CanvasResult<(i64, u32)> {
        if size == 0 {
            return Err(CanvasError::InvalidImageData);
        }
        let origin = i64::from(origin);
        if size < 0 {
            Ok((origin + i64::from(size), size.unsigned_abs()))
        } else {
            Ok((origin, size as u32))
        }
    }

    pub(crate) fn premultiply_pixel(source: [f32; 4], force_opaque: bool) -> [u8; 4] {
        let alpha = source[3].clamp(0.0, 1.0);
        [
            (source[0].clamp(0.0, 1.0) * alpha * 255.0).round() as u8,
            (source[1].clamp(0.0, 1.0) * alpha * 255.0).round() as u8,
            (source[2].clamp(0.0, 1.0) * alpha * 255.0).round() as u8,
            if force_opaque {
                255
            } else {
                (alpha * 255.0).round() as u8
            },
        ]
    }

    pub(crate) fn unpremultiply_pixel(source: &[u8]) -> [u8; 4] {
        let alpha = u16::from(source[3]);
        if alpha == 0 {
            return [0, 0, 0, 0];
        }
        [
            ((u16::from(source[0]) * 255 + alpha / 2) / alpha).min(255) as u8,
            ((u16::from(source[1]) * 255 + alpha / 2) / alpha).min(255) as u8,
            ((u16::from(source[2]) * 255 + alpha / 2) / alpha).min(255) as u8,
            source[3],
        ]
    }

    fn component_length(width: u32, height: u32) -> CanvasResult<usize> {
        if width == 0 || height == 0 {
            return Err(CanvasError::InvalidImageData);
        }
        usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(CanvasError::InvalidImageData)
    }

    fn allocate_zeroed<T: Clone + Default>(length: usize) -> CanvasResult<Vec<T>> {
        let mut data = Vec::new();
        data.try_reserve_exact(length)
            .map_err(|_| CanvasError::ImageDataAllocation)?;
        data.resize(length, T::default());
        Ok(data)
    }

    fn float_to_unorm8(value: f32) -> u8 {
        (value.clamp(0.0, 1.0) * 255.0).round() as u8
    }
}

/// Immutable Canvas image source backed by a native bitmap.
#[derive(Clone, Debug)]
pub struct CanvasImage {
    bitmap: Rc<Bitmap>,
    alpha: CanvasBitmapAlpha,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanvasBitmapAlpha {
    Opaque,
    Premultiplied,
    Unpremultiplied,
}

impl CanvasImage {
    /// Creates an image from tightly packed, unpremultiplied RGBA bytes.
    pub fn from_rgba(data: Vec<u8>, width: u32, height: u32) -> CanvasResult<Self> {
        Ok(Self::from_image_data(&ImageData::from_rgba(
            data, width, height,
        )?))
    }

    pub fn from_image_data(image_data: &ImageData) -> Self {
        let mut bitmap = Bitmap::new(
            image_data.width,
            image_data.height,
            BitmapFormat {
                color: ColorFormat::Rgba8888,
                alpha: AlphaFormat::Unpremul,
            },
        );
        bitmap
            .pixels_mut()
            .copy_from_slice(&image_data.to_srgb_unorm8());
        Self {
            bitmap: Rc::new(bitmap),
            alpha: CanvasBitmapAlpha::Unpremultiplied,
        }
    }

    pub(crate) fn from_canvas_bitmap(bitmap: &Bitmap, alpha: bool) -> Self {
        let mut snapshot = Bitmap::new(
            bitmap.width(),
            bitmap.height(),
            BitmapFormat {
                color: ColorFormat::Rgba8888,
                alpha: if alpha {
                    AlphaFormat::Premul
                } else {
                    AlphaFormat::Opaque
                },
            },
        );
        snapshot.pixels_mut().copy_from_slice(bitmap.pixels());
        Self {
            bitmap: Rc::new(snapshot),
            alpha: if alpha {
                CanvasBitmapAlpha::Premultiplied
            } else {
                CanvasBitmapAlpha::Opaque
            },
        }
    }

    /// Decodes PNG, JPEG, WebP, GIF, SVG, HEIF, or any other format exposed by
    /// the device image framework into a native canvas image.
    pub fn decode(data: &[u8], options: CanvasImageDecodeOptions) -> CanvasResult<Self> {
        if data.is_empty()
            || options
                .desired_size
                .is_some_and(|[width, height]| width == 0 || height == 0)
        {
            return Err(CanvasError::InvalidImageData);
        }

        let source = ImageSource::create_from_data(data)
            .map_err(|error| CanvasError::ImageDecode(error.code))?;
        let mut decoding =
            DecodingOptions::new().map_err(|error| CanvasError::ImageDecode(error.code))?;
        decoding
            .set_index(options.frame_index)
            .map_err(|error| CanvasError::ImageDecode(error.code))?;
        decoding
            .set_pixel_format(PixelFormat::Rgba8888)
            .map_err(|error| CanvasError::ImageDecode(error.code))?;
        if let Some([width, height]) = options.desired_size {
            decoding
                .set_desired_size(ohos_image_native_binding::types::ImageSize { width, height })
                .map_err(|error| CanvasError::ImageDecode(error.code))?;
        }
        let pixelmap = source
            .create_pixelmap(&mut decoding)
            .map_err(|error| CanvasError::ImageDecode(error.code))?;
        Self::from_pixelmap(&pixelmap)
    }

    /// Returns the decoder formats exposed by the current device.
    pub fn supported_decode_formats() -> CanvasResult<Vec<String>> {
        ImageSource::supported_formats().map_err(|error| CanvasError::ImageDecode(error.code))
    }

    /// Returns the encoder formats exposed by the current device.
    pub fn supported_encode_formats() -> CanvasResult<Vec<String>> {
        ImagePacker::supported_formats().map_err(|error| CanvasError::ImageEncode(error.code))
    }

    /// Encodes this image with the device image framework.
    pub fn encode(
        &self,
        format: CanvasImageFormat,
        options: CanvasImageEncodeOptions,
    ) -> CanvasResult<Vec<u8>> {
        if !options.quality.is_finite() {
            return Err(CanvasError::NonFinite);
        }
        let supported = Self::supported_encode_formats()?;
        if !supported
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(format.mime_type()))
        {
            return Err(CanvasError::UnsupportedImageFormat);
        }

        let mut pixels = Self::copy_bytes(self.bitmap.pixels())?;
        let mut initialization = PixelMapInitializationOptions::new()
            .map_err(|error| CanvasError::ImageEncode(error.code))?;
        initialization
            .set_width(self.width())
            .map_err(|error| CanvasError::ImageEncode(error.code))?;
        initialization
            .set_height(self.height())
            .map_err(|error| CanvasError::ImageEncode(error.code))?;
        initialization
            .set_src_pixel_format(PixelFormat::Rgba8888)
            .map_err(|error| CanvasError::ImageEncode(error.code))?;
        initialization
            .set_pixel_format(PixelFormat::Rgba8888)
            .map_err(|error| CanvasError::ImageEncode(error.code))?;
        initialization
            .set_row_stride(
                i32::try_from(
                    self.width()
                        .checked_mul(4)
                        .ok_or(CanvasError::InvalidDimensions)?,
                )
                .map_err(|_| CanvasError::InvalidDimensions)?,
            )
            .map_err(|error| CanvasError::ImageEncode(error.code))?;
        initialization
            .set_alpha_type(match self.alpha {
                CanvasBitmapAlpha::Opaque => PixelMapAlphaType::Opaque,
                CanvasBitmapAlpha::Premultiplied => PixelMapAlphaType::Premultiplied,
                CanvasBitmapAlpha::Unpremultiplied => PixelMapAlphaType::Unpremultiplied,
            })
            .map_err(|error| CanvasError::ImageEncode(error.code))?;
        let pixelmap = PixelMap::create(&mut pixels, &mut initialization)
            .map_err(|error| CanvasError::ImageEncode(error.code))?;

        let mut packing =
            PackingOptions::new().map_err(|error| CanvasError::ImageEncode(error.code))?;
        let mut mime = ImageString::from_str(format.mime_type())
            .map_err(|error| CanvasError::ImageEncode(error.code))?;
        packing
            .set_mime_type(&mut mime)
            .map_err(|error| CanvasError::ImageEncode(error.code))?;
        packing
            .set_quality((options.quality.clamp(0.0, 1.0) * 100.0).round() as u32)
            .map_err(|error| CanvasError::ImageEncode(error.code))?;

        let raw_length = pixels.len();
        let output_capacity = raw_length
            .checked_add((raw_length / 8).max(64 * 1024))
            .ok_or(CanvasError::ImageDataAllocation)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(output_capacity)
            .map_err(|_| CanvasError::ImageDataAllocation)?;
        output.resize(output_capacity, 0);
        let mut packer =
            ImagePacker::new().map_err(|error| CanvasError::ImageEncode(error.code))?;
        let length = packer
            .pack_to_data_from_pixelmap(&mut packing, &pixelmap, &mut output)
            .map_err(|error| CanvasError::ImageEncode(error.code))?;
        if length > output.len() {
            return Err(CanvasError::ImageEncode(
                ohos_image_native_binding::sys::Image_ErrorCode_IMAGE_ENCODE_FAILED,
            ));
        }
        output.truncate(length);
        Ok(output)
    }

    /// Copies the native bitmap into W3C-style unpremultiplied RGBA bytes.
    pub fn to_image_data(&self) -> CanvasResult<ImageData> {
        let mut pixels = Self::copy_bytes(self.bitmap.pixels())?;
        if self.alpha == CanvasBitmapAlpha::Premultiplied {
            for pixel in pixels.chunks_exact_mut(4) {
                pixel.copy_from_slice(&ImageData::unpremultiply_pixel(pixel));
            }
        }
        ImageData::from_rgba(pixels, self.width(), self.height())
    }

    fn from_pixelmap(pixelmap: &PixelMap) -> CanvasResult<Self> {
        let info = pixelmap
            .image_info()
            .map_err(|error| CanvasError::ImageDecode(error.code))?;
        let width = info
            .width()
            .map_err(|error| CanvasError::ImageDecode(error.code))?;
        let height = info
            .height()
            .map_err(|error| CanvasError::ImageDecode(error.code))?;
        if width == 0 || height == 0 {
            return Err(CanvasError::InvalidImageData);
        }
        let row_stride = usize::try_from(
            info.row_stride()
                .map_err(|error| CanvasError::ImageDecode(error.code))?,
        )
        .map_err(|_| CanvasError::InvalidDimensions)?;
        let tight_stride = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or(CanvasError::InvalidDimensions)?;
        if row_stride < tight_stride {
            return Err(CanvasError::InvalidImageData);
        }
        let storage_length = row_stride
            .checked_mul(usize::try_from(height).map_err(|_| CanvasError::InvalidDimensions)?)
            .ok_or(CanvasError::InvalidDimensions)?;
        let mut storage = Vec::new();
        storage
            .try_reserve_exact(storage_length)
            .map_err(|_| CanvasError::ImageDataAllocation)?;
        storage.resize(storage_length, 0);
        let written = pixelmap
            .read_pixels(&mut storage)
            .map_err(|error| CanvasError::ImageDecode(error.code))?;
        if written < storage_length {
            return Err(CanvasError::InvalidImageData);
        }
        let alpha = match info
            .alpha_type()
            .map_err(|error| CanvasError::ImageDecode(error.code))?
        {
            PixelMapAlphaType::Opaque => CanvasBitmapAlpha::Opaque,
            PixelMapAlphaType::Premultiplied => CanvasBitmapAlpha::Premultiplied,
            PixelMapAlphaType::Unknown | PixelMapAlphaType::Unpremultiplied => {
                CanvasBitmapAlpha::Unpremultiplied
            }
        };
        let mut bitmap = Bitmap::new(
            width,
            height,
            BitmapFormat {
                color: ColorFormat::Rgba8888,
                alpha: match alpha {
                    CanvasBitmapAlpha::Opaque => AlphaFormat::Opaque,
                    CanvasBitmapAlpha::Premultiplied => AlphaFormat::Premul,
                    CanvasBitmapAlpha::Unpremultiplied => AlphaFormat::Unpremul,
                },
            },
        );
        for (destination, source) in bitmap
            .pixels_mut()
            .chunks_exact_mut(tight_stride)
            .zip(storage.chunks_exact(row_stride))
        {
            destination.copy_from_slice(&source[..tight_stride]);
        }
        Ok(Self {
            bitmap: Rc::new(bitmap),
            alpha,
        })
    }

    fn copy_bytes(source: &[u8]) -> CanvasResult<Vec<u8>> {
        let mut copy = Vec::new();
        copy.try_reserve_exact(source.len())
            .map_err(|_| CanvasError::ImageDataAllocation)?;
        copy.extend_from_slice(source);
        Ok(copy)
    }

    pub fn width(&self) -> u32 {
        self.bitmap.width()
    }

    pub fn height(&self) -> u32 {
        self.bitmap.height()
    }

    pub(crate) fn bitmap(&self) -> &Bitmap {
        &self.bitmap
    }
}

impl PartialEq for CanvasImage {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.bitmap, &other.bitmap)
    }
}

impl Eq for CanvasImage {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_both_w3c_image_data_formats() {
        let unorm = ImageData::new(2, 3).unwrap();
        assert_eq!(unorm.pixel_format(), ImageDataPixelFormat::RgbaUnorm8);
        assert_eq!(unorm.data().len(), 24);

        let float = ImageData::new_with_settings(
            2,
            3,
            ImageDataSettings {
                color_space: Some(CanvasColorSpace::DisplayP3),
                pixel_format: ImageDataPixelFormat::RgbaFloat16,
            },
        )
        .unwrap();
        assert_eq!(float.color_space(), CanvasColorSpace::DisplayP3);
        assert_eq!(float.pixel_format(), ImageDataPixelFormat::RgbaFloat16);
        assert_eq!(float.data().len(), 24);
    }

    #[test]
    fn normalizes_signed_canvas_dimensions() {
        assert_eq!(ImageData::normalized_dimensions(-12, 8), Ok((12, 8)));
        assert_eq!(ImageData::normalized_dimensions(12, -8), Ok((12, 8)));
        assert_eq!(
            ImageData::normalized_dimensions(0, 8),
            Err(CanvasError::InvalidImageData)
        );
    }

    #[test]
    fn converts_between_srgb_and_display_p3() {
        let mut red = [1.0, 0.0, 0.0, 1.0];
        ColorSpaceTransform::convert(
            &mut red,
            CanvasColorSpace::Srgb,
            CanvasColorSpace::DisplayP3,
        );
        assert!((red[0] - 0.917_5).abs() < 0.001);
        assert!((red[1] - 0.200_3).abs() < 0.001);
        assert!((red[2] - 0.138_6).abs() < 0.001);

        ColorSpaceTransform::convert(
            &mut red,
            CanvasColorSpace::DisplayP3,
            CanvasColorSpace::Srgb,
        );
        assert!((red[0] - 1.0).abs() < 0.001);
        assert!(red[1].abs() < 0.001);
        assert!(red[2].abs() < 0.001);
    }
}
