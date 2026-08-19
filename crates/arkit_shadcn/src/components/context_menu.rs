//! ContextMenu — right-click/long-press trigger + portal dropdown of entries.
//!
//! Ported from the legacy Elm builder `context_menu.rs`. The trigger owns a
//! native ArkUI long-press recognizer; ordinary taps remain available to its
//! child content. The menu panel renders through a root-projected portal.

use crate::components::floating_layer::live_trigger_frame;
use crate::components::menu_common::{
    menu_closed_panel_height, menu_overlay_content, MenuEntry, MenuOverlayPlacement, MenuStyle,
};
use crate::theme::*;
use arkit_prelude::*;

const MENU_PANEL_WIDTH: f32 = 224.0;

pub type ContextMenuEntry = MenuEntry;

#[component]
pub fn ContextMenu(
    items: Vec<ContextMenuEntry>,
    children: Element,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<EventHandler<bool>>,
    #[props(default)] width: Option<f32>,
) -> Element {
    let theme = use_theme();
    let viewport = arkit_hooks::use_overlay_viewport();
    let trigger_ref = arkit_hooks::use_native_element_ref();
    let trigger_frame = use_signal(arkit_hooks::LayoutFrame::default);
    let mut cursor_placement = use_signal(|| None::<MenuOverlayPlacement>);
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
        cursor_placement.set(None);
    });

    let panel_height = menu_closed_panel_height(&items);
    let placement = (*cursor_placement.read()).unwrap_or_else(|| {
        MenuOverlayPlacement::resolve(
            live_trigger_frame(&trigger_ref, *trigger_frame.read()),
            viewport,
            style.width,
            panel_height,
            style.side_offset_vp,
        )
    });

    rsx! {
        row {
            native_ref: trigger_ref.clone(),
            onlongpress: move |evt: dioxus_core::Event<dioxus_elements::event::ClickData>| {
                if current_open {
                    dismiss.call(());
                    return;
                }
                let frame = live_trigger_frame(&trigger_ref, *trigger_frame.read());
                let placement = evt
                    .data()
                    .pointer
                    .and_then(|pointer| {
                        MenuOverlayPlacement::from_cursor(
                            pointer,
                            viewport,
                            style.width,
                            panel_height,
                            style.side_offset_vp,
                        )
                    })
                    .unwrap_or_else(|| {
                        MenuOverlayPlacement::resolve(
                            frame,
                            viewport,
                            style.width,
                            panel_height,
                            style.side_offset_vp,
                        )
                    });
                cursor_placement.set(Some(placement));
                set_open.call(true);
            },
            {children}
        }
        if current_open {
            arkit_hooks::Portal {
                layer: arkit_hooks::OverlayLayer::Floating,
                {menu_overlay_content(style, theme, dismiss, items, placement, None)}
            }
        }
    }
}
