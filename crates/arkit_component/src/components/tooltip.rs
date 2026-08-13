//! Tooltip — a small label that appears above a trigger on hover (and toggles
//! on tap for touch).
//!
//! Migrated from the legacy Elm builder API. The trigger opens the tooltip on
//! hover (`on_hover`) and toggles it on click; the panel renders through the app
//! root-projected portal so parent layout cannot clip it. Panel styling preserved:
//! `px-3 py-1.5`, `md` radius, 1px border, `popover` background,
//! `popover_foreground` text at native text-base size. Anchored above the
//! trigger.

use super::floating_layer::{FloatingAlign, FloatingPanelPlacement, FloatingSide};
use crate::style::*;
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
    let viewport = arkit_hooks::use_overlay_viewport();
    let trigger_ref = arkit_hooks::use_native_element_ref();
    let trigger_frame = use_signal(arkit_hooks::LayoutFrame::default);
    arkit_hooks::use_layout_frame(trigger_ref.clone(), move |frame| {
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

    let show_tooltip = EventHandler::new(move |_: ()| {
        if !current {
            set_open.call(true);
        }
    });

    let close_tooltip = EventHandler::new(move |_: ()| {
        if current {
            set_open.call(false);
        }
    });

    let placement = FloatingPanelPlacement::resolve(
        *trigger_frame.read(),
        viewport,
        panel_width,
        TOOLTIP_ESTIMATED_HEIGHT,
        FloatingSide::Top,
        FloatingAlign::Center,
        spacing::XXS,
    );

    rsx! {
        row {
            native_ref: trigger_ref,
            onclick: move |_| set_open.call(!current),
            on_hover: move |evt| {
                if evt.data().is_hovering {
                    show_tooltip.call(());
                } else {
                    close_tooltip.call(());
                }
            },
            {trigger}
        }
        if current {
            arkit_hooks::Portal {
                layer: arkit_hooks::OverlayLayer::Floating,
                {tooltip_overlay_content(theme, panel_width, placement, content)}
            }
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
        stack {
            width: "100%",
            height: "100%",
            hit_test_behavior: "none",
            row {
                position: format!("{left},{top}"),
                width: panel_width,
                align_items: "center",
                justify_content: "center",
                hit_test_behavior: "default",
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
