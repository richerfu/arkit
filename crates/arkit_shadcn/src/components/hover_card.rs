//! Hover card — a floating card anchored beneath a trigger, shown on hover
//! (and toggled on tap for touch).
//!
//! Migrated from the legacy Elm builder API. The trigger opens the card on
//! hover (`on_hover`) and toggles it on click; the panel renders through the app
//! root-projected portal so parent layout cannot clip it. Panel styling preserved
//! (legacy `panel_surface`): default width `256` (Tailwind `w-64`),
//! `spacing::LG` padding, `md` radius, 1px border, `popover`/`border` tokens,
//! small outer shadow, start-aligned content. Anchored below the trigger.

use super::floating_layer::{
    trigger_frame_for_anchor, FloatingAlign, FloatingPanelPlacement, FloatingSide,
};
use super::motion::{OverlayPresence, FLOATING_ENTER_MS, FLOATING_EXIT_MS};
use super::popup_shadow::{PopupShadow, PopupShadowKind};
use crate::theme::*;
use arkit_prelude::*;
use dioxus_core_macro::component;

const HOVER_CARD_DEFAULT_WIDTH: f32 = 256.0;
const HOVER_CARD_ESTIMATED_HEIGHT: f32 = 132.0;

/// Hover/tap hover card.
#[component]
pub fn HoverCard(
    trigger: Element,
    #[props(default)] accessibility_label: Option<String>,
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
    let trigger_frame = use_signal(arkit_arkui::LayoutFramePx::default);
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
    let panel_width = super::panel_viewport::bounded_panel_width(
        viewport,
        width.unwrap_or(HOVER_CARD_DEFAULT_WIDTH),
    );

    let set_open = EventHandler::new(move |next: bool| {
        if next == current {
            return;
        }
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

    let surface = super::hover_surface::use_hover_surface(set_open);
    let trigger_hover = surface.clone();
    let trigger_focus = surface.clone();
    let trigger_blur = surface.clone();
    let trigger_click = surface.clone();
    let trigger_key = surface.clone();

    let frame = if current {
        trigger_frame_for_anchor(&trigger_ref, *trigger_frame.read())
    } else {
        *trigger_frame.read()
    };
    let placement = FloatingPanelPlacement::resolve(
        frame,
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
            accessibility_role: "button",
            accessibility_text: if let Some(label) = accessibility_label { label },
            accessibility_description: if current { "expanded" } else { "collapsed" },
            accessibility_group: true,
            accessibility_actions: "click",
            accessibility_selected: current,
            focusable: true,
            focus_on_touch: true,
            key_capture: "enter space escape",
            onclick: move |_| trigger_click.toggle_pin(current),
            onfocusin: move |_| trigger_focus.trigger_focus(true),
            onfocusout: move |_| trigger_blur.trigger_focus(false),
            onkey: move |event| {
                if event.data().is_down() && event.data().key == dioxus_elements::event::KeyboardKey::Escape {
                    event.stop_propagation();
                    trigger_key.dismiss();
                } else if event.data().activates() {
                    event.stop_propagation();
                    trigger_key.toggle_pin(current);
                }
            },
            onhover: move |event| trigger_hover.trigger_hover(event.data().is_hovering),
            {trigger}
        }
        OverlayPresence {
            open: current,
            preset: Some(arkit_animation::TransitionPreset::Fade),
            duration_ms: Some(FLOATING_ENTER_MS),
            exit_duration_ms: Some(FLOATING_EXIT_MS),
            fill: Some(true),
            layer: Some(arkit_hooks::OverlayLayer::Floating),
            {hover_card_overlay_content(theme, panel_width, placement, children, surface)}
        }
    }
}

fn hover_card_overlay_content(
    theme: Theme,
    panel_width: f32,
    placement: FloatingPanelPlacement,
    children: Element,
    surface: super::hover_surface::HoverSurface,
) -> Element {
    let viewport = arkit_hooks::use_overlay_viewport();
    let max_height = (super::panel_viewport::panel_available_height(viewport, placement.y)
        - spacing::LG * 2.0)
        .max(1.0);
    let top = placement.y.max(0.0);
    let left = placement.x.max(0.0);
    let pinned = surface.pinned();
    let outside = surface.clone();
    let panel_hover = surface.clone();
    let panel_focus = surface.clone();
    let panel_blur = surface.clone();
    let panel_key = surface.clone();
    rsx! {
        stack {
            width: "100%",
            height: "100%",
            background_color: if pinned { super::floating_layer::FLOATING_CAPTURE_COLOR } else { 0x00000000 },
            hit_test_behavior: if pinned { "default" } else { "none" },
            onclick: move |_| { if pinned { outside.dismiss(); } },
            PopupShadow {
                position: Some(format!("{left},{top}")),
                width: panel_width,
                radius: theme.radii.md,
                background: theme.colors.popover,
                kind: PopupShadowKind::Md,
                column {
                    accessibility_role: "group",
                    focus_scope: pinned,
                    key_capture: "escape",
                    onhover: move |event| panel_hover.panel_hover(event.data().is_hovering),
                    onfocusin: move |_| panel_focus.panel_focus(true),
                    onfocusout: move |_| panel_blur.panel_focus(false),
                    onkey: move |event| {
                        if event.data().is_down() && event.data().key == dioxus_elements::event::KeyboardKey::Escape { event.stop_propagation(); panel_key.dismiss(); }
                    },
                    onclick: move |event| event.stop_propagation(),
                    width: panel_width,
                    align_items: "start",
                    hit_test_behavior: "default",
                    padding: spacing::LG,
                    border_radius: theme.radii.md,
                    border_width: 1.0,
                    border_color: theme.colors.border,
                    background_color: theme.colors.popover,
                    clip: false,
                    super::panel_viewport::PanelViewport { max_height, {children} }
                }
            }
        }
    }
}
