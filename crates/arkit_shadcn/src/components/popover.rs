//! Popover — a floating panel anchored to a trigger, toggled by tapping the
//! trigger and dismissed by tapping outside.
//!
//! Migrated from the legacy Elm builder API. The trigger toggles the open
//! state (click mode). The panel renders through the app overlay root so it is
//! not clipped by the trigger's parent layout. Panel styling preserved: default
//! width `288` (Tailwind `w-72`), `spacing::LG` padding, `md` radius, 1px
//! border, `popover`/`border` tokens, small outer shadow, start-aligned
//! content.

use super::floating_layer::{
    FloatingAlign, FloatingPanelPlacement, FloatingSide, FLOATING_CAPTURE_COLOR, HIT_TEST_DEFAULT,
    SHADOW_SM,
};
use crate::theme::*;
use arkit_prelude::*;
use dioxus_core_macro::component;

const POPOVER_DEFAULT_WIDTH: f32 = 288.0;
const POPOVER_ESTIMATED_HEIGHT: f32 = 132.0;

/// Popover floating panel.
#[component]
pub fn Popover(
    trigger: Element,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    on_open_change: Option<EventHandler<bool>>,
    width: Option<f32>,
    children: Element,
) -> Element {
    let theme = use_theme();
    let overlay = arkit_hooks::use_overlay();
    let trigger_frame = use_signal(arkit_hooks::LayoutFrame::default);
    arkit_hooks::use_layout_frame(move |frame| {
        let mut trigger_frame = trigger_frame;
        trigger_frame.set(frame);
    });
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let panel_width = width.unwrap_or(POPOVER_DEFAULT_WIDTH);

    let set_open = EventHandler::new(move |next: bool| {
        if !controlled {
            internal.set(next);
        }
        if let Some(handler) = on_open_change {
            handler.call(next);
        }
        if !next {
            if let Some(handler) = on_close {
                handler.call(());
            }
        }
    });

    let toggle = move |pointer: Option<dioxus_elements::event::PointerPayload>| {
        if current {
            set_open.call(false);
            overlay.dismiss();
        } else {
            set_open.call(true);
            let panel = children.clone();
            let frame = *trigger_frame.read();
            let viewport = overlay.viewport();
            let placement = if let Some(placement) = pointer.and_then(|pointer| {
                FloatingPanelPlacement::from_pointer(
                    pointer,
                    viewport,
                    panel_width,
                    POPOVER_ESTIMATED_HEIGHT,
                    FloatingSide::Bottom,
                    FloatingAlign::Center,
                    spacing::XXS,
                )
            }) {
                placement
            } else if frame.is_measured() {
                FloatingPanelPlacement::from_trigger(
                    frame,
                    viewport,
                    panel_width,
                    POPOVER_ESTIMATED_HEIGHT,
                    FloatingSide::Bottom,
                    FloatingAlign::Center,
                    spacing::XXS,
                )
            } else {
                FloatingPanelPlacement::fallback(viewport)
            };
            let dismiss_overlay = overlay.clone();
            let dismiss = EventHandler::new(move |_: ()| {
                set_open.call(false);
                dismiss_overlay.dismiss();
            });
            overlay.show_floating(move || {
                popover_overlay_content(theme, panel_width, placement, dismiss, panel)
            });
        }
    };

    rsx! {
        row {
            onclick: move |evt: dioxus_core::Event<dioxus_elements::event::ClickData>| {
                toggle(evt.data().pointer);
            },
            {trigger}
        }
    }
}

fn popover_overlay_content(
    theme: Theme,
    panel_width: f32,
    placement: FloatingPanelPlacement,
    on_dismiss: EventHandler<()>,
    children: Element,
) -> Element {
    let top = placement.y.max(0.0);
    let left = placement.x.max(0.0);
    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            align_items: "start",
            padding_top: top,
            background_color: FLOATING_CAPTURE_COLOR,
            hit_test_behavior: HIT_TEST_DEFAULT,
            onclick: move |_| on_dismiss.call(()),
            row {
                percent_width: 1.0,
                align_items: "start",
                arkit_animation::MountTransition {
                    preset: Some(arkit_animation::TransitionPreset::SlideUp),
                    duration_ms: Some(140),
                    column {
                        onclick: move |evt| evt.stop_propagation(),
                        margin_left: left,
                        width: panel_width,
                        align_items: "start",
                        padding: spacing::LG,
                        border_radius: theme.radii.md,
                        border_width: 1.0,
                        border_color: theme.colors.border,
                        background_color: theme.colors.popover,
                        shadow: SHADOW_SM,
                        {children}
                    }
                }
            }
        }
    }
}
