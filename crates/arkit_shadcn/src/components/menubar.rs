//! Menubar — horizontal row of menus, each opening an anchored overlay
//! dropdown.
//!
//! Ported from the legacy Elm builder `menubar.rs`. Each menu's open state is
//! held in a `Signal<Option<usize>>`; clicking a trigger toggles that menu and
//! closes the others. All original entry variants/styles are preserved via
//! [`crate::components::menu_common`].

use std::cell::RefCell;
use std::rc::Rc;

use crate::components::menu_common::{
    menu_closed_panel_height, menu_overlay_content, MenuEntry, MenuOverlayPassThroughRegion,
    MenuOverlayPlacement, MenuStyle,
};
use crate::components::motion::{OverlayPresence, FLOATING_ENTER_MS, FLOATING_EXIT_MS};
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
    let viewport = arkit_hooks::use_overlay_viewport();
    let menubar_ref = arkit_hooks::use_native_element_ref();
    let menubar_frame = use_signal(arkit_hooks::LayoutFrame::default);
    arkit_hooks::use_layout_frame(menubar_ref.clone(), move |frame| {
        let mut menubar_frame = menubar_frame;
        menubar_frame.set(frame);
    });
    let mut internal_active = use_signal(|| default_active);
    let is_controlled = active.is_some();
    let current_active = active.unwrap_or_else(|| *internal_active.read());
    let trigger_frames = use_hook(|| Rc::new(RefCell::new(Vec::<arkit_hooks::LayoutFrame>::new())));
    {
        let mut frames = trigger_frames.borrow_mut();
        if frames.len() != menus.len() {
            frames.resize(menus.len(), arkit_hooks::LayoutFrame::default());
        }
    }
    let mut frames_version = use_signal(|| 0_u64);
    let _ = frames_version();

    let set_active = EventHandler::new(move |value: Option<usize>| {
        if !is_controlled {
            internal_active.set(value);
        }
        if let Some(handler) = on_active_change {
            handler.call(value);
        }
    });
    let recorded_frames = trigger_frames.clone();
    let on_trigger_frame =
        EventHandler::new(move |(index, frame): (usize, arkit_hooks::LayoutFrame)| {
            let mut frames = recorded_frames.borrow_mut();
            if frames.get(index).copied() != Some(frame) && index < frames.len() {
                frames[index] = frame;
                frames_version += 1;
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
    let overlay_open = current_active.is_some();
    let overlay_payload = current_active.and_then(|index| {
        let items = menus.get(index)?.items.clone();
        let frame = trigger_frames
            .borrow()
            .get(index)
            .copied()
            .unwrap_or_default();
        let panel_height = menu_closed_panel_height(&items);
        let placement = MenuOverlayPlacement::resolve(
            frame,
            viewport,
            style.width,
            panel_height,
            style.side_offset_vp,
        );
        Some((items, placement))
    });
    let last_overlay = use_hook(|| {
        Rc::new(RefCell::new(
            None::<(Vec<MenubarEntry>, MenuOverlayPlacement)>,
        ))
    });
    if let Some(payload) = overlay_payload.clone() {
        *last_overlay.borrow_mut() = Some(payload);
    }
    let painted_overlay = overlay_payload.or_else(|| last_overlay.borrow().clone());
    let pass_through_region =
        MenuOverlayPassThroughRegion::from_frame(*menubar_frame.read(), viewport.frame);

    rsx! {
        row {
            native_ref: menubar_ref,
            padding: spacing::XXS,
            height: 36.0,
            align_items: "center",
            border_radius: md,
            border_width: 1.0,
            border_color: border,
            background_color: background,
            shadow: "sm",
            for (index, spec) in menus.iter().enumerate() {
                MenubarMenu {
                    index,
                    title: spec.title.clone(),
                    active: current_active == Some(index),
                    trigger_radius: sm,
                    active_background: accent,
                    foreground,
                    on_active_change: set_active,
                    on_trigger_frame,
                }
            }
        }
        OverlayPresence {
            open: overlay_open,
            preset: Some(arkit_animation::TransitionPreset::Fade),
            duration_ms: Some(FLOATING_ENTER_MS),
            exit_duration_ms: Some(FLOATING_EXIT_MS),
            fill: Some(true),
            layer: Some(arkit_hooks::OverlayLayer::Floating),
            {
                if let Some((items, placement)) = painted_overlay {
                    menu_overlay_content(
                        style,
                        theme,
                        EventHandler::new(move |_: ()| set_active.call(None)),
                        items,
                        placement,
                        pass_through_region,
                    )
                } else {
                    rsx! {}
                }
            }
        }
    }
}

#[component]
fn MenubarMenu(
    index: usize,
    title: String,
    active: bool,
    trigger_radius: f32,
    active_background: u32,
    foreground: u32,
    on_active_change: EventHandler<Option<usize>>,
    on_trigger_frame: EventHandler<(usize, arkit_hooks::LayoutFrame)>,
) -> Element {
    let trigger_ref = arkit_hooks::use_native_element_ref();
    arkit_hooks::use_layout_frame(trigger_ref.clone(), move |frame| {
        on_trigger_frame.call((index, frame));
    });

    rsx! {
        row {
            native_ref: trigger_ref,
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
            onclick: move |_| {
                if active {
                    on_active_change.call(None);
                } else {
                    on_active_change.call(Some(index));
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
