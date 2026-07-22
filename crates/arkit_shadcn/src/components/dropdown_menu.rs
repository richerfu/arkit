//! DropdownMenu — trigger + portal dropdown of entries.
//!
//! Ported from the legacy Elm builder `dropdown_menu.rs`. Menu panels render
//! through the framework overlay root instead of inline so they are not clipped
//! by the trigger's parent layout and so trigger children can remain ordinary
//! shadcn buttons.

use crate::components::menu_common::{
    menu_closed_panel_height, use_menu_overlay_refresh, MenuEntry, MenuOverlayPlacement,
    MenuOverlaySession, MenuStyle,
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
    let overlay = arkit_hooks::use_overlay();
    let trigger_frame = use_signal(arkit_hooks::LayoutFrame::default);
    let mut overlay_session = use_signal(|| None::<MenuOverlaySession>);
    arkit_hooks::use_layout_frame(move |frame| {
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

    let dismiss_overlay = overlay.clone();
    let mut dismiss_session = overlay_session;
    let dismiss = EventHandler::new(move |_: ()| {
        set_open.call(false);
        dismiss_session.set(None);
        dismiss_overlay.dismiss();
    });

    use_menu_overlay_refresh(
        overlay.clone(),
        current_open,
        overlay_session,
        style,
        theme,
        dismiss,
        items.clone(),
    );

    let mut toggle = move |_| {
        if current_open {
            dismiss.call(());
        } else {
            set_open.call(true);
            let entries = items.clone();
            let frame = *trigger_frame.read();
            let viewport = overlay.viewport();
            let panel_height = menu_closed_panel_height(&entries);
            let placement = MenuOverlayPlacement::resolve(
                frame,
                viewport,
                style.width,
                panel_height,
                style.side_offset_vp,
            );
            let session = MenuOverlaySession::new(placement, None);
            overlay_session.set(Some(session));
            session.show(&overlay, style, theme, dismiss, entries);
        }
    };

    rsx! {
        row {
            onclick: move |_| {
                toggle(());
            },
            {children}
        }
    }
}
