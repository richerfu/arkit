//! Hover card — a floating card anchored beneath a trigger, shown on hover
//! (and toggled on tap for touch).
//!
//! Migrated from the legacy Elm builder API. The trigger opens the card on
//! hover (`on_hover`) and toggles it on click; the panel renders through the app
//! overlay root so parent layout cannot clip it. Panel styling preserved
//! (legacy `panel_surface`): default width `256` (Tailwind `w-64`),
//! `spacing::LG` padding, `md` radius, 1px border, `popover`/`border` tokens,
//! small outer shadow, start-aligned content. Anchored below the trigger.

use super::floating_layer::{
    FloatingAlign, FloatingPanelPlacement, FloatingSide, FLOATING_BACKDROP, SHADOW_SM,
};
use crate::theme::*;
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

    let show_overlay = overlay.clone();
    let show_card = EventHandler::new(
        move |pointer: Option<dioxus_elements::event::PointerPayload>| {
            if current {
                return;
            }
            set_open.call(true);
            let panel = children.clone();
            let frame = *trigger_frame.read();
            let overlay_frame = show_overlay.overlay_frame();
            let placement = if let Some(placement) = pointer.and_then(|pointer| {
                FloatingPanelPlacement::from_pointer(
                    pointer,
                    overlay_frame,
                    panel_width,
                    HOVER_CARD_ESTIMATED_HEIGHT,
                    FloatingSide::Bottom,
                    FloatingAlign::Center,
                    spacing::XXS,
                )
            }) {
                placement
            } else if frame.is_measured() {
                FloatingPanelPlacement::from_trigger(
                    frame,
                    overlay_frame,
                    panel_width,
                    HOVER_CARD_ESTIMATED_HEIGHT,
                    FloatingSide::Bottom,
                    FloatingAlign::Center,
                    spacing::XXS,
                )
            } else {
                FloatingPanelPlacement::fallback()
            };
            let dismiss_overlay = show_overlay.clone();
            let dismiss = EventHandler::new(move |_: ()| {
                set_open.call(false);
                dismiss_overlay.dismiss();
            });
            show_overlay.show_floating(move || {
                hover_card_overlay_content(theme, panel_width, placement, dismiss, panel)
            });
        },
    );

    let leave_overlay = overlay.clone();
    let close_card = EventHandler::new(move |_: ()| {
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
            show_card.call(pointer);
        }
    };

    rsx! {
        row {
            onclick: move |evt: dioxus_core::Event<dioxus_elements::event::ClickData>| {
                toggle(evt.data().pointer);
            },
            on_hover: move |evt| {
                if evt.data().is_hovering {
                    show_card.call(None);
                } else {
                    close_card.call(());
                }
            },
            {trigger}
        }
    }
}

fn hover_card_overlay_content(
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
            background_color: FLOATING_BACKDROP,
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
