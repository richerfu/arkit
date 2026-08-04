//! ArkUI native image resource types.
//!
//! [`ArkImageSource`] is a declarative image source value that flows through
//! dioxus `AttributeValue::Any`. The renderer resolves it to a
//! [`RetainedImage`] (PixelMap + DrawableDescriptor) and holds it for the
//! lifetime of the host node, dropping it on dispose.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::rc::Weak;
use std::sync::Arc;

use ohos_arkui_binding::api::attribute_option::DrawableDescriptor;
use ohos_arkui_binding::common::error::{ArkUIError, ArkUIResult};
use ohos_arkui_binding::image_native_binding::types::{ImageSize, PixelFormat};
use ohos_arkui_binding::image_native_binding::{DecodingOptions, ImageSource, PixelMap};

/// A declarative image source carried through `AttributeValue::Any`.
///
/// Clones cheaply because encoded bytes are shared. Two sources are equal when
/// their key + bytes + dimensions match, so dioxus diff skips redundant
/// updates.
#[derive(Clone)]
pub struct ArkImageSource {
    key: Arc<str>,
    data: ArkImageData,
    width: u32,
    height: u32,
}

impl PartialEq for ArkImageSource {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.data == other.data
            && self.width == other.width
            && self.height == other.height
    }
}

impl std::fmt::Debug for ArkImageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArkImageSource")
            .field("key", &self.key)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl ArkImageSource {
    /// Create an image source from an SVG string. `key` should be unique per
    /// (icon-name, size, color, stroke-width) combination for caching.
    pub fn svg(key: impl Into<String>, svg: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            key: Arc::from(key.into()),
            data: ArkImageData::Svg(Arc::from(svg.into())),
            width,
            height,
        }
    }

    /// Create a source from already-shared SVG storage.
    pub fn svg_shared(key: impl Into<Arc<str>>, svg: Arc<str>, width: u32, height: u32) -> Self {
        Self {
            key: key.into(),
            data: ArkImageData::Svg(svg),
            width,
            height,
        }
    }

    /// Create an image source from encoded image bytes such as PNG or WebP.
    ///
    /// The bytes are decoded to the requested pixel dimensions. `key` must
    /// identify both the encoded content and the requested dimensions.
    pub fn encoded(
        key: impl Into<String>,
        data: impl Into<Vec<u8>>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            key: Arc::from(key.into()),
            data: ArkImageData::Encoded(Arc::from(data.into())),
            width,
            height,
        }
    }

    /// Create an image source from already-shared encoded image bytes.
    pub fn encoded_shared(
        key: impl Into<Arc<str>>,
        data: Arc<[u8]>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            key: key.into(),
            data: ArkImageData::Encoded(data),
            width,
            height,
        }
    }

    /// Return the same encoded source decoded at different pixel dimensions.
    ///
    /// The content key and shared bytes are preserved; dimensions remain part
    /// of the retained-image cache key.
    pub fn with_dimensions(&self, width: u32, height: u32) -> Self {
        Self {
            key: self.key.clone(),
            data: self.data.clone(),
            width,
            height,
        }
    }

    /// Resolve to a [`RetainedImage`] (PixelMap → DrawableDescriptor).
    /// Cached per `key` in a thread-local.
    pub fn resolve(&self) -> ArkUIResult<Rc<RetainedImage>> {
        RETAINED_CACHE.with(|cache| {
            let key = ImageCacheKey {
                key: self.key.clone(),
                data: self.data.clone(),
                width: self.width,
                height: self.height,
            };
            if let Some(existing) = cache.borrow_mut().get(&key) {
                return Ok(existing);
            }
            let retained = Rc::new(RetainedImage::decode(
                self.data.as_bytes(),
                self.width,
                self.height,
            )?);
            cache.borrow_mut().insert(key, &retained);
            Ok(retained)
        })
    }

    /// Decode the source to RGBA8888 pixels.
    ///
    /// The renderer's retained image cache is reused, so repeated calls for
    /// the same source do not repeat image decoding.
    pub fn rgba_pixels(&self) -> ArkUIResult<ArkImagePixels> {
        self.resolve()?.rgba_pixels()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ArkImageData {
    Svg(Arc<str>),
    Encoded(Arc<[u8]>),
}

impl ArkImageData {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Svg(svg) => svg.as_bytes(),
            Self::Encoded(data) => data,
        }
    }
}

/// Decoded RGBA8888 image data.
#[derive(Debug)]
pub struct ArkImagePixels {
    width: u32,
    height: u32,
    row_stride: u32,
    alpha_type: i32,
    pixels: Vec<u8>,
}

impl ArkImagePixels {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn row_stride(&self) -> u32 {
        self.row_stride
    }

    pub fn alpha_type(&self) -> i32 {
        self.alpha_type
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }
}

/// A decoded native image resource. Holds the PixelMap (keeps the decoding
/// alive) and the DrawableDescriptor (the ArkUI-usable handle). `Drop` is a
/// no-op — the `Rc` refcount controls lifetime. When the last `Rc` drops
/// (renderer disposes the host node), the PixelMap and DrawableDescriptor are
/// freed.
pub struct RetainedImage {
    pixel_map: PixelMap,
    drawable: DrawableDescriptor,
}

impl std::fmt::Debug for RetainedImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedImage").finish_non_exhaustive()
    }
}

impl RetainedImage {
    fn decode(data: &[u8], width: u32, height: u32) -> ArkUIResult<Self> {
        let mut encoded_bytes = data.to_vec();
        let source =
            ImageSource::create_from_data(encoded_bytes.as_mut_slice()).map_err(image_error)?;
        let mut options = DecodingOptions::new().map_err(image_error)?;
        options
            .set_pixel_format(PixelFormat::Rgba8888)
            .map_err(image_error)?;
        options
            .set_desired_size(ImageSize { width, height })
            .map_err(image_error)?;
        let pixel_map = source.create_pixelmap(&mut options).map_err(image_error)?;
        let drawable = DrawableDescriptor::from_pixel_map(pixel_map.handle())?;
        Ok(Self {
            pixel_map,
            drawable,
        })
    }

    fn rgba_pixels(&self) -> ArkUIResult<ArkImagePixels> {
        let info = self.pixel_map.image_info().map_err(image_error)?;
        let width = info.width().map_err(image_error)?;
        let height = info.height().map_err(image_error)?;
        let row_stride = info.row_stride().map_err(image_error)?;
        let alpha_type = info.alpha_type().map_err(image_error)?;
        let byte_len = usize::try_from(row_stride)
            .ok()
            .and_then(|stride| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| stride.checked_mul(height))
            })
            .ok_or_else(|| image_error("decoded image byte length overflow"))?;
        let mut pixels = vec![0_u8; byte_len];
        let read = self
            .pixel_map
            .read_pixels(&mut pixels)
            .map_err(image_error)?;
        if read < byte_len {
            pixels.truncate(read);
        }
        Ok(ArkImagePixels {
            width,
            height,
            row_stride,
            // Binding 0.2.5 reports the alpha type as a typed PixelMapAlphaType
            // enum; ArkImagePixels keeps the raw i32 for the callers.
            alpha_type: alpha_type.into(),
            pixels,
        })
    }

    /// The ArkUI-usable drawable handle.
    pub fn drawable(&self) -> &DrawableDescriptor {
        &self.drawable
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ImageCacheKey {
    key: Arc<str>,
    data: ArkImageData,
    width: u32,
    height: u32,
}

fn image_error(error: impl ToString) -> ArkUIError {
    ArkUIError::new(
        ohos_arkui_binding::arkui_input_binding::ArkUIErrorCode::ParamInvalid,
        error.to_string(),
    )
}

const RETAINED_CACHE_CAPACITY: usize = 128;

#[derive(Default)]
struct RetainedImageCache {
    entries: BTreeMap<ImageCacheKey, Weak<RetainedImage>>,
    order: VecDeque<ImageCacheKey>,
}

impl RetainedImageCache {
    fn get(&mut self, key: &ImageCacheKey) -> Option<Rc<RetainedImage>> {
        let image = self.entries.get(key)?.upgrade();
        if image.is_some() {
            self.touch(key);
        } else {
            self.entries.remove(key);
            self.order.retain(|candidate| candidate != key);
        }
        image
    }

    fn insert(&mut self, key: ImageCacheKey, image: &Rc<RetainedImage>) {
        self.entries.insert(key.clone(), Rc::downgrade(image));
        self.touch(&key);
        while self.order.len() > RETAINED_CACHE_CAPACITY {
            if let Some(evicted) = self.order.pop_front() {
                self.entries.remove(&evicted);
            }
        }
    }

    fn touch(&mut self, key: &ImageCacheKey) {
        self.order.retain(|candidate| candidate != key);
        self.order.push_back(key.clone());
    }
}

thread_local! {
    static RETAINED_CACHE: RefCell<RetainedImageCache> =
        RefCell::new(RetainedImageCache::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_source_identity_includes_bytes_and_dimensions() {
        let source = ArkImageSource::encoded("logo", vec![1, 2, 3], 64, 32);
        assert_eq!(source, source.with_dimensions(64, 32));
        assert_ne!(source, source.with_dimensions(128, 64));
        assert_ne!(
            source,
            ArkImageSource::encoded("logo", vec![3, 2, 1], 64, 32)
        );
    }

    #[test]
    fn svg_and_encoded_sources_are_distinct() {
        let svg = ArkImageSource::svg("logo", "<svg/>", 32, 32);
        let encoded = ArkImageSource::encoded("logo", b"<svg/>".to_vec(), 32, 32);
        assert_ne!(svg, encoded);
    }
}
