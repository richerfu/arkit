//! `use_overlay` — declarative floating and modal overlays.
//!
//! Overlays render their content declaratively: `show_*` publishes the content
//! `Element` on the `ArkHost`'s overlay-content signal, and `OverlayRoot`
//! (rendered once at the app root) mounts it as a full-screen `stack` subtree
//! on top of the app. `dismiss()` clears the signal. This keeps overlay content
//! inside the dioxus tree (no second VirtualDom, no imperative portal chrome),
//! so signals/hooks in the overlay continue to work.
//!
use std::cell::RefCell;
use std::rc::Rc;

use crate::layout::LayoutFrame;
use crate::node::{use_ark_host, ArkHost};

// Bring the ArkUI element descriptors (`stack`, `row`, `column`, ...) and
// event descriptors into scope for `rsx!`.
use arkit_prelude::*;

const STACK_ALIGN_CENTER: i32 = 4;

/// Modal presentation style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModalPresentation {
    #[default]
    CenteredDialog,
    RightSheet,
    BottomDrawer,
}

/// Spec for a modal overlay.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModalOverlaySpec {
    pub open: bool,
    pub presentation: ModalPresentation,
    pub dismiss_on_backdrop: bool,
    pub backdrop_color: u32,
    pub viewport_inset: f32,
}

impl Default for ModalOverlaySpec {
    fn default() -> Self {
        Self {
            open: false,
            presentation: ModalPresentation::CenteredDialog,
            dismiss_on_backdrop: true,
            backdrop_color: 0x80000000,
            viewport_inset: 16.0,
        }
    }
}

/// Internal overlay state. The content `Element` is published on the host's
/// overlay-content signal; this struct only tracks whether content is open.
struct OverlayState {
    host: ArkHost,
    open: bool,
}

impl OverlayState {
    fn new(host: ArkHost) -> Self {
        Self { host, open: false }
    }
}

/// Handle returned by [`use_overlay`]. Cloning shares the underlying state.
#[derive(Clone)]
pub struct OverlayApi {
    inner: Rc<RefCell<OverlayState>>,
}

impl OverlayApi {
    /// Show floating content as a full-screen subtree at `OverlayRoot`.
    /// Positioning and dismissal layers are part of the supplied Dioxus tree.
    pub fn show_floating(&self, content: impl FnOnce() -> Element + 'static) {
        self.set_content(content);
    }

    /// Show a modal overlay (centered dialog / right sheet / bottom drawer).
    pub fn show_modal(&self, spec: ModalOverlaySpec, content: impl FnOnce() -> Element + 'static) {
        self.show_modal_with_dismiss(spec, content, || {});
    }

    /// Show a modal overlay with an explicit dismiss callback. The callback is
    /// invoked for backdrop/outside clicks before the overlay signal is cleared.
    pub fn show_modal_with_dismiss(
        &self,
        spec: ModalOverlaySpec,
        content: impl FnOnce() -> Element + 'static,
        on_dismiss: impl Fn() + 'static,
    ) {
        let dismiss = {
            let overlay = self.clone();
            Rc::new(move || {
                on_dismiss();
                overlay.dismiss();
            }) as Rc<dyn Fn()>
        };
        let element = modal_overlay_layer(spec, content(), dismiss);
        self.set_element(element);
    }

    /// Dismiss the active overlay (clears the overlay-content signal).
    pub fn dismiss(&self) {
        let mut sig = self.inner.borrow().host.overlay_content();
        sig.set(None);
        let mut state = self.inner.borrow_mut();
        state.open = false;
    }

    /// Whether an overlay is currently open.
    pub fn is_open(&self) -> bool {
        self.inner.borrow().open
    }

    /// Current measured frame of the app-level overlay root, in physical
    /// pixels. Floating overlays use this to translate window-relative trigger
    /// frames into overlay-local coordinates.
    pub fn overlay_frame(&self) -> LayoutFrame {
        self.inner.borrow().host.overlay_frame_value()
    }

    /// Shared helper: render the content closure to an `Element` and publish it
    /// on the host's overlay-content signal, marking the overlay open.
    fn set_content(&self, content: impl FnOnce() -> Element + 'static) {
        let element = content();
        self.set_element(element);
    }

    fn set_element(&self, element: Element) {
        let mut sig = self.inner.borrow().host.overlay_content();
        sig.set(Some(element));
        let mut state = self.inner.borrow_mut();
        state.open = true;
    }
}

fn dismiss_if_allowed(spec: ModalOverlaySpec, dismiss: &Rc<dyn Fn()>) {
    if spec.dismiss_on_backdrop {
        dismiss();
    }
}

fn modal_overlay_layer(spec: ModalOverlaySpec, panel: Element, dismiss: Rc<dyn Fn()>) -> Element {
    let backdrop_dismiss = dismiss.clone();
    let overlay_panel = match spec.presentation {
        ModalPresentation::CenteredDialog => {
            let outside_dismiss = dismiss.clone();
            rsx! {
                stack {
                    percent_width: 1.0,
                    percent_height: 1.0,
                    alignment: STACK_ALIGN_CENTER,
                    padding_top: spec.viewport_inset,
                    padding_right: spec.viewport_inset,
                    padding_bottom: spec.viewport_inset,
                    padding_left: spec.viewport_inset,
                    onclick: move |evt| {
                        evt.stop_propagation();
                        dismiss_if_allowed(spec, &outside_dismiss);
                    },
                    stack {
                        clip: false,
                        onclick: move |evt| evt.stop_propagation(),
                        {panel}
                    }
                }
            }
        }
        ModalPresentation::RightSheet => {
            let outside_dismiss = dismiss.clone();
            rsx! {
                row {
                    percent_width: 1.0,
                    percent_height: 1.0,
                    justify_content: "end",
                    padding_top: spec.viewport_inset,
                    padding_right: spec.viewport_inset,
                    padding_bottom: spec.viewport_inset,
                    padding_left: spec.viewport_inset,
                    onclick: move |evt| {
                        evt.stop_propagation();
                        dismiss_if_allowed(spec, &outside_dismiss);
                    },
                    column {
                        percent_height: 1.0,
                        stack {
                            clip: false,
                            onclick: move |evt| evt.stop_propagation(),
                            {panel}
                        }
                    }
                }
            }
        }
        ModalPresentation::BottomDrawer => {
            let outside_dismiss = dismiss.clone();
            rsx! {
                column {
                    percent_width: 1.0,
                    percent_height: 1.0,
                    justify_content: "end",
                    padding_top: spec.viewport_inset,
                    padding_right: spec.viewport_inset,
                    padding_bottom: spec.viewport_inset,
                    padding_left: spec.viewport_inset,
                    onclick: move |evt| {
                        evt.stop_propagation();
                        dismiss_if_allowed(spec, &outside_dismiss);
                    },
                    stack {
                        clip: false,
                        onclick: move |evt| evt.stop_propagation(),
                        {panel}
                    }
                }
            }
        }
    };

    rsx! {
        stack {
            percent_width: 1.0,
            percent_height: 1.0,
            alignment: 0,
            clip: false,
            row {
                percent_width: 1.0,
                percent_height: 1.0,
                background_color: spec.backdrop_color,
                onclick: move |evt| {
                    evt.stop_propagation();
                    dismiss_if_allowed(spec, &backdrop_dismiss);
                },
            }
            {overlay_panel}
        }
    }
}

/// Create an [`OverlayApi`] bound to the current [`ArkHost`].
///
/// Requires an ancestor to have called [`crate::use_ark_host_provider`], and
/// the app root to render [`crate::OverlayRoot`] so the overlay content
/// actually mounts.
pub fn use_overlay() -> OverlayApi {
    let host = use_ark_host();
    use_hook(|| {
        let state = OverlayState::new(host.clone());
        OverlayApi {
            inner: Rc::new(RefCell::new(state)),
        }
    })
}
