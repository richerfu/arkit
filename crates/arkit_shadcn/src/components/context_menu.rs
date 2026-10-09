//! ContextMenu — right-click/long-press trigger + portal dropdown of entries.
//!
//! Ported from the legacy Elm builder `context_menu.rs`. The trigger owns a
//! native ArkUI long-press recognizer; ordinary taps remain available to its
//! child content. The menu panel renders through a root-projected portal.

use crate::components::floating_layer::trigger_frame_for_anchor;
use crate::components::menu_common::{
    menu_closed_panel_height, menu_overlay_content, MenuEntry, MenuOverlayPlacement, MenuStyle,
};
use crate::components::motion::{OverlayPresence, FLOATING_ENTER_MS, FLOATING_EXIT_MS};
use crate::theme::*;
use arkit_prelude::*;

const MENU_PANEL_WIDTH: f32 = 224.0;

#[component]
pub fn ContextMenu(
    items: Vec<MenuEntry>,
    children: Element,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<EventHandler<bool>>,
    /// Accessible name for the long-press trigger. When omitted, ArkUI derives
    /// the name from the grouped trigger children.
    #[props(default)]
    accessibility_label: Option<String>,
    /// Optional usage hint appended to the expanded/collapsed state.
    #[props(default)]
    accessibility_description: Option<String>,
    #[props(default)] width: Option<f32>,
) -> Element {
    let theme = use_theme();
    let viewport = arkit_hooks::use_overlay_viewport();
    let trigger_ref = arkit_hooks::use_native_element_ref();
    let trigger_frame = use_signal(arkit_arkui::LayoutFramePx::default);
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

    let panel_width =
        super::panel_viewport::bounded_panel_width(viewport, width.unwrap_or(MENU_PANEL_WIDTH));
    let style = MenuStyle {
        width: panel_width,
        submenu_width: panel_width - (spacing::XXS * 2.0),
        side_offset_vp: spacing::XXS,
    };

    let dismiss = EventHandler::new(move |_: ()| {
        set_open.call(false);
    });

    let panel_height = menu_closed_panel_height(&items);
    let trigger_anchor = if current_open {
        trigger_frame_for_anchor(&trigger_ref, *trigger_frame.read())
    } else {
        *trigger_frame.read()
    };
    let placement = (*cursor_placement.read()).unwrap_or_else(|| {
        MenuOverlayPlacement::resolve(
            trigger_anchor,
            viewport,
            style.width,
            panel_height,
            style.side_offset_vp,
        )
    });
    let semantic_description = accessibility_description.unwrap_or_else(|| {
        if current_open {
            "Context menu expanded".to_string()
        } else {
            "Long press to open context menu".to_string()
        }
    });
    let mouse_trigger_ref = trigger_ref.clone();
    let long_press_trigger_ref = trigger_ref.clone();

    rsx! {
        row {
            native_ref: trigger_ref.clone(),
            accessibility_role: "button",
            accessibility_text: if let Some(label) = accessibility_label { label },
            accessibility_description: semantic_description,
            accessibility_group: true,
            accessibility_actions: "long_click",
            accessibility_selected: current_open,
            focusable: true,
            focus_on_touch: true,
            onmouse: move |evt| {
                let mouse = evt.data();
                if !mouse.secondary_down() {
                    return;
                }
                evt.stop_propagation();
                let pointer = dioxus_elements::event::PointerPayload {
                    action: mouse.action,
                    x: mouse.x,
                    y: mouse.y,
                    window_x: mouse.window_x,
                    window_y: mouse.window_y,
                    ..Default::default()
                };
                let placement = MenuOverlayPlacement::from_cursor(
                    pointer,
                    viewport,
                    style.width,
                    panel_height,
                    style.side_offset_vp,
                )
                .unwrap_or_else(|| {
                    let frame = trigger_frame_for_anchor(&mouse_trigger_ref, *trigger_frame.read());
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
            onkey: move |event| {
                if !event.data().is_down() {
                    return;
                }
                if event.data().key == dioxus_elements::event::KeyboardKey::Escape {
                    dismiss.call(());
                } else if event.data().key == dioxus_elements::event::KeyboardKey::Menu
                    || (event.data().key == dioxus_elements::event::KeyboardKey::F10
                        && event.data().modifiers.shift)
                {
                    cursor_placement.set(None);
                    set_open.call(true);
                }
            },
            onlongpress: move |evt: dioxus_core::Event<dioxus_elements::event::ClickData>| {
                evt.stop_propagation();
                if current_open {
                    dismiss.call(());
                    return;
                }
                let frame = trigger_frame_for_anchor(&long_press_trigger_ref, *trigger_frame.read());
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
            onaccessibilityaction: move |_| {
                // This trigger advertises only LongClick. AccessibilityManager
                // uses a different action numbering space from ArkUI's native
                // bit mask on current OpenHarmony images, so filtering the raw
                // integer would drop a valid performAction("longClick").
                if !current_open {
                    cursor_placement.set(None);
                    set_open.call(true);
                }
            },
            {children}
        }
        OverlayPresence {
            open: current_open,
            // The capture plane must not translate: moving the full-screen
            // hit layer shifts a cursor-anchored menu and can consume the
            // release event that completed a long press.
            preset: Some(arkit_animation::TransitionPreset::Fade),
            duration_ms: Some(FLOATING_ENTER_MS),
            exit_duration_ms: Some(FLOATING_EXIT_MS),
            fill: Some(true),
            layer: Some(arkit_hooks::OverlayLayer::Floating),
            {menu_overlay_content(style, theme, dismiss, items, placement, None, None)}
        }
    }
}
