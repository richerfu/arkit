//! Declarative root projection for floating, modal, and transient content.

use crate::layout::LayoutFrame;
use crate::safe_area::{use_safe_area, use_window_metrics};
use arkit_prelude::*;

const STACK_ALIGN_CENTER: &str = "center";

/// Stable root projection planes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum OverlayLayer {
    #[default]
    Modal,
    Floating,
    Transient,
}

impl OverlayLayer {
    const fn name(self) -> &'static str {
        match self {
            Self::Modal => "modal",
            Self::Floating => "floating",
            Self::Transient => "transient",
        }
    }

    pub const fn z_index(self) -> i32 {
        match self {
            Self::Modal => 100,
            Self::Floating => 200,
            Self::Transient => 300,
        }
    }
}

/// Keep Dioxus ownership and context at the declaration site while projecting
/// the native subtree into a stable renderer-root layer.
#[component]
pub fn Portal(#[props(default)] layer: OverlayLayer, children: Element) -> Element {
    // Locals keep both attributes dynamic so the renderer receives them even
    // when the component body itself came from a static RSX template.
    let portal_layer = layer.name();
    let z_index = layer.z_index();
    rsx! {
        portal {
            portal_layer,
            width: "100%",
            height: "100%",
            alignment: "top-start",
            hit_test_behavior: "none",
            z_index,
            {children}
        }
    }
}

/// Modal presentation style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModalPresentation {
    #[default]
    CenteredDialog,
    RightSheet,
    BottomDrawer,
}

/// Full portal frame plus current safe insets.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct OverlayViewport {
    /// Portal frame in physical window pixels.
    pub frame: LayoutFrame,
    /// Visual safe-area insets in vp.
    pub safe_area: arkit_runtime::EdgeInsets,
    /// Physical-pixel to ArkUI-vp scale.
    pub scale: f32,
}

/// Read positioning geometry for a declarative [`Portal`].
#[track_caller]
pub fn use_overlay_viewport() -> OverlayViewport {
    let metrics = use_window_metrics();
    let content = metrics.content_rect;
    OverlayViewport {
        frame: LayoutFrame {
            x: content.left as f32,
            y: content.top as f32,
            width: content.width.max(0) as f32,
            height: content.height.max(0) as f32,
        },
        safe_area: metrics.safe_area,
        scale: if metrics.scale.is_finite() && metrics.scale > 0.0 {
            metrics.scale
        } else {
            1.0
        },
    }
}

/// Declarative modal shell with backdrop and safe-area-aware placement.
#[component]
pub fn ModalPortal(
    open: bool,
    #[props(default)] presentation: ModalPresentation,
    #[props(default = true)] dismiss_on_backdrop: bool,
    #[props(default = 0x80000000)] backdrop_color: u32,
    #[props(default = 16.0)] viewport_inset: f32,
    on_dismiss: EventHandler<()>,
    children: Element,
) -> Element {
    if !open {
        return rsx! {};
    }

    rsx! {
        Portal {
            layer: OverlayLayer::Modal,
            ModalOverlayLayer {
                presentation,
                dismiss_on_backdrop,
                backdrop_color,
                viewport_inset,
                on_dismiss,
                panel: children,
            }
        }
    }
}

#[derive(Clone, Props)]
struct ModalOverlayLayerProps {
    presentation: ModalPresentation,
    dismiss_on_backdrop: bool,
    backdrop_color: u32,
    viewport_inset: f32,
    on_dismiss: EventHandler<()>,
    panel: Element,
}

impl PartialEq for ModalOverlayLayerProps {
    fn eq(&self, _other: &Self) -> bool {
        // Element props can close over changing signals. Always reconcile the
        // declared modal subtree rather than snapshotting it outside the owner.
        false
    }
}

fn dismiss_if_allowed(allowed: bool, dismiss: EventHandler<()>) {
    if allowed {
        dismiss.call(());
    }
}

#[allow(non_snake_case)]
fn ModalOverlayLayer(props: ModalOverlayLayerProps) -> Element {
    let safe_area = use_safe_area();
    let inset_top = props.viewport_inset + safe_area.top;
    let inset_right = props.viewport_inset + safe_area.right;
    let inset_bottom = props.viewport_inset + safe_area.bottom;
    let inset_left = props.viewport_inset + safe_area.left;
    let dismiss_on_backdrop = props.dismiss_on_backdrop;
    let dismiss = props.on_dismiss;
    let panel = props.panel;

    let overlay_panel = match props.presentation {
        ModalPresentation::CenteredDialog => rsx! {
            stack {
                width: "100%",
                height: "100%",
                alignment: STACK_ALIGN_CENTER,
                padding_top: inset_top,
                padding_right: inset_right,
                padding_bottom: inset_bottom,
                padding_left: inset_left,
                onclick: move |event| {
                    event.stop_propagation();
                    dismiss_if_allowed(dismiss_on_backdrop, dismiss);
                },
                stack {
                    width: "100%",
                    clip: false,
                    onclick: move |event| event.stop_propagation(),
                    {panel}
                }
            }
        },
        ModalPresentation::RightSheet => rsx! {
            row {
                width: "100%",
                height: "100%",
                justify_content: "end",
                padding_top: inset_top,
                padding_right: inset_right,
                padding_bottom: inset_bottom,
                padding_left: inset_left,
                onclick: move |event| {
                    event.stop_propagation();
                    dismiss_if_allowed(dismiss_on_backdrop, dismiss);
                },
                column {
                    height: "100%",
                    stack {
                        clip: false,
                        onclick: move |event| event.stop_propagation(),
                        {panel}
                    }
                }
            }
        },
        ModalPresentation::BottomDrawer => rsx! {
            column {
                width: "100%",
                height: "100%",
                padding_top: inset_top,
                padding_right: inset_right,
                padding_bottom: 0.0,
                padding_left: inset_left,
                onclick: move |event| {
                    event.stop_propagation();
                    dismiss_if_allowed(dismiss_on_backdrop, dismiss);
                },
                row {
                    width: "100%",
                    layout_weight: 1.0,
                }
                column {
                    width: "100%",
                    clip: false,
                    onclick: move |event| event.stop_propagation(),
                    {panel}
                }
            }
        },
    };

    rsx! {
        stack {
            width: "100%",
            height: "100%",
            alignment: "top-start",
            clip: false,
            row {
                width: "100%",
                height: "100%",
                background_color: props.backdrop_color,
                onclick: move |event| {
                    event.stop_propagation();
                    dismiss_if_allowed(dismiss_on_backdrop, dismiss);
                },
            }
            {overlay_panel}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_layers_have_stable_projection_order() {
        assert!(OverlayLayer::Modal < OverlayLayer::Floating);
        assert!(OverlayLayer::Floating < OverlayLayer::Transient);
        assert_eq!(OverlayLayer::Modal.name(), "modal");
        assert_eq!(OverlayLayer::Transient.z_index(), 300);
    }
}
