//! DropdownMenu — trigger + portal dropdown of entries.
//!
//! Ported from the legacy Elm builder `dropdown_menu.rs`. Menu panels render
//! through root-projected portals instead of inline so they are not clipped by
//! the trigger's parent layout and trigger children remain ordinary shadcn
//! buttons.

use crate::components::menu_common::{
    menu_closed_panel_height, menu_overlay_content, MenuEntry, MenuOverlayPlacement, MenuStyle,
};
use crate::components::motion::{
    OverlayPresence, FLOATING_DISTANCE, FLOATING_ENTER_MS, FLOATING_EXIT_MS,
};
use crate::theme::*;
use arkit_prelude::*;

const MENU_PANEL_WIDTH: f32 = 224.0;

pub type DropdownMenuEntry = MenuEntry;

#[component]
pub fn DropdownMenu(
    items: Vec<DropdownMenuEntry>,
    children: Element,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<EventHandler<bool>>,
    trigger_capture: Option<bool>,
    #[props(default)] width: Option<f32>,
) -> Element {
    let _ = trigger_capture;
    let theme = use_theme();
    let viewport = arkit_hooks::use_overlay_viewport();
    let trigger_ref = arkit_hooks::use_native_element_ref();
    let trigger_frame = use_signal(arkit_hooks::LayoutFrame::default);
    arkit_hooks::use_layout_frame(trigger_ref.clone(), move |frame| {
        let mut trigger_frame = trigger_frame;
        trigger_frame.set(frame);
    });
    let mut internal_open = use_signal(|| default_open);
    let is_controlled = open.is_some();
    let current_open = open.unwrap_or_else(|| *internal_open.read());

    let set_open = EventHandler::new(move |value: bool| {
        if !is_controlled {
            internal_open.set(value);
        }
        if let Some(handler) = on_open_change {
            handler.call(value);
        }
    });

    let panel_width = width.unwrap_or(MENU_PANEL_WIDTH);
    let style = MenuStyle {
        width: panel_width,
        submenu_width: panel_width - (spacing::XXS * 2.0),
        side_offset_vp: spacing::XXS,
    };

    let dismiss = EventHandler::new(move |_: ()| {
        set_open.call(false);
    });

    let panel_height = menu_closed_panel_height(&items);
    let placement = MenuOverlayPlacement::resolve(
        *trigger_frame.read(),
        viewport,
        style.width,
        panel_height,
        style.side_offset_vp,
    );

    rsx! {
        row {
            native_ref: trigger_ref,
            onclick: move |_| set_open.call(!current_open),
            {children}
        }
        OverlayPresence {
            open: current_open,
            preset: Some(arkit_animation::TransitionPreset::SlideUp),
            duration_ms: Some(FLOATING_ENTER_MS),
            exit_duration_ms: Some(FLOATING_EXIT_MS),
            distance: Some(FLOATING_DISTANCE),
            fill: Some(true),
            layer: Some(arkit_hooks::OverlayLayer::Floating),
            {menu_overlay_content(style, theme, dismiss, items, placement, None)}
        }
    }
}
