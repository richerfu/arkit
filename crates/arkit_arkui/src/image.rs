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
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::image_native_binding::types::ImageSize;
use ohos_arkui_binding::image_native_binding::{DecodingOptions, ImageSource, PixelMap};

/// A declarative image source carried through `AttributeValue::Any`.
///
/// Clones cheaply (the SVG bytes are shared via `Rc`). Two sources are equal
/// when their key + bytes + dimensions match, so dioxus diff skips redundant
/// updates.
#[derive(Clone)]
pub struct ArkImageSource {
    key: Arc<str>,
    svg: Arc<str>,
    width: u32,
    height: u32,
}

impl PartialEq for ArkImageSource {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.svg == other.svg
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
            svg: Arc::from(svg.into()),
            width,
            height,
        }
    }

    /// Create a source from already-shared SVG storage.
    pub fn svg_shared(key: impl Into<Arc<str>>, svg: Arc<str>, width: u32, height: u32) -> Self {
        Self {
            key: key.into(),
            svg,
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
                svg: self.svg.clone(),
                width: self.width,
                height: self.height,
            };
            if let Some(existing) = cache.borrow_mut().get(&key) {
                return Ok(existing);
            }
            let retained = Rc::new(RetainedImage::decode(&self.svg, self.width, self.height)?);
            cache.borrow_mut().insert(key, &retained);
            Ok(retained)
        })
    }
}

/// A decoded native image resource. Holds the PixelMap (keeps the decoding
/// alive) and the DrawableDescriptor (the ArkUI-usable handle). `Drop` is a
/// no-op — the `Rc` refcount controls lifetime. When the last `Rc` drops
/// (renderer disposes the host node), the PixelMap and DrawableDescriptor are
/// freed.
pub struct RetainedImage {
    _pixel_map: PixelMap,
    drawable: DrawableDescriptor,
}

impl std::fmt::Debug for RetainedImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedImage").finish_non_exhaustive()
    }
}

impl RetainedImage {
    fn decode(svg: &str, width: u32, height: u32) -> ArkUIResult<Self> {
        let mut svg_bytes = svg.as_bytes().to_vec();
        let source = ImageSource::create_from_data(svg_bytes.as_mut_slice()).map_err(|e| {
            ohos_arkui_binding::common::error::ArkUIError::new(
                ohos_arkui_binding::arkui_input_binding::ArkUIErrorCode::ParamInvalid,
                e.to_string(),
            )
        })?;
        let mut options = DecodingOptions::new().map_err(|e| {
            ohos_arkui_binding::common::error::ArkUIError::new(
                ohos_arkui_binding::arkui_input_binding::ArkUIErrorCode::ParamInvalid,
                e.to_string(),
            )
        })?;
        options
            .set_desired_size(ImageSize { width, height })
            .map_err(|e| {
                ohos_arkui_binding::common::error::ArkUIError::new(
                    ohos_arkui_binding::arkui_input_binding::ArkUIErrorCode::ParamInvalid,
                    e.to_string(),
                )
            })?;
        let pixel_map = source.create_pixelmap(&mut options).map_err(|e| {
            ohos_arkui_binding::common::error::ArkUIError::new(
                ohos_arkui_binding::arkui_input_binding::ArkUIErrorCode::ParamInvalid,
                e.to_string(),
            )
        })?;
        let drawable = DrawableDescriptor::from_pixel_map(pixel_map.handle())?;
        Ok(Self {
            _pixel_map: pixel_map,
            drawable,
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
    svg: Arc<str>,
    width: u32,
    height: u32,
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
