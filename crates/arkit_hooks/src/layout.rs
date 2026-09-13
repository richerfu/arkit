//! Layout observation for an exact RSX element.

use std::cell::RefCell;
use std::rc::Rc;

use arkit_arkui::{LayoutFramePx, LayoutSizePx, NativeElementEvent, NativeElementRef};
use arkit_prelude::use_hook;

use crate::node::use_native_element_events;

fn size_from_frame(frame: LayoutFramePx) -> LayoutSizePx {
    frame.into()
}

fn size_close(previous: LayoutSizePx, next: LayoutSizePx) -> bool {
    (previous.width - next.width).abs() < 0.5 && (previous.height - next.height).abs() < 0.5
}

fn frame_close(previous: LayoutFramePx, next: LayoutFramePx) -> bool {
    size_close(size_from_frame(previous), size_from_frame(next))
        && (previous.x - next.x).abs() < 0.5
        && (previous.y - next.y).abs() < 0.5
}

type LayoutCallback<T> = Rc<dyn Fn(T)>;

fn use_layout_event<T: Copy + 'static>(
    reference: NativeElementRef,
    callback: LayoutCallback<T>,
    project: impl Fn(LayoutFramePx) -> T + 'static,
    is_close: impl Fn(T, T) -> bool + 'static,
) {
    // This declaration must happen synchronously during render. Effects run
    // after the native_ref attribute has already been projected.
    reference.request_layout_observation();
    let last = use_hook(|| Rc::new(RefCell::new(None::<T>)));
    use_native_element_events(reference, move |event| {
        let frame = match event {
            Some(NativeElementEvent::Mounted(lease)) => lease.layout_frame_px(),
            Some(NativeElementEvent::Layout { frame, .. }) => frame.is_measured().then_some(frame),
            None | Some(NativeElementEvent::Unmounted { .. }) => {
                last.borrow_mut().take();
                None
            }
            Some(NativeElementEvent::Visibility { .. }) => None,
        };
        let Some(frame) = frame else {
            return;
        };
        let next = project(frame);
        let changed = !last
            .borrow()
            .is_some_and(|previous| is_close(previous, next));
        if changed {
            last.replace(Some(next));
            callback(next);
        }
    });
}

/// Observe the measured size of the element carrying `reference`.
///
/// The same handle must be assigned to that element's `native_ref` attribute.
#[track_caller]
pub fn use_layout_size(reference: NativeElementRef, on_change: impl Fn(LayoutSizePx) + 'static) {
    use_layout_event(reference, Rc::new(on_change), size_from_frame, size_close);
}

/// Observe the window-relative frame of the element carrying `reference`.
///
/// The same handle must be assigned to that element's `native_ref` attribute.
#[track_caller]
pub fn use_layout_frame(reference: NativeElementRef, on_change: impl Fn(LayoutFramePx) + 'static) {
    use_layout_event(reference, Rc::new(on_change), |frame| frame, frame_close);
}

/// Live window-relative frame for a mounted element, if it has been measured.
///
/// Overlay anchors should prefer this over the last `use_layout_frame` sample:
/// area-change events can lag a scroll by a frame, so a tap immediately after
/// scrolling would otherwise open against a stale trigger rect.
pub fn current_layout_frame(reference: &NativeElementRef) -> Option<LayoutFramePx> {
    reference
        .current()
        .and_then(|lease| lease.layout_frame_px())
        .filter(|frame| frame.is_measured())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subpixel_layout_noise_is_ignored() {
        let first = LayoutFramePx {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 40.0,
        };
        let noise = LayoutFramePx {
            x: 10.25,
            y: 19.75,
            width: 100.25,
            height: 39.75,
        };
        assert!(frame_close(first, noise));
    }
}
