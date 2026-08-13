//! Hover card — a floating card anchored beneath a trigger, shown on hover
//! (and toggled on tap for touch).
//!
//! Migrated from the legacy Elm builder API. The trigger opens the card on
//! hover (`on_hover`) and toggles it on click; the panel renders through the app
//! root-projected portal so parent layout cannot clip it. Panel styling preserved
//! (legacy `panel_surface`): default width `256` (Tailwind `w-64`),
//! `spacing::LG` padding, `md` radius, 1px border, `popover`/`border` tokens,
//! small outer shadow, start-aligned content. Anchored below the trigger.

use super::floating_layer::{FloatingAlign, FloatingPanelPlacement, FloatingSide};
use crate::style::*;
use arkit_prelude::*;
use dioxus_core_macro::component;

const HOVER_CARD_DEFAULT_WIDTH: f32 = 256.0;
const HOVER_CARD_ESTIMATED_HEIGHT: f32 = 132.0;

/// Hover/tap hover card.
#[component]
pub fn HoverCard(
    trigger: Element,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    on_open_change: Option<EventHandler<bool>>,
    width: Option<f32>,
    children: Element,
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
    let panel_width = width.unwrap_or(HOVER_CARD_DEFAULT_WIDTH);

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

    let show_card = EventHandler::new(move |_: ()| {
        if !current {
            set_open.call(true);
        }
    });

    let close_card = EventHandler::new(move |_: ()| {
        if current {
            set_open.call(false);
        }
    });

    let placement = FloatingPanelPlacement::resolve(
        *trigger_frame.read(),
        viewport,
        panel_width,
        HOVER_CARD_ESTIMATED_HEIGHT,
        FloatingSide::Bottom,
        FloatingAlign::Center,
        spacing::XXS,
    );

    rsx! {
        row {
            native_ref: trigger_ref,
            onclick: move |_| set_open.call(!current),
            on_hover: move |evt| {
                if evt.data().is_hovering {
                    show_card.call(());
                } else {
                    close_card.call(());
                }
            },
            {trigger}
        }
        if current {
            arkit_hooks::Portal {
                layer: arkit_hooks::OverlayLayer::Floating,
                {hover_card_overlay_content(theme, panel_width, placement, children)}
            }
        }
    }
}

fn hover_card_overlay_content(
    theme: Theme,
    panel_width: f32,
    placement: FloatingPanelPlacement,
    children: Element,
) -> Element {
    let top = placement.y.max(0.0);
    let left = placement.x.max(0.0);
    rsx! {
        stack {
            width: "100%",
            height: "100%",
            hit_test_behavior: "none",
            column {
                position: format!("{left},{top}"),
                width: panel_width,
                align_items: "start",
                hit_test_behavior: "default",
                padding: spacing::LG,
                border_radius: theme.radii.md,
                border_width: 1.0,
                border_color: theme.colors.border,
                background_color: theme.colors.popover,
                shadow: "sm",
                {children}
            }
        }
    }
}
