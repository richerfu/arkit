//! `use_virtual_range` — the visible-item-range signal for virtual lists.
//!
//! This hook provides the plain-data range struct plus a `Signal`/setter that a
//! virtual-list component reads to drive ArkUI NodeAdapter item counts. The
//! setter is intended to be called from an `on_scroll` / `on_scroll_index`
//! event handler.
//!
//! ArkUI NodeAdapter ownership remains in `arkit_arkui`; this module owns only
//! the reactive visible-range value.

use arkit_prelude::{use_signal, Signal};

/// The visible item range reported by a virtual container's scroll-index event.
///
/// Indices are inclusive. A `first_index` of `0` and `last_index` of `-1`
/// represents "no items visible yet" (the default).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VirtualVisibleRange {
    pub first_index: i32,
    pub last_index: i32,
}

impl VirtualVisibleRange {
    pub fn new(first: i32, last: i32) -> Self {
        Self {
            first_index: first,
            last_index: last,
        }
    }

    pub fn is_empty(self) -> bool {
        self.last_index < self.first_index
    }
}

/// Allocate a visible-range signal and its setter.
///
/// Returns `(read, write)` where `read` is the signal a virtual-container
/// component watches and `write` is the same signal used as a setter (call
/// `write.set(...)` or `*write.write() = ...` from a scroll-index handler). In
/// dioxus 0.7 a `Signal` is both readable and writable; the pair is returned
/// for ergonomic destructuring and to make intent explicit.
///
/// ```ignore
/// fn my_list() -> Element {
///     let (range, set_range) = use_virtual_range();
///     rsx! {
///         list {
///             onscroll_index: move |evt| {
///                 set_range.set(VirtualVisibleRange::new(evt.first, evt.last));
///             }
///         }
///     }
/// }
/// ```
#[track_caller]
pub fn use_virtual_range() -> (Signal<VirtualVisibleRange>, Signal<VirtualVisibleRange>) {
    let signal: Signal<VirtualVisibleRange> = use_signal(VirtualVisibleRange::default);
    (signal, signal)
}
