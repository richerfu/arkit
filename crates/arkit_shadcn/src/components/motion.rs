//! Shared enter/exit motion for shadcn overlays.
//!
//! Overlay components keep their portal mounted through
//! [`arkit_animation::use_presence_visibility`] so hide timelines can finish
//! before the native subtree is dropped.

use arkit_animation::{
    use_presence_visibility, PresencePhase, PresenceTransition, TransitionPreset,
};
use arkit_prelude::*;

use super::floating_layer::FloatingSide;

pub(crate) const OVERLAY_ENTER_MS: i32 = 220;
pub(crate) const OVERLAY_EXIT_MS: i32 = 160;
pub(crate) const FLOATING_ENTER_MS: i32 = 160;
pub(crate) const FLOATING_EXIT_MS: i32 = 120;
pub(crate) const TOAST_ENTER_MS: i32 = 380;
pub(crate) const TOAST_EXIT_MS: i32 = 260;
pub(crate) const TOAST_STACK_MS: i32 = 400;
pub(crate) const ACCORDION_MS: i32 = 160;
pub(crate) const TOAST_DISTANCE: f32 = 80.0;
pub(crate) const SHEET_DISTANCE: f32 = 56.0;
pub(crate) const FLOATING_DISTANCE: f32 = 10.0;
pub(crate) const ACCORDION_DISTANCE: f32 = 8.0;

pub(crate) fn slide_in_from(side: FloatingSide) -> TransitionPreset {
    match side {
        FloatingSide::Top => TransitionPreset::SlideDown,
        FloatingSide::Bottom => TransitionPreset::SlideUp,
        FloatingSide::Left => TransitionPreset::SlideRight,
        FloatingSide::Right => TransitionPreset::SlideLeft,
    }
}

/// Keep a portaled overlay mounted through its hide timeline.
#[component]
pub(crate) fn OverlayPresence(
    open: bool,
    #[props(default)] preset: Option<TransitionPreset>,
    #[props(default)] duration_ms: Option<i32>,
    #[props(default)] exit_duration_ms: Option<i32>,
    #[props(default)] distance: Option<f32>,
    #[props(default)] fill: Option<bool>,
    #[props(default)] layer: Option<arkit_hooks::OverlayLayer>,
    children: Element,
) -> Element {
    let visibility = use_presence_visibility(open);
    if !visibility.mounted {
        return rsx! {};
    }
    let body = rsx! {
        PresenceTransition {
            phase: visibility.phase,
            on_terminal: visibility.on_terminal,
            preset,
            duration_ms,
            exit_duration_ms,
            distance,
            fill,
            {children}
        }
    };
    match layer {
        Some(layer) => rsx! {
            arkit_hooks::Portal {
                layer,
                {body}
            }
        },
        None => body,
    }
}

/// Modal portal that stays up until the panel hide timeline settles.
///
/// Backdrop and panel share the same presence phase so they appear and
/// disappear together. The panel wrapper is content-sized — `fill` would
/// stretch a bottom sheet to the full overlay and pin it to the top.
#[component]
pub(crate) fn AnimatedModal(
    open: bool,
    presentation: arkit_hooks::ModalPresentation,
    on_dismiss: EventHandler<()>,
    children: Element,
    #[props(default = true)] dismiss_on_backdrop: bool,
    #[props(default = 0x80000000)] backdrop_color: u32,
    #[props(default = 16.0)] viewport_inset: f32,
    #[props(default)] preset: Option<TransitionPreset>,
    #[props(default)] duration_ms: Option<i32>,
    #[props(default)] exit_duration_ms: Option<i32>,
    #[props(default)] distance: Option<f32>,
) -> Element {
    let visibility = use_presence_visibility(open);
    let safe_area = arkit_hooks::use_safe_area();
    if !visibility.mounted {
        return rsx! {};
    }
    let inset_top = viewport_inset + safe_area.top;
    let inset_right = viewport_inset + safe_area.right;
    let inset_bottom = viewport_inset + safe_area.bottom;
    let inset_left = viewport_inset + safe_area.left;
    let allow_dismiss = dismiss_on_backdrop;
    let dismiss = on_dismiss;
    let placed = match presentation {
        arkit_hooks::ModalPresentation::CenteredDialog => rsx! {
            stack {
                width: "100%",
                height: "100%",
                alignment: "center",
                padding_top: inset_top,
                padding_right: inset_right,
                padding_bottom: inset_bottom,
                padding_left: inset_left,
                hit_test_behavior: "transparent",
                stack {
                    width: "100%",
                    clip: false,
                    hit_test_behavior: "transparent",
                    PresenceTransition {
                        phase: visibility.phase,
                        on_terminal: visibility.on_terminal,
                        preset,
                        duration_ms,
                        exit_duration_ms,
                        distance,
                        {children}
                    }
                }
            }
        },
        arkit_hooks::ModalPresentation::RightSheet => rsx! {
            row {
                width: "100%",
                height: "100%",
                justify_content: "end",
                padding_top: inset_top,
                padding_right: inset_right,
                padding_bottom: inset_bottom,
                padding_left: inset_left,
                hit_test_behavior: "transparent",
                column {
                    height: "100%",
                    hit_test_behavior: "transparent",
                    PresenceTransition {
                        phase: visibility.phase,
                        on_terminal: visibility.on_terminal,
                        preset,
                        duration_ms,
                        exit_duration_ms,
                        distance,
                        {children}
                    }
                }
            }
        },
        arkit_hooks::ModalPresentation::BottomDrawer => rsx! {
            column {
                width: "100%",
                height: "100%",
                padding_top: inset_top,
                padding_right: inset_right,
                padding_bottom: 0.0,
                padding_left: inset_left,
                hit_test_behavior: "transparent",
                row {
                    width: "100%",
                    layout_weight: 1.0,
                    hit_test_behavior: "none",
                }
                column {
                    width: "100%",
                    clip: false,
                    hit_test_behavior: "transparent",
                    PresenceTransition {
                        phase: visibility.phase,
                        on_terminal: visibility.on_terminal,
                        preset,
                        duration_ms,
                        exit_duration_ms,
                        distance,
                        {children}
                    }
                }
            }
        },
    };
    rsx! {
        arkit_hooks::Portal {
            layer: arkit_hooks::OverlayLayer::Modal,
            stack {
                width: "100%",
                height: "100%",
                alignment: "top-start",
                clip: false,
                PresenceTransition {
                    phase: visibility.phase,
                    preset: Some(TransitionPreset::Fade),
                    duration_ms,
                    exit_duration_ms,
                    fill: Some(true),
                    row {
                        width: "100%",
                        height: "100%",
                        background_color: backdrop_color,
                        hit_test_behavior: "default",
                        onclick: move |event| {
                            event.stop_propagation();
                            if allow_dismiss {
                                dismiss.call(());
                            }
                        },
                    }
                }
                {placed}
            }
        }
    }
}

/// Inline expand/collapse with a matched hide timeline.
///
/// Leaving content is clipped out of flow immediately so a parent popup
/// shrinks on the same frame the user collapses, instead of staying tall
/// until the exit timeline settles.
#[component]
pub(crate) fn ExpandPresence(
    open: bool,
    children: Element,
    #[props(default)] preset: Option<TransitionPreset>,
    #[props(default)] duration_ms: Option<i32>,
    #[props(default)] distance: Option<f32>,
) -> Element {
    let visibility = use_presence_visibility(open);
    if !visibility.mounted {
        return rsx! {};
    }
    let leaving = visibility.phase == PresencePhase::Leaving;
    rsx! {
        column {
            width: "100%",
            constraint_size: if leaving {
                "0,100000,0,0"
            } else {
                "0,100000,0,100000"
            },
            clip: true,
            hit_test_behavior: if leaving { "none" } else { "transparent" },
            PresenceTransition {
                phase: visibility.phase,
                on_terminal: visibility.on_terminal,
                preset: preset.or(Some(TransitionPreset::SlideUp)),
                duration_ms: duration_ms.or(Some(ACCORDION_MS)),
                exit_duration_ms: duration_ms.or(Some(ACCORDION_MS)),
                distance: distance.or(Some(ACCORDION_DISTANCE)),
                {children}
            }
        }
    }
}
