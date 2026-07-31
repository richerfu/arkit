//! Layout observation for an exact RSX element.

use std::cell::RefCell;
use std::rc::Rc;

use arkit_arkui::{LayoutFramePx, NativeElementEvent, NativeElementRef, NativeElementSubscription};
use arkit_prelude::{use_drop, use_effect, use_hook};

/// A measured size in physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutSize {
    pub width: f32,
    pub height: f32,
}

impl LayoutSize {
    pub fn is_measured(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

/// A measured frame in physical pixels, relative to the window.
pub type LayoutFrame = LayoutFramePx;

fn size_from_frame(frame: LayoutFrame) -> LayoutSize {
    LayoutSize {
        width: frame.width,
        height: frame.height,
    }
}

fn size_close(previous: LayoutSize, next: LayoutSize) -> bool {
    (previous.width - next.width).abs() < 0.5 && (previous.height - next.height).abs() < 0.5
}

fn frame_close(previous: LayoutFrame, next: LayoutFrame) -> bool {
    size_close(size_from_frame(previous), size_from_frame(next))
        && (previous.x - next.x).abs() < 0.5
        && (previous.y - next.y).abs() < 0.5
}

type LayoutCallback<T> = Rc<dyn Fn(T)>;
type SharedLayoutCallback<T> = Rc<RefCell<LayoutCallback<T>>>;

struct LayoutHookState<T: Copy + 'static> {
    callback: SharedLayoutCallback<T>,
    subscription: Rc<RefCell<Option<NativeElementSubscription>>>,
}

impl<T: Copy + 'static> Clone for LayoutHookState<T> {
    fn clone(&self) -> Self {
        Self {
            callback: self.callback.clone(),
            subscription: self.subscription.clone(),
        }
    }
}

fn use_layout_event<T: Copy + 'static>(
    reference: NativeElementRef,
    initial_callback: LayoutCallback<T>,
    project: impl Fn(LayoutFrame) -> T + 'static,
    is_close: impl Fn(T, T) -> bool + 'static,
) {
    // This declaration must happen synchronously during render. Effects run
    // after the native_ref attribute has already been projected.
    reference.request_layout_observation();
    let project = Rc::new(project) as Rc<dyn Fn(LayoutFrame) -> T>;
    let is_close = Rc::new(is_close) as Rc<dyn Fn(T, T) -> bool>;
    let initial = initial_callback.clone();
    let state = use_hook(move || LayoutHookState {
        callback: Rc::new(RefCell::new(initial)),
        subscription: Rc::new(RefCell::new(None)),
    });
    *state.callback.borrow_mut() = initial_callback;

    let effect_state = state.clone();
    use_effect(move || {
        let callback = effect_state.callback.clone();
        let project = project.clone();
        let is_close = is_close.clone();
        let last = Rc::new(RefCell::new(None::<T>));
        let event_last = last.clone();
        let subscription = reference.subscribe(move |event| {
            let frame = match event {
                NativeElementEvent::Mounted(lease) => lease.layout_frame_px(),
                NativeElementEvent::Layout { frame, .. } => Some(frame),
                NativeElementEvent::Unmounted { .. } => {
                    event_last.borrow_mut().take();
                    None
                }
                NativeElementEvent::Visibility { .. } => None,
            };
            let Some(frame) = frame else {
                return;
            };
            let next = project(frame);
            let changed = !event_last
                .borrow()
                .is_some_and(|previous| is_close(previous, next));
            if changed {
                event_last.replace(Some(next));
                callback.borrow().clone()(next);
            }
        });
        effect_state.subscription.replace(Some(subscription));
    });

    let cleanup = state.subscription.clone();
    use_drop(move || {
        cleanup.borrow_mut().take();
    });
}

/// Observe the measured size of the element carrying `reference`.
///
/// The same handle must be assigned to that element's `native_ref` attribute.
#[track_caller]
pub fn use_layout_size(reference: NativeElementRef, on_change: impl Fn(LayoutSize) + 'static) {
    use_layout_event(reference, Rc::new(on_change), size_from_frame, size_close);
}

/// Observe the window-relative frame of the element carrying `reference`.
///
/// The same handle must be assigned to that element's `native_ref` attribute.
#[track_caller]
pub fn use_layout_frame(reference: NativeElementRef, on_change: impl Fn(LayoutFrame) + 'static) {
    use_layout_event(reference, Rc::new(on_change), |frame| frame, frame_close);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subpixel_layout_noise_is_ignored() {
        let first = LayoutFrame {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 40.0,
        };
        let noise = LayoutFrame {
            x: 10.25,
            y: 19.75,
            width: 100.25,
            height: 39.75,
        };
        assert!(frame_close(first, noise));
    }
}
