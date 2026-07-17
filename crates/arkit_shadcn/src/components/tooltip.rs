//! Tooltip — a small label that appears above a trigger on hover (and toggles
//! on tap for touch).
//!
//! Migrated from the legacy Elm builder API. The trigger opens the tooltip on
//! hover (`on_hover`) and toggles it on click; the panel renders through the app
//! overlay root so parent layout cannot clip it. Panel styling preserved:
//! `px-3 py-1.5`, `md` radius, 1px border, `popover` background,
//! `popover_foreground` text at native text-base size. Anchored above the
//! trigger.

use super::floating_layer::{
    FloatingAlign, FloatingPanelPlacement, FloatingSide, HIT_TEST_DEFAULT, HIT_TEST_NONE,
};
use crate::theme::*;
use arkit_prelude::*;
use dioxus_core_macro::component;

const TOOLTIP_ESTIMATED_HEIGHT: f32 = 36.0;

/// Hover/tap tooltip.
#[component]
pub fn Tooltip(
    trigger: Element,
    content: String,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    on_open_change: Option<EventHandler<bool>>,
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
    let panel_width = ((content.chars().count() as f32 * 7.0) + 24.0).clamp(80.0, 240.0);

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

    let show_overlay = overlay.clone();
    let show_tooltip = EventHandler::new(
        move |pointer: Option<dioxus_elements::event::PointerPayload>| {
            if current {
                return;
            }
            set_open.call(true);
            let label = content.clone();
            let frame = *trigger_frame.read();
            let viewport = show_overlay.viewport();
            let placement = if let Some(placement) = pointer.and_then(|pointer| {
                FloatingPanelPlacement::from_pointer(
                    pointer,
                    viewport,
                    panel_width,
                    TOOLTIP_ESTIMATED_HEIGHT,
                    FloatingSide::Top,
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
                    TOOLTIP_ESTIMATED_HEIGHT,
                    FloatingSide::Top,
                    FloatingAlign::Center,
                    spacing::XXS,
                )
            } else {
                FloatingPanelPlacement::fallback(viewport)
            };
            show_overlay.show_floating(move || {
                tooltip_overlay_content(theme, panel_width, placement, label)
            });
        },
    );

    let leave_overlay = overlay.clone();
    let close_tooltip = EventHandler::new(move |_: ()| {
        if !current {
            return;
        }
        set_open.call(false);
        leave_overlay.dismiss();
    });

    let toggle_overlay = overlay.clone();
    let toggle = move |pointer: Option<dioxus_elements::event::PointerPayload>| {
        if current {
            set_open.call(false);
            toggle_overlay.dismiss();
        } else {
            show_tooltip.call(pointer);
        }
    };

    rsx! {
        row {
            onclick: move |evt: dioxus_core::Event<dioxus_elements::event::ClickData>| {
                toggle(evt.data().pointer);
            },
            on_hover: move |evt| {
                if evt.data().is_hovering {
                    show_tooltip.call(None);
                } else {
                    close_tooltip.call(());
                }
            },
            {trigger}
        }
    }
}

fn tooltip_overlay_content(
    theme: Theme,
    panel_width: f32,
    placement: FloatingPanelPlacement,
    content: String,
) -> Element {
    let top = placement.y.max(0.0);
    let left = placement.x.max(0.0);
    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            align_items: "start",
            padding_top: top,
            hit_test_behavior: HIT_TEST_NONE,
            row {
                percent_width: 1.0,
                align_items: "start",
                hit_test_behavior: HIT_TEST_NONE,
                arkit_animation::MountTransition {
                    preset: Some(arkit_animation::TransitionPreset::SlideDown),
                    duration_ms: Some(120),
                    row {
                        onclick: move |evt| evt.stop_propagation(),
                        margin_left: left,
                        width: panel_width,
                        align_items: "center",
                        justify_content: "center",
                        hit_test_behavior: HIT_TEST_DEFAULT,
                        padding_top: 6.0,
                        padding_right: 12.0,
                        padding_bottom: 6.0,
                        padding_left: 12.0,
                        border_radius: theme.radii.md,
                        border_width: 1.0,
                        border_color: theme.colors.border,
                        background_color: theme.colors.popover,
                        shadow: super::floating_layer::SHADOW_SM,
                        text {
                            content: content,
                            font_size: typography::MD,
                            font_color: theme.colors.popover_foreground,
                            line_height: 20.0,
                            max_lines: 1,
                        }
                    }
                }
            }
        }
    }
}
