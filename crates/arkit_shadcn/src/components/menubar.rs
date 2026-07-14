//! Menubar — horizontal row of menus, each opening an anchored overlay
//! dropdown.
//!
//! Ported from the legacy Elm builder `menubar.rs`. Each menu's open state is
//! held in a `Signal<Option<usize>>`; clicking a trigger toggles that menu and
//! closes the others. All original entry variants/styles are preserved via
//! [`crate::components::menu_common`].

use crate::components::menu_common::{
    menu_closed_panel_height, use_menu_overlay_refresh, MenuEntry, MenuOverlayPassThroughRegion,
    MenuOverlayPlacement, MenuOverlaySession, MenuStyle,
};
use crate::theme::*;
use arkit_prelude::*;

const MENU_PANEL_WIDTH: f32 = 224.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);
const MENU_PANEL_SIDE_OFFSET: f32 = spacing::SM;
const MENUBAR_ITEM_TRANSPARENT: u32 = 0x00000000;

pub type MenubarEntry = MenuEntry;

/// A single menu spec: trigger title + entries.
#[derive(Debug, Clone, PartialEq)]
pub struct MenubarMenuSpec {
    pub title: String,
    pub items: Vec<MenubarEntry>,
}

impl MenubarMenuSpec {
    pub fn new(title: impl Into<String>, items: Vec<MenubarEntry>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

#[component]
pub fn Menubar(
    menus: Vec<MenubarMenuSpec>,
    active: Option<Option<usize>>,
    default_active: Option<usize>,
    on_active_change: Option<EventHandler<Option<usize>>>,
) -> Element {
    let theme = use_theme();
    let menubar_frame = use_signal(arkit_hooks::LayoutFrame::default);
    let mut internal_active = use_signal(|| default_active);
    let is_controlled = active.is_some();
    let current_active = active.unwrap_or_else(|| *internal_active.read());
    let current_menubar_frame = *menubar_frame.read();

    let set_active = EventHandler::new(move |value: Option<usize>| {
        if !is_controlled {
            internal_active.set(value);
        }
        if let Some(handler) = on_active_change {
            handler.call(value);
        }
    });

    let style = MenuStyle {
        width: MENU_PANEL_WIDTH,
        submenu_width: SUBMENU_PANEL_WIDTH,
        side_offset_vp: MENU_PANEL_SIDE_OFFSET,
    };
    let sm = theme.radii.sm;
    let md = theme.radii.md;
    let accent = theme.colors.accent;
    let foreground = theme.colors.foreground;
    let border = theme.colors.border;
    let background = theme.colors.background;

    rsx! {
        row {
            padding: spacing::XXS,
            height: 36.0,
            align_items: "center",
            border_radius: md,
            border_width: 1.0,
            border_color: border,
            background_color: background,
            shadow: 1i32,
            onarea: move |evt: dioxus_core::Event<dioxus_elements::event::AreaData>| {
                let frame = evt.data().frame;
                if frame.is_measured() {
                    let mut menubar_frame = menubar_frame;
                    menubar_frame.set(arkit_hooks::LayoutFrame {
                        x: frame.x,
                        y: frame.y,
                        width: frame.width,
                        height: frame.height,
                    });
                }
            },
            for (index, spec) in menus.iter().enumerate() {
                MenubarMenu {
                    index,
                    title: spec.title.clone(),
                    items: spec.items.clone(),
                    active: current_active == Some(index),
                    pass_through_frame: current_menubar_frame,
                    style,
                    theme,
                    trigger_radius: sm,
                    active_background: accent,
                    foreground,
                    on_active_change: set_active,
                }
            }
        }
    }
}

#[component]
fn MenubarMenu(
    index: usize,
    title: String,
    items: Vec<MenubarEntry>,
    active: bool,
    pass_through_frame: arkit_hooks::LayoutFrame,
    style: MenuStyle,
    theme: Theme,
    trigger_radius: f32,
    active_background: u32,
    foreground: u32,
    on_active_change: EventHandler<Option<usize>>,
) -> Element {
    let overlay = arkit_hooks::use_overlay();
    let trigger_frame = use_signal(arkit_hooks::LayoutFrame::default);
    let mut overlay_session = use_signal(|| None::<MenuOverlaySession>);

    let dismiss_overlay = overlay.clone();
    let mut dismiss_session = overlay_session;
    let dismiss = EventHandler::new(move |_: ()| {
        on_active_change.call(None);
        dismiss_session.set(None);
        dismiss_overlay.dismiss();
    });

    use_menu_overlay_refresh(
        overlay.clone(),
        active,
        *overlay_session.read(),
        style,
        theme,
        dismiss,
        items.clone(),
    );

    let open_overlay = overlay.clone();

    let mut open_menu = move |pointer: Option<dioxus_elements::event::PointerPayload>| {
        on_active_change.call(Some(index));
        let entries = items.clone();
        let frame = *trigger_frame.read();
        let viewport = open_overlay.viewport();
        let panel_height = menu_closed_panel_height(&entries);
        let pass_through_region =
            MenuOverlayPassThroughRegion::from_frame(pass_through_frame, viewport.frame);
        let placement = if let Some(placement) = pointer.and_then(|pointer| {
            MenuOverlayPlacement::from_pointer(
                pointer,
                viewport,
                style.width,
                panel_height,
                style.side_offset_vp,
            )
        }) {
            placement
        } else if frame.is_measured() {
            MenuOverlayPlacement::from_trigger(
                frame,
                viewport,
                style.width,
                panel_height,
                style.side_offset_vp,
            )
        } else {
            MenuOverlayPlacement::fallback(viewport)
        };
        let session = MenuOverlaySession::new(placement, pass_through_region);
        overlay_session.set(Some(session));
        session.show(&open_overlay, style, theme, dismiss, entries);
    };

    let close_menu = move || {
        dismiss.call(());
    };

    rsx! {
        row {
            margin_left: if index > 0 { spacing::XXS } else { 0.0 },
            height: 28.0,
            align_items: "center",
            justify_content: "center",
            padding_top: spacing::XXS,
            padding_right: spacing::SM,
            padding_bottom: spacing::XXS,
            padding_left: spacing::SM,
            border_radius: trigger_radius,
            background_color: if active { active_background } else { MENUBAR_ITEM_TRANSPARENT },
            onarea: move |evt: dioxus_core::Event<dioxus_elements::event::AreaData>| {
                let frame = evt.data().frame;
                if frame.is_measured() {
                    let mut trigger_frame = trigger_frame;
                    trigger_frame.set(arkit_hooks::LayoutFrame {
                        x: frame.x,
                        y: frame.y,
                        width: frame.width,
                        height: frame.height,
                    });
                }
            },
            onclick: move |evt: dioxus_core::Event<dioxus_elements::event::ClickData>| {
                if active {
                    close_menu();
                } else {
                    open_menu(evt.data().pointer);
                }
            },
            text {
                font_size: typography::SM,
                font_weight: 500i32,
                font_color: foreground,
                line_height: 20.0,
                {title}
            }
        }
    }
}
