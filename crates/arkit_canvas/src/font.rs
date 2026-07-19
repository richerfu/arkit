use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use ohos_drawing_binding::Typeface;

use crate::{CanvasError, CanvasResult};

thread_local! {
    static REGISTERED_FONTS: RefCell<HashMap<Box<str>, CanvasFontFace>> =
        RefCell::new(HashMap::new());
}

/// A validated native font face, similar to the web platform's `FontFace`.
///
/// The bytes are retained so each native `Font` can own the `Typeface` it
/// borrows without leaking global native objects.
#[derive(Clone, Debug)]
pub struct CanvasFontFace {
    family: Box<str>,
    bytes: Rc<[u8]>,
    collection_index: i32,
    typeface: Rc<Typeface>,
}

impl CanvasFontFace {
    pub fn from_bytes(
        family: impl Into<Box<str>>,
        bytes: impl Into<Vec<u8>>,
    ) -> CanvasResult<Self> {
        Self::from_collection_bytes(family, bytes, 0)
    }

    pub fn from_collection_bytes(
        family: impl Into<Box<str>>,
        bytes: impl Into<Vec<u8>>,
        collection_index: i32,
    ) -> CanvasResult<Self> {
        let family = family.into();
        let bytes = bytes.into();
        if family.trim().is_empty() || family.contains('\0') || bytes.is_empty() {
            return Err(CanvasError::InvalidFont);
        }
        let typeface = Typeface::from_data(&bytes, collection_index)
            .map(Rc::new)
            .map_err(|_| CanvasError::InvalidFont)?;
        Ok(Self {
            family,
            bytes: bytes.into(),
            collection_index,
            typeface,
        })
    }

    pub fn from_file(family: impl Into<Box<str>>, path: impl AsRef<Path>) -> CanvasResult<Self> {
        Self::from_collection_file(family, path, 0)
    }

    pub fn from_collection_file(
        family: impl Into<Box<str>>,
        path: impl AsRef<Path>,
        collection_index: i32,
    ) -> CanvasResult<Self> {
        let bytes = std::fs::read(path).map_err(|_| CanvasError::FontIo)?;
        Self::from_collection_bytes(family, bytes, collection_index)
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn collection_index(&self) -> i32 {
        self.collection_index
    }

    pub fn data(&self) -> &[u8] {
        &self.bytes
    }
}

/// Thread-local registry consulted by every Canvas 2D text operation.
///
/// Keeping this registry thread-local matches OHOS drawing object affinity and
/// avoids making native pointer wrappers spuriously `Send` or `Sync`.
#[derive(Clone, Copy, Debug, Default)]
pub struct CanvasFontRegistry;

impl CanvasFontRegistry {
    pub fn register(face: CanvasFontFace) -> Option<CanvasFontFace> {
        REGISTERED_FONTS
            .with_borrow_mut(|fonts| fonts.insert(Self::normalized_family(face.family()), face))
    }

    pub fn unregister(family: &str) -> Option<CanvasFontFace> {
        REGISTERED_FONTS
            .with_borrow_mut(|fonts| fonts.remove(Self::normalized_family(family).as_ref()))
    }

    pub fn contains(family: &str) -> bool {
        REGISTERED_FONTS
            .with_borrow(|fonts| fonts.contains_key(Self::normalized_family(family).as_ref()))
    }

    pub fn families() -> Vec<String> {
        REGISTERED_FONTS.with_borrow(|fonts| {
            let mut families: Vec<_> = fonts
                .values()
                .map(|face| face.family().to_owned())
                .collect();
            families.sort_unstable();
            families
        })
    }

    pub fn clear() {
        REGISTERED_FONTS.with_borrow_mut(HashMap::clear);
    }

    pub(crate) fn resolve_typeface(families: &[&str]) -> Option<Rc<Typeface>> {
        REGISTERED_FONTS.with_borrow(|fonts| {
            families.iter().find_map(|family| {
                fonts
                    .get(Self::normalized_family(family).as_ref())
                    .map(|face| face.typeface.clone())
            })
        })
    }

    fn normalized_family(family: &str) -> Box<str> {
        family.trim().to_lowercase().into_boxed_str()
    }
}
