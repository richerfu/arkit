//! Shared menu helpers/styles — entry types, menu style, and rendering helpers.
//!
//! Ported from the legacy Elm builder `menu_common.rs`. This is NOT a
//! `#[component]`: it exposes entry enum types, a [`MenuStyle`] descriptor, and
//! `rsx!`-based rendering helpers shared by `dropdown_menu`, `context_menu`,
//! `menubar`, and `select`.
//!
//! The root popup is rendered through the Dioxus overlay tree. Nested submenus
//! stay inside the same popup and are expanded by path state owned by the menu
//! panel, matching the legacy interaction contract without making callers track
//! submenu state.

use crate::theme::*;
use arkit_prelude::*;

use super::floating_layer::{FloatingAlign, FLOATING_CAPTURE_COLOR};
use super::motion::ExpandPresence;

pub(crate) const TRANSPARENT: u32 = 0x00000000;
const MENU_PANEL_HORIZONTAL_PADDING: f32 = spacing::XXS * 2.0;
const MENU_PANEL_VERTICAL_PADDING: f32 = spacing::XXS * 2.0;
const MENU_ROW_HEIGHT: f32 = control::HEIGHT_SM;
const MENU_SEPARATOR_HEIGHT: f32 = 9.0;
const MENU_TEXT_MAX_LINES: i32 = 1;
const MENU_TEXT_OVERFLOW_ELLIPSIS: &str = "ellipsis";
const MENU_TRAILING_GAP: f32 = spacing::SM;
const MENU_VIEWPORT_PADDING: f32 = spacing::LG;
const MENU_ICON_SIZE: f32 = 14.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MenuOverlayPlacement {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MenuOverlayPassThroughRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl MenuOverlayPassThroughRegion {
    pub(crate) fn from_frame(
        frame: arkit_arkui::LayoutFramePx,
        viewport: arkit_hooks::OverlayViewport,
    ) -> Option<Self> {
        if !frame.is_measured() {
            return None;
        }

        let scale = super::floating_layer::viewport_scale(viewport);
        let measured_overlay = if viewport.frame.is_measured() {
            viewport.frame
        } else {
            Default::default()
        };
        let origin = super::floating_layer::overlay_local_vp(
            arkit_arkui::WindowPxPoint::new(frame.x, frame.y),
            arkit_arkui::WindowPxPoint::new(measured_overlay.x, measured_overlay.y),
            scale,
        );
        let size = arkit_arkui::LogicalSizeVp::from_physical(frame.into(), scale)?;

        Some(Self {
            x: origin.x.max(0.0),
            y: origin.y.max(0.0),
            width: size.width.max(0.0),
            height: size.height.max(0.0),
        })
    }

    fn top(self) -> f32 {
        self.y.max(0.0)
    }
}

impl MenuOverlayPlacement {
    pub(crate) fn from_trigger(
        trigger: arkit_arkui::LayoutFramePx,
        viewport: arkit_hooks::OverlayViewport,
        panel_width: f32,
        panel_height: f32,
        align: FloatingAlign,
        side_offset: f32,
    ) -> Self {
        let scale = super::floating_layer::viewport_scale(viewport);
        let metrics = super::floating_layer::overlay_metrics_vp(
            viewport.frame,
            scale,
            panel_width,
            panel_height,
        );
        let trigger_origin = super::floating_layer::overlay_local_vp(
            arkit_arkui::WindowPxPoint::new(trigger.x, trigger.y),
            metrics.origin,
            scale,
        );
        let trigger_x = trigger_origin.x;
        let trigger_y = trigger_origin.y;
        let trigger_width = trigger.width / scale;
        let trigger_height = trigger.height / scale;
        let edge = MENU_VIEWPORT_PADDING;
        let min_x = viewport.safe_area.left.max(0.0) + edge;
        let min_y = viewport.safe_area.top.max(0.0) + edge;
        let max_x = (metrics.size.width - viewport.safe_area.right.max(0.0) - panel_width - edge)
            .max(min_x);
        let trigger_bottom = trigger_y + trigger_height;
        let below_y = trigger_bottom + side_offset;
        let above_y = trigger_y - panel_height - side_offset;
        let max_y =
            (metrics.size.height - viewport.safe_area.bottom.max(0.0) - panel_height - edge)
                .max(min_y);
        let below_fits = below_y + panel_height
            <= metrics.size.height - viewport.safe_area.bottom.max(0.0) - edge;
        let above_fits = above_y >= min_y;
        let y = if below_fits || !above_fits {
            below_y
        } else {
            above_y
        };

        let raw_x = match align {
            FloatingAlign::Start => trigger_x,
            FloatingAlign::Center => trigger_x + ((trigger_width - panel_width) / 2.0),
            FloatingAlign::End => trigger_x + trigger_width - panel_width,
        };

        Self {
            x: raw_x.clamp(min_x, max_x),
            y: y.clamp(min_y, max_y),
        }
    }

    /// Prefer the measured trigger root (shadcn-style). Ignore pointer target
    /// bounds — those often describe an inner label/icon, not the control.
    pub(crate) fn resolve(
        trigger: arkit_arkui::LayoutFramePx,
        viewport: arkit_hooks::OverlayViewport,
        panel_width: f32,
        panel_height: f32,
        align: FloatingAlign,
        side_offset: f32,
    ) -> Self {
        if trigger.is_measured() {
            Self::from_trigger(
                trigger,
                viewport,
                panel_width,
                panel_height,
                align,
                side_offset,
            )
        } else {
            Self::fallback(viewport)
        }
    }

    /// Cursor-point anchor for context menus (window coords as a 1×1 frame).
    pub(crate) fn from_cursor(
        pointer: dioxus_elements::event::PointerPayload,
        viewport: arkit_hooks::OverlayViewport,
        panel_width: f32,
        panel_height: f32,
        _side_offset: f32,
    ) -> Option<Self> {
        if !pointer.has_window_position() {
            return None;
        }
        let scale = super::floating_layer::viewport_scale(viewport);
        let metrics = super::floating_layer::overlay_metrics_vp(
            viewport.frame,
            scale,
            panel_width,
            panel_height,
        );
        let cursor = super::floating_layer::overlay_local_vp(
            arkit_arkui::WindowPxPoint::new(pointer.window_x, pointer.window_y),
            metrics.origin,
            scale,
        );
        let edge = MENU_VIEWPORT_PADDING;
        let min_x = viewport.safe_area.left.max(0.0) + edge;
        let min_y = viewport.safe_area.top.max(0.0) + edge;
        let max_x = (metrics.size.width - viewport.safe_area.right.max(0.0) - panel_width - edge)
            .max(min_x);
        let max_y =
            (metrics.size.height - viewport.safe_area.bottom.max(0.0) - panel_height - edge)
                .max(min_y);
        Some(Self {
            x: cursor.x.clamp(min_x, max_x),
            y: cursor.y.clamp(min_y, max_y),
        })
    }

    pub(crate) fn fallback(viewport: arkit_hooks::OverlayViewport) -> Self {
        Self {
            x: viewport.safe_area.left + MENU_VIEWPORT_PADDING,
            y: (viewport.safe_area.top + MENU_VIEWPORT_PADDING).max(96.0),
        }
    }
}

/// Layout/sizing descriptor for a menu popup.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuStyle {
    pub width: f32,
    pub submenu_width: f32,
    pub side_offset_vp: f32,
}

impl Default for MenuStyle {
    fn default() -> Self {
        Self {
            width: 224.0,
            submenu_width: 224.0 - (spacing::XXS * 2.0),
            side_offset_vp: spacing::XXS,
        }
    }
}

/// A single entry in a menu.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuEntry {
    Action(MenuActionEntry),
    Submenu(MenuSubmenuEntry),
    Checkbox(MenuCheckboxEntry),
    Radio(MenuRadioEntry),
    Label(MenuLabelEntry),
    Separator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuActionEntry {
    pub title: String,
    pub shortcut: Option<String>,
    pub icon: Option<String>,
    pub destructive: bool,
    pub disabled: bool,
    pub inset: bool,
    pub on_select: Option<EventHandler<()>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuSubmenuEntry {
    pub title: String,
    pub icon: Option<String>,
    pub inset: bool,
    pub items: Vec<MenuEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuCheckboxEntry {
    pub title: String,
    pub shortcut: Option<String>,
    pub checked: bool,
    pub close_on_select: bool,
    pub on_toggle: EventHandler<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuRadioEntry {
    pub title: String,
    pub value: String,
    pub selected: String,
    pub close_on_select: bool,
    pub on_select: EventHandler<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuLabelEntry {
    pub title: String,
    pub inset: bool,
}

// --- Entry constructors ----------------------------------------------------

impl MenuEntry {
    pub fn action(title: impl Into<String>) -> Self {
        Self::Action(MenuActionEntry {
            title: title.into(),
            shortcut: None,
            icon: None,
            destructive: false,
            disabled: false,
            inset: false,
            on_select: None,
        })
    }

    pub fn submenu(title: impl Into<String>, items: Vec<MenuEntry>) -> Self {
        Self::Submenu(MenuSubmenuEntry {
            title: title.into(),
            icon: None,
            inset: false,
            items,
        })
    }

    pub fn checkbox(
        title: impl Into<String>,
        checked: bool,
        on_toggle: EventHandler<bool>,
    ) -> Self {
        Self::Checkbox(MenuCheckboxEntry {
            title: title.into(),
            shortcut: None,
            checked,
            close_on_select: false,
            on_toggle,
        })
    }

    pub fn radio(
        title: impl Into<String>,
        value: impl Into<String>,
        selected: impl Into<String>,
        on_select: EventHandler<String>,
    ) -> Self {
        Self::Radio(MenuRadioEntry {
            title: title.into(),
            value: value.into(),
            selected: selected.into(),
            close_on_select: false,
            on_select,
        })
    }

    pub fn label(title: impl Into<String>) -> Self {
        Self::Label(MenuLabelEntry {
            title: title.into(),
            inset: false,
        })
    }

    pub fn separator() -> Self {
        Self::Separator
    }

    pub fn destructive(mut self) -> Self {
        if let Self::Action(entry) = &mut self {
            entry.destructive = true;
        }
        self
    }

    pub fn disabled(mut self) -> Self {
        if let Self::Action(entry) = &mut self {
            entry.disabled = true;
        }
        self
    }

    pub fn inset(mut self) -> Self {
        match &mut self {
            Self::Action(entry) => entry.inset = true,
            Self::Submenu(entry) => entry.inset = true,
            Self::Label(entry) => entry.inset = true,
            _ => {}
        }
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        let shortcut = shortcut.into();
        match &mut self {
            Self::Action(entry) => entry.shortcut = Some(shortcut.clone()),
            Self::Checkbox(entry) => entry.shortcut = Some(shortcut),
            _ => {}
        }
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        let icon = icon.into();
        match &mut self {
            Self::Action(entry) => entry.icon = Some(icon.clone()),
            Self::Submenu(entry) => entry.icon = Some(icon),
            _ => {}
        }
        self
    }

    pub fn on_select(mut self, callback: EventHandler<()>) -> Self {
        if let Self::Action(entry) = &mut self {
            entry.on_select = Some(callback);
        }
        self
    }

    /// Close the owning menu after a checkbox/radio value is committed.
    pub fn close_on_select(mut self) -> Self {
        match &mut self {
            Self::Checkbox(entry) => entry.close_on_select = true,
            Self::Radio(entry) => entry.close_on_select = true,
            _ => {}
        }
        self
    }
}

pub fn menu_action_entry(
    title: impl Into<String>,
    shortcut: Option<String>,
    destructive: bool,
    disabled: bool,
    inset: bool,
    on_select: Option<EventHandler<()>>,
) -> MenuEntry {
    MenuEntry::Action(MenuActionEntry {
        title: title.into(),
        shortcut,
        icon: None,
        destructive,
        disabled,
        inset,
        on_select,
    })
}

pub fn menu_submenu_entry(
    title: impl Into<String>,
    inset: bool,
    items: Vec<MenuEntry>,
) -> MenuEntry {
    MenuEntry::Submenu(MenuSubmenuEntry {
        title: title.into(),
        icon: None,
        inset,
        items,
    })
}

pub fn menu_checkbox_entry(
    title: impl Into<String>,
    checked: bool,
    on_toggle: EventHandler<bool>,
) -> MenuEntry {
    MenuEntry::Checkbox(MenuCheckboxEntry {
        title: title.into(),
        shortcut: None,
        checked,
        close_on_select: false,
        on_toggle,
    })
}

pub fn menu_radio_entry(
    title: impl Into<String>,
    value: impl Into<String>,
    selected: impl Into<String>,
    on_select: EventHandler<String>,
) -> MenuEntry {
    MenuEntry::Radio(MenuRadioEntry {
        title: title.into(),
        value: value.into(),
        selected: selected.into(),
        close_on_select: false,
        on_select,
    })
}

pub fn menu_label_entry(title: impl Into<String>, inset: bool) -> MenuEntry {
    MenuEntry::Label(MenuLabelEntry {
        title: title.into(),
        inset,
    })
}

pub fn menu_separator_entry() -> MenuEntry {
    MenuEntry::Separator
}

// --- Style helpers ---------------------------------------------------------

pub(crate) fn menu_row_min_width(style: &MenuStyle) -> f32 {
    (style.width - MENU_PANEL_HORIZONTAL_PADDING).max(0.0)
}

pub(crate) fn menu_submenu_min_width(style: &MenuStyle) -> f32 {
    style.submenu_width.max(0.0)
}

pub(crate) fn menu_subtree_min_width(style: &MenuStyle) -> f32 {
    menu_row_min_width(style).max(menu_submenu_min_width(style))
}

pub(crate) fn menu_closed_panel_height(entries: &[MenuEntry]) -> f32 {
    menu_entries_closed_height(entries)
}

fn menu_entries_closed_height(entries: &[MenuEntry]) -> f32 {
    MENU_PANEL_VERTICAL_PADDING
        + entries
            .iter()
            .map(|entry| match entry {
                MenuEntry::Separator => MENU_SEPARATOR_HEIGHT,
                _ => MENU_ROW_HEIGHT,
            })
            .sum::<f32>()
}

// --- Rendering helpers -----------------------------------------------------

/// Render the menu popup panel (column) with entries. Caller renders this
/// inline when the menu is open.
pub(crate) fn menu_content(
    style: MenuStyle,
    theme: &Theme,
    on_dismiss: EventHandler<()>,
    entries: &[MenuEntry],
    top: f32,
    navigation: Option<MenuHorizontalNavigation>,
) -> Element {
    rsx! {
        MenuContentPanel {
            key: "menu-{navigation.map(|navigation| navigation.identity).unwrap_or(0)}",
            style,
            theme: *theme,
            on_dismiss,
            entries: entries.to_vec(),
            top,
            navigation,
        }
    }
}

#[component]
fn MenuContentPanel(
    style: MenuStyle,
    theme: Theme,
    on_dismiss: EventHandler<()>,
    entries: Vec<MenuEntry>,
    top: f32,
    navigation: Option<MenuHorizontalNavigation>,
) -> Element {
    let mut open_path = use_signal(Vec::<usize>::new);
    let current_open_path = open_path.read().clone();
    let set_open_path = EventHandler::new(move |next: Vec<usize>| {
        open_path.set(next);
    });
    let colors = &theme.colors;
    let viewport = arkit_hooks::use_overlay_viewport();
    let max_height = (super::panel_viewport::panel_available_height(viewport, top)
        - spacing::XXS * 2.0)
        .max(1.0);
    let reserve_leading_slot = menu_entries_need_leading_slot(&entries);
    let render_context = MenuRenderContext {
        open_path: &current_open_path,
        set_open_path,
        style,
        theme: &theme,
        on_dismiss,
        reserve_leading_slot,
    };

    rsx! {
        column {
            accessibility_role: "menu",
            focus_scope: true,
            focus_navigation: "vertical",
            key_capture: "escape",
            width: style.width,
            align_self: "start",
            align_items: "start",
            onkey: move |event| {
                if event.data().action == dioxus_elements::event::KeyAction::Down
                    && event.data().key == dioxus_elements::event::KeyboardKey::Escape
                {
                    event.stop_propagation();
                    on_dismiss.call(());
                } else if event.data().is_down() && matches!(event.data().key, dioxus_elements::event::KeyboardKey::ArrowLeft | dioxus_elements::event::KeyboardKey::ArrowRight) {
                    if let Some(navigation) = navigation { event.stop_propagation(); navigation.on_key.call(event.data().key); }
                }
            },
            padding_top: spacing::XXS,
            padding_right: spacing::XXS,
            padding_bottom: spacing::XXS,
            padding_left: spacing::XXS,
            border_radius: theme.radii.md,
            border_width: 1.0,
            border_color: colors.border,
            clip: false,
            background_color: colors.popover,
            shadow: shadow::MD,
            super::panel_viewport::PanelViewport {
                max_height,
            for (index, entry) in entries.iter().enumerate() {
                {
                    render_menu_entry(
                        entry,
                        index,
                        &[],
                        render_context,
                    )
                }
            }
            }
        }
    }
}

/// Render menu content in the app-level overlay portal.
///
/// The root overlay is the one valid stack here: it layers a full-screen
/// dismiss region behind an anchored panel. Menu item layout below this point
/// uses row/column, matching the legacy builder implementation.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct MenuHorizontalNavigation {
    pub identity: usize,
    pub on_key: EventHandler<dioxus_elements::event::KeyboardKey>,
}

pub(crate) fn menu_overlay_content(
    style: MenuStyle,
    theme: Theme,
    on_dismiss: EventHandler<()>,
    entries: Vec<MenuEntry>,
    placement: MenuOverlayPlacement,
    pass_through_region: Option<MenuOverlayPassThroughRegion>,
    navigation: Option<MenuHorizontalNavigation>,
) -> Element {
    let top = placement.y.max(0.0);
    let left = placement.x.max(0.0);
    let pass_through_region =
        pass_through_region.filter(|region| region.width > 0.0 && region.height > 0.0);
    rsx! {
        stack {
            width: "100%",
            height: "100%",
            alignment: "top-start",
            hit_test_behavior: "none",
            column {
                width: "100%",
                height: "100%",
                align_items: "start",
                hit_test_behavior: "none",
                if let Some(region) = pass_through_region {
                    if region.top() > 0.0 {
                        row {
                            width: "100%",
                            height: region.top(),
                            background_color: FLOATING_CAPTURE_COLOR,
                            hit_test_behavior: "default",
                            onclick: move |_| on_dismiss.call(()),
                        }
                    }
                    row {
                        width: "100%",
                        height: region.height,
                        hit_test_behavior: "none",
                        if region.x > 0.0 {
                            row {
                                width: region.x,
                                height: "100%",
                                background_color: FLOATING_CAPTURE_COLOR,
                                hit_test_behavior: "default",
                                onclick: move |_| on_dismiss.call(()),
                            }
                        }
                        row {
                            width: region.width,
                            height: "100%",
                            hit_test_behavior: "none",
                        }
                        row {
                            layout_weight: 1.0,
                            height: "100%",
                            background_color: FLOATING_CAPTURE_COLOR,
                            hit_test_behavior: "default",
                            onclick: move |_| on_dismiss.call(()),
                        }
                    }
                }
                column {
                    width: "100%",
                    layout_weight: 1.0,
                    background_color: FLOATING_CAPTURE_COLOR,
                    hit_test_behavior: "default",
                    onclick: move |_| on_dismiss.call(()),
                }
            }
            column {
                position: format!("{left},{top}"),
                onclick: move |evt| evt.stop_propagation(),
                {menu_content(style, &theme, on_dismiss, &entries, top, navigation)}
            }
        }
    }
}

#[derive(Clone, Copy)]
struct MenuRenderContext<'a> {
    open_path: &'a [usize],
    set_open_path: EventHandler<Vec<usize>>,
    style: MenuStyle,
    theme: &'a Theme,
    on_dismiss: EventHandler<()>,
    reserve_leading_slot: bool,
}

/// Shared desktop interaction shell for actionable menu rows. Keeping hover,
/// focus, click, and keyboard activation here prevents the four entry variants
/// from drifting apart visually or behaviorally.
#[component]
fn MenuItemSurface(
    accessibility_label: String,
    #[props(default = "menuitem".to_string())] accessibility_role: String,
    #[props(default)] accessibility_checked: Option<bool>,
    #[props(default)] accessibility_selected: bool,
    width: f32,
    height: f32,
    radius: f32,
    background: u32,
    interactive_background: u32,
    opacity: f32,
    disabled: bool,
    on_activate: EventHandler<()>,
    #[props(default)] on_navigation: Option<EventHandler<dioxus_elements::event::KeyboardKey>>,
    children: Element,
) -> Element {
    let mut hovering = use_signal(|| false);
    let mut focused = use_signal(|| false);
    rsx! {
        row {
            accessibility_role,
            accessibility_text: accessibility_label,
            accessibility_group: true,
            accessibility_actions: "click",
            accessibility_disabled: disabled,
            accessibility_checked: if let Some(checked) = accessibility_checked { checked },
            accessibility_selected,
            enabled: !disabled,
            width,
            height,
            align_self: "start",
            align_items: "center",
            justify_content: "start",
            padding_top: 8.0,
            padding_right: 8.0,
            padding_bottom: 8.0,
            padding_left: 8.0,
            border_radius: radius,
            clip: true,
            background_color: if !disabled && (hovering() || focused()) {
                interactive_background
            } else {
                background
            },
            opacity,
            focusable: !disabled,
            key_capture: "enter space left right escape",
            focus_on_touch: true,
            onclick: move |event| {
                event.stop_propagation();
                if !disabled {
                    on_activate.call(());
                }
            },
            onkey: move |event| {
                if !disabled && event.data().activates() {
                    event.stop_propagation();
                    on_activate.call(());
                } else if !disabled && event.data().is_down() {
                    if let Some(handler) = on_navigation {
                        if event.data().key == dioxus_elements::event::KeyboardKey::ArrowRight { event.stop_propagation(); handler.call(event.data().key); }
                    }
                }
            },
            onhover: move |event| hovering.set(event.data().is_hovering),
            onfocus: move |_| focused.set(true),
            onblur: move |_| focused.set(false),
            {children}
        }
    }
}

fn render_menu_entry(
    entry: &MenuEntry,
    index: usize,
    parent_path: &[usize],
    context: MenuRenderContext<'_>,
) -> Element {
    match entry {
        MenuEntry::Action(e) => render_action_entry(
            e,
            context.style,
            context.theme,
            context.on_dismiss,
            context.reserve_leading_slot,
        ),
        MenuEntry::Submenu(e) => render_submenu_entry(e, index, parent_path, context),
        MenuEntry::Checkbox(e) => {
            render_checkbox_entry(e, context.style, context.theme, context.on_dismiss)
        }
        MenuEntry::Radio(e) => {
            render_radio_entry(e, context.style, context.theme, context.on_dismiss)
        }
        MenuEntry::Label(e) => render_label_entry(
            e,
            context.style,
            context.theme,
            context.reserve_leading_slot,
        ),
        MenuEntry::Separator => render_separator_entry(context.style, context.theme),
    }
}

fn render_action_entry(
    entry: &MenuActionEntry,
    style: MenuStyle,
    theme: &Theme,
    on_dismiss: EventHandler<()>,
    reserve_leading_slot: bool,
) -> Element {
    let min_width = menu_row_min_width(&style);
    let colors = &theme.colors;
    let title_color = if entry.destructive {
        colors.destructive
    } else {
        colors.popover_foreground
    };
    let title = entry.title.clone();
    let shortcut = entry.shortcut.clone();
    let icon = entry.icon.clone();
    let on_select = entry.on_select;
    let disabled = entry.disabled;
    let inset = entry.inset;
    let sm = theme.radii.sm;

    rsx! {
        MenuItemSurface {
            accessibility_label: title.clone(),
            width: min_width,
            height: MENU_ROW_HEIGHT,
            radius: sm,
            background: TRANSPARENT,
            interactive_background: colors.accent,
            opacity: if disabled { 0.5f32 } else { 1.0f32 },
            disabled,
            on_activate: move |_| {
                if let Some(on_select) = on_select {
                    on_select.call(());
                }
                on_dismiss.call(());
            },
            row {
                layout_weight: 1.0,
                clip: true,
                justify_content: "start",
                align_items: "center",
                if let Some(icon) = icon {
                    {menu_icon_leading_slot(icon, colors.foreground)}
                } else if inset || reserve_leading_slot {
                    {menu_empty_leading_slot()}
                }
                {menu_item_text(title.clone(), title_color, 400)}
            }
            if let Some(shortcut) = shortcut {
                {menu_trailing_text(shortcut, colors.muted_foreground)}
            }
        }
    }
}

fn render_submenu_entry(
    entry: &MenuSubmenuEntry,
    index: usize,
    parent_path: &[usize],
    context: MenuRenderContext<'_>,
) -> Element {
    let MenuRenderContext {
        open_path,
        set_open_path,
        style,
        theme,
        reserve_leading_slot,
        ..
    } = context;
    let min_width = menu_row_min_width(&style);
    let submenu_min_width = menu_submenu_min_width(&style);
    let colors = &theme.colors;
    let title = entry.title.clone();
    let icon = entry.icon.clone();
    let inset = entry.inset;
    let sm = theme.radii.sm;
    let branch_path = menu_branch_path(parent_path, index);
    let submenu_open = menu_branch_is_open(open_path, &branch_path);
    let next_open_path = if submenu_open {
        parent_path.to_vec()
    } else {
        branch_path.clone()
    };
    let chevron = if submenu_open {
        "chevron-up"
    } else {
        "chevron-down"
    };
    let child_reserve_leading_slot = menu_entries_need_leading_slot(&entry.items);
    let keyboard_open_path = branch_path.clone();
    let keyboard_parent_path = parent_path.to_vec();

    rsx! {
        column {
            width: menu_subtree_min_width(&style),
            align_self: "start",
            align_items: "start",
            MenuItemSurface {
            accessibility_label: title.clone(),
                accessibility_selected: submenu_open,
                width: min_width,
                height: MENU_ROW_HEIGHT,
                radius: sm,
                background: if submenu_open { colors.accent } else { TRANSPARENT },
                interactive_background: colors.accent,
                opacity: 1.0,
                disabled: false,
                on_activate: move |_| {
                    set_open_path.call(next_open_path.clone());
                },
                on_navigation: move |_| set_open_path.call(keyboard_open_path.clone()),
                row {
                    layout_weight: 1.0,
                    clip: true,
                    justify_content: "start",
                    align_items: "center",
                    if let Some(icon) = icon {
                        {menu_icon_leading_slot(icon, colors.foreground)}
                    } else if inset || reserve_leading_slot {
                        {menu_empty_leading_slot()}
                    }
                    {menu_item_text(title.clone(), colors.popover_foreground, 400)}
                }
                row {
                    align_items: "center",
                    justify_content: "center",
                    margin_left: MENU_TRAILING_GAP,
                    width: 18.0,
                    height: 18.0,
                    {crate::icon::icon_placeholder(chevron, 18.0, colors.foreground)}
                }
            }
            ExpandPresence {
                open: submenu_open,
                    column {
                        focus_scope: submenu_open,
                        focus_navigation: "vertical",
                        enabled: submenu_open,
                        key_capture: "left escape",
                        onkey: move |event| {
                            if event.data().is_down() && matches!(event.data().key, dioxus_elements::event::KeyboardKey::ArrowLeft | dioxus_elements::event::KeyboardKey::Escape) {
                                event.stop_propagation();
                                set_open_path.call(keyboard_parent_path.clone());
                            }
                        },
                        width: submenu_min_width.max(min_width),
                        align_self: "start",
                        align_items: "start",
                        margin_top: spacing::XXS,
                        margin_bottom: 0.0,
                        padding_top: spacing::XXS,
                        padding_right: spacing::XXS,
                        padding_bottom: spacing::XXS,
                        padding_left: spacing::XXS,
                        border_radius: theme.radii.md,
                        border_width: 1.0,
                        border_color: colors.border,
                        clip: false,
                        background_color: colors.popover,
                        shadow: shadow::LG,
                        for (child_index, child) in entry.items.iter().enumerate() {
                            {
                                render_menu_entry(
                                    child,
                                    child_index,
                                    &branch_path,
                                    MenuRenderContext {
                                        reserve_leading_slot: child_reserve_leading_slot,
                                        ..context
                                    },
                                )
                            }
                        }
                    }
            }
        }
    }
}

fn render_checkbox_entry(
    entry: &MenuCheckboxEntry,
    style: MenuStyle,
    theme: &Theme,
    on_dismiss: EventHandler<()>,
) -> Element {
    let min_width = menu_row_min_width(&style);
    let colors = &theme.colors;
    let title = entry.title.clone();
    let shortcut = entry.shortcut.clone();
    let checked = entry.checked;
    let close_on_select = entry.close_on_select;
    let on_toggle = entry.on_toggle;
    let sm = theme.radii.sm;

    rsx! {
        MenuItemSurface {
            accessibility_label: title.clone(),
            accessibility_role: "checkbox",
            accessibility_checked: Some(checked),
            width: min_width,
            height: MENU_ROW_HEIGHT,
            radius: sm,
            background: TRANSPARENT,
            interactive_background: colors.accent,
            opacity: 1.0,
            disabled: false,
            on_activate: move |_| {
                on_toggle.call(!checked);
                if close_on_select {
                    on_dismiss.call(());
                }
            },
            row {
                layout_weight: 1.0,
                clip: true,
                justify_content: "start",
                align_items: "center",
                {menu_leading_slot(rsx! {
                    if checked {
                        {arkit_icon::icon_with_stroke("check", 16.0, colors.foreground, 3.0)}
                    }
                })}
                {menu_item_text(title.clone(), colors.popover_foreground, 400)}
            }
            if let Some(shortcut) = shortcut {
                {menu_trailing_text(shortcut, colors.muted_foreground)}
            }
        }
    }
}

fn render_radio_entry(
    entry: &MenuRadioEntry,
    style: MenuStyle,
    theme: &Theme,
    on_dismiss: EventHandler<()>,
) -> Element {
    let min_width = menu_row_min_width(&style);
    let colors = &theme.colors;
    let title = entry.title.clone();
    let selected = entry.selected == entry.value;
    let value = entry.value.clone();
    let close_on_select = entry.close_on_select;
    let on_select = entry.on_select;
    let sm = theme.radii.sm;
    let full_radius = theme.radii.full;

    rsx! {
        MenuItemSurface {
            accessibility_label: title.clone(),
            accessibility_role: "radio",
            accessibility_checked: Some(selected),
            accessibility_selected: selected,
            width: min_width,
            height: MENU_ROW_HEIGHT,
            radius: sm,
            background: TRANSPARENT,
            interactive_background: colors.accent,
            opacity: 1.0,
            disabled: false,
            on_activate: move |_| {
                on_select.call(value.clone());
                if close_on_select {
                    on_dismiss.call(());
                }
            },
            row {
                layout_weight: 1.0,
                clip: true,
                justify_content: "start",
                align_items: "center",
                {menu_leading_slot(rsx! {
                    if selected {
                        row {
                            width: 8.0,
                            height: 8.0,
                            border_radius: full_radius,
                            background_color: colors.foreground,
                        }
                    }
                })}
                {menu_item_text(title.clone(), colors.popover_foreground, 400)}
            }
        }
    }
}

fn render_label_entry(
    entry: &MenuLabelEntry,
    style: MenuStyle,
    theme: &Theme,
    reserve_leading_slot: bool,
) -> Element {
    let min_width = menu_row_min_width(&style);
    let colors = &theme.colors;
    let title = entry.title.clone();
    let inset = entry.inset;
    let sm = theme.radii.sm;

    rsx! {
        row {
            accessibility_role: "heading",
            accessibility_text: title.clone(),
            accessibility_group: true,
            width: min_width,
            height: 32.0,
            align_self: "start",
            align_items: "center",
            justify_content: "start",
            padding_top: 6.0,
            padding_right: 8.0,
            padding_bottom: 6.0,
            padding_left: 8.0,
            border_radius: sm,
            clip: true,
            background_color: TRANSPARENT,
            row {
                layout_weight: 1.0,
                clip: true,
                justify_content: "start",
                align_items: "center",
                if inset || reserve_leading_slot {
                    {menu_empty_leading_slot()}
                }
                {menu_label_text(title.clone(), colors.foreground)}
            }
        }
    }
}

fn render_separator_entry(style: MenuStyle, theme: &Theme) -> Element {
    let min_width = menu_row_min_width(&style);
    let border = theme.colors.border;

    rsx! {
        row {
            width: min_width,
            height: 1.0,
            align_self: "start",
            margin_top: 4.0,
            margin_bottom: 4.0,
            background_color: border,
        }
    }
}

fn menu_item_text(content: String, color: u32, weight: i32) -> Element {
    rsx! {
        row {
            layout_weight: 1.0,
            clip: true,
            text {
                width: "100%",
                font_size: typography::SM,
                font_weight: weight,
                font_color: color,
                line_height: 20.0,
                max_lines: MENU_TEXT_MAX_LINES,
                text_overflow: MENU_TEXT_OVERFLOW_ELLIPSIS,
                {content}
            }
        }
    }
}

fn menu_label_text(content: String, color: u32) -> Element {
    rsx! {
        row {
            layout_weight: 1.0,
            clip: true,
            text {
                width: "100%",
                font_size: typography::XS,
                font_weight: 600_i32,
                font_color: color,
                line_height: 16.0,
                max_lines: MENU_TEXT_MAX_LINES,
                text_overflow: MENU_TEXT_OVERFLOW_ELLIPSIS,
                {content}
            }
        }
    }
}

fn menu_trailing_text(content: String, color: u32) -> Element {
    rsx! {
        row {
            align_items: "center",
            margin_left: MENU_TRAILING_GAP,
            text {
                font_size: typography::SM,
                font_color: color,
                line_height: 18.0,
                text_letter_spacing: 1.2f32,
                max_lines: MENU_TEXT_MAX_LINES,
                text_overflow: MENU_TEXT_OVERFLOW_ELLIPSIS,
                {content}
            }
        }
    }
}

fn menu_empty_leading_slot() -> Element {
    menu_leading_slot(rsx! {})
}

fn menu_icon_leading_slot(name: String, color: u32) -> Element {
    menu_leading_slot(rsx! {
        {crate::icon::icon_placeholder(name.as_str(), MENU_ICON_SIZE, color)}
    })
}

fn menu_leading_slot(child: Element) -> Element {
    rsx! {
        row {
            margin_right: 8.0,
            row {
                width: 16.0,
                height: 16.0,
                align_items: "center",
                justify_content: "center",
                {child}
            }
        }
    }
}

fn menu_entries_need_leading_slot(entries: &[MenuEntry]) -> bool {
    entries.iter().any(menu_entry_needs_leading_slot)
}

fn menu_entry_needs_leading_slot(entry: &MenuEntry) -> bool {
    match entry {
        MenuEntry::Action(entry) => entry.icon.is_some() || entry.inset,
        MenuEntry::Submenu(entry) => entry.icon.is_some() || entry.inset,
        MenuEntry::Checkbox(_) | MenuEntry::Radio(_) => true,
        MenuEntry::Label(entry) => entry.inset,
        MenuEntry::Separator => false,
    }
}

fn menu_branch_path(parent_path: &[usize], index: usize) -> Vec<usize> {
    let mut path = Vec::with_capacity(parent_path.len() + 1);
    path.extend_from_slice(parent_path);
    path.push(index);
    path
}

fn menu_branch_is_open(open_path: &[usize], branch_path: &[usize]) -> bool {
    open_path.len() >= branch_path.len()
        && open_path
            .iter()
            .zip(branch_path.iter())
            .all(|(open, branch)| open == branch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pc_viewport() -> arkit_hooks::OverlayViewport {
        arkit_hooks::OverlayViewport {
            frame: arkit_arkui::LayoutFramePx {
                x: 24.0,
                y: 70.0,
                width: 1000.0,
                height: 700.0,
            },
            safe_area: arkit_hooks::EdgeInsets::default(),
            scale: 2.0,
        }
    }

    fn trigger() -> arkit_arkui::LayoutFramePx {
        arkit_arkui::LayoutFramePx {
            x: 224.0,
            y: 270.0,
            width: 200.0,
            height: 40.0,
        }
    }

    #[test]
    fn menu_and_pass_through_share_the_portal_coordinate_space() {
        let viewport = pc_viewport();
        let placement = MenuOverlayPlacement::from_trigger(
            trigger(),
            viewport,
            100.0,
            80.0,
            FloatingAlign::Start,
            4.0,
        );
        let pass_through = MenuOverlayPassThroughRegion::from_frame(trigger(), viewport).unwrap();

        assert!((placement.x - 100.0).abs() < 0.01);
        assert!((placement.y - 124.0).abs() < 0.01);
        assert!((pass_through.x - 100.0).abs() < 0.01);
        assert!((pass_through.y - 100.0).abs() < 0.01);
        assert!((pass_through.width - 100.0).abs() < 0.01);
        assert!((pass_through.height - 20.0).abs() < 0.01);
    }

    #[test]
    fn context_cursor_uses_native_window_pixels_without_magnitude_guessing() {
        let pointer = dioxus_elements::event::PointerPayload {
            window_x: 224.0,
            window_y: 270.0,
            ..Default::default()
        };
        let placement =
            MenuOverlayPlacement::from_cursor(pointer, pc_viewport(), 100.0, 80.0, 4.0).unwrap();

        assert!((placement.x - 100.0).abs() < 0.01);
        assert!((placement.y - 100.0).abs() < 0.01);
    }

    #[test]
    fn centered_dropdown_uses_portal_local_coordinates() {
        let viewport = arkit_hooks::OverlayViewport {
            frame: arkit_arkui::LayoutFramePx {
                x: 0.0,
                y: 124.0,
                width: 1260.0,
                height: 2505.0,
            },
            safe_area: arkit_hooks::EdgeInsets::default(),
            scale: 3.25,
        };
        let placement = MenuOverlayPlacement::from_trigger(
            arkit_arkui::LayoutFramePx {
                x: 510.0,
                y: 467.0,
                width: 240.0,
                height: 117.0,
            },
            viewport,
            288.0,
            300.0,
            FloatingAlign::Center,
            spacing::XXS,
        );

        let window_x = viewport.frame.x + placement.x * viewport.scale;
        let window_y = viewport.frame.y + placement.y * viewport.scale;
        assert!((window_x - 162.0).abs() < 0.5);
        assert!((window_y - 597.0).abs() < 0.5);
    }

    #[test]
    fn start_aligned_menu_keeps_trigger_edge_when_space_allows() {
        let viewport = arkit_hooks::OverlayViewport {
            frame: arkit_arkui::LayoutFramePx {
                x: 0.0,
                y: 100.0,
                width: 1200.0,
                height: 2200.0,
            },
            safe_area: arkit_hooks::EdgeInsets::default(),
            scale: 3.0,
        };
        let placement = MenuOverlayPlacement::from_trigger(
            arkit_arkui::LayoutFramePx {
                x: 300.0,
                y: 400.0,
                width: 180.0,
                height: 90.0,
            },
            viewport,
            224.0,
            180.0,
            FloatingAlign::Start,
            spacing::SM,
        );

        assert!((placement.x - 100.0).abs() < 0.5);
    }
}
