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
use crate::safe_area::use_safe_area;

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
    token: u64,
    window_metrics: Option<arkit_runtime::WindowMetricsHandle>,
    open: bool,
    mounted: bool,
}

impl OverlayState {
    fn new(host: ArkHost, window_metrics: Option<arkit_runtime::WindowMetricsHandle>) -> Self {
        let token = host.allocate_overlay_token();
        Self {
            host,
            token,
            window_metrics,
            open: false,
            mounted: true,
        }
    }
}

/// Full overlay frame plus the current effective visual safe insets.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct OverlayViewport {
    pub frame: LayoutFrame,
    pub safe_area: arkit_runtime::EdgeInsets,
}

/// Handle returned by [`use_overlay`]. Cloning shares the underlying state.
#[derive(Clone)]
pub struct OverlayApi {
    inner: Rc<RefCell<OverlayState>>,
}

impl OverlayApi {
    /// Show floating content as a full-screen subtree at `OverlayRoot`.
    /// Positioning and dismissal layers are part of the supplied Dioxus tree.
    /// The portal root itself uses ArkUI `HitTestMode::None`, so blank space is
    /// pass-through by default. A floating subtree that needs outside-click
    /// dismissal must add an explicit blocking hit plane; otherwise a single
    /// click can both dismiss the surface and activate a retained route below
    /// it. Masked/modal surfaces follow the same blocking rule.
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
        let (host, token, mounted) = {
            let state = self.inner.borrow();
            (state.host.clone(), state.token, state.mounted)
        };
        if !mounted {
            return;
        }
        host.dismiss_overlay(token);
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

    /// Current positioning viewport for floating content.
    pub fn viewport(&self) -> OverlayViewport {
        let state = self.inner.borrow();
        OverlayViewport {
            frame: state.host.overlay_frame_value(),
            safe_area: state
                .window_metrics
                .as_ref()
                .map(|metrics| metrics.get().safe_area)
                .unwrap_or_default(),
        }
    }

    /// Shared helper: render the content closure to an `Element` and publish it
    /// on the host's overlay-content signal, marking the overlay open.
    fn set_content(&self, content: impl FnOnce() -> Element + 'static) {
        let element = content();
        self.set_element(element);
    }

    fn set_element(&self, element: Element) {
        let (host, token, mounted) = {
            let state = self.inner.borrow();
            (state.host.clone(), state.token, state.mounted)
        };
        if !mounted {
            return;
        }
        host.set_overlay(token, element);
        let mut state = self.inner.borrow_mut();
        state.open = true;
    }

    fn dispose(&self) {
        let (host, token, mounted) = {
            let state = self.inner.borrow();
            (state.host.clone(), state.token, state.mounted)
        };
        if mounted {
            host.dismiss_overlay(token);
        }
        let mut state = self.inner.borrow_mut();
        state.open = false;
        state.mounted = false;
    }
}

fn dismiss_if_allowed(spec: ModalOverlaySpec, dismiss: &Rc<dyn Fn()>) {
    if spec.dismiss_on_backdrop {
        dismiss();
    }
}

fn modal_overlay_layer(spec: ModalOverlaySpec, panel: Element, dismiss: Rc<dyn Fn()>) -> Element {
    rsx! {
        ModalOverlayLayer {
            spec,
            panel,
            dismiss,
        }
    }
}

#[derive(Clone, Props)]
struct ModalOverlayLayerProps {
    spec: ModalOverlaySpec,
    panel: Element,
    dismiss: Rc<dyn Fn()>,
}

impl PartialEq for ModalOverlayLayerProps {
    fn eq(&self, _other: &Self) -> bool {
        // Modal content may close over reactive business state. Rebuilding it
        // also lets safe-area changes update panel constraints immediately.
        false
    }
}

#[allow(non_snake_case)]
fn ModalOverlayLayer(props: ModalOverlayLayerProps) -> Element {
    let ModalOverlayLayerProps {
        spec,
        panel,
        dismiss,
    } = props;
    let safe_area = use_safe_area();
    let inset_top = spec.viewport_inset + safe_area.top;
    let inset_right = spec.viewport_inset + safe_area.right;
    let inset_bottom = spec.viewport_inset + safe_area.bottom;
    let inset_left = spec.viewport_inset + safe_area.left;
    let backdrop_dismiss = dismiss.clone();
    let overlay_panel = match spec.presentation {
        ModalPresentation::CenteredDialog => {
            let outside_dismiss = dismiss.clone();
            rsx! {
                stack {
                    percent_width: 1.0,
                    percent_height: 1.0,
                    alignment: STACK_ALIGN_CENTER,
                    padding_top: inset_top,
                    padding_right: inset_right,
                    padding_bottom: inset_bottom,
                    padding_left: inset_left,
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
                    padding_top: inset_top,
                    padding_right: inset_right,
                    padding_bottom: inset_bottom,
                    padding_left: inset_left,
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
                    padding_top: inset_top,
                    padding_right: inset_right,
                    // Bottom-anchored surfaces own their internal safe-area
                    // padding so their background reaches the screen edge.
                    padding_bottom: 0.0,
                    padding_left: inset_left,
                    onclick: move |evt| {
                        evt.stop_propagation();
                        dismiss_if_allowed(spec, &outside_dismiss);
                    },
                    // API 24 does not reliably honor bottom alignment on
                    // full-height Column/Stack nodes. A weighted spacer uses
                    // the measured remaining height and pins the intrinsic
                    // sheet to the bottom without needing to know its height.
                    row {
                        percent_width: 1.0,
                        layout_weight: 1.0,
                    }
                    column {
                        percent_width: 1.0,
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
    let window_metrics = dioxus_core::try_consume_context::<arkit_runtime::WindowMetricsHandle>();
    let overlay = use_hook(|| {
        let state = OverlayState::new(host.clone(), window_metrics.clone());
        OverlayApi {
            inner: Rc::new(RefCell::new(state)),
        }
    });
    let cleanup = overlay.clone();
    use_drop(move || cleanup.dispose());
    overlay
}
