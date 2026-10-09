//! Tabs — shadcn-style tabbed navigation.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. The tabs list fills its parent width with equal-weight triggers.
//! List surface is `muted` at `36.0` height with `3.0` padding; active trigger
//! uses `background` fill, `md` radius, and a transparent border. Panels stay
//! mounted and toggle visibility.

use crate::theme::*;
use arkit_prelude::*;

const TRANSPARENT: u32 = 0x00000000;
const TABS_LIST_HEIGHT: f32 = 36.0;
const TABS_LIST_PADDING: f32 = 3.0;
const TABS_TRIGGER_HEIGHT: f32 = TABS_LIST_HEIGHT - (TABS_LIST_PADDING * 2.0);

/// Props for [`TabsList`].
#[derive(Props, Clone, PartialEq)]
pub struct TabsListProps {
    pub children: Element,
}

/// The rounded container holding tab triggers.
#[component]
pub fn TabsList(props: TabsListProps) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            accessibility_role: "tablist",
            width: "100%",
            align_items: "center",
            justify_content: "start",
            padding: TABS_LIST_PADDING,
            height: TABS_LIST_HEIGHT,
            border_radius: theme.radii.lg,
            background_color: theme.colors.muted,
            {props.children}
        }
    }
}

/// Props for [`TabsTrigger`].
#[derive(Props, Clone, PartialEq)]
pub struct TabsTriggerProps {
    pub label: String,
    pub active: bool,
    #[props(default)]
    pub native_ref: Option<arkit_arkui::NativeElementRef>,
    #[props(default)]
    pub on_navigation: Option<EventHandler<dioxus_elements::event::KeyData>>,
    #[props(default)]
    pub on_press: EventHandler<()>,
}

/// A single tab trigger. Highlights with the `background` color when active.
#[component]
pub fn TabsTrigger(props: TabsTriggerProps) -> Element {
    let theme = use_theme();
    let mut hovering = use_signal(|| false);
    let background = if props.active {
        theme.colors.background
    } else if hovering() {
        theme.colors.accent
    } else {
        TRANSPARENT
    };
    let on_press = props.on_press;
    rsx! {
        row {
            native_ref: props.native_ref,
            accessibility_role: "tab",
            accessibility_text: props.label.clone(),
            accessibility_group: true,
            accessibility_actions: "click",
            accessibility_selected: props.active,
            focusable: true,
            tab_stop: props.active,
            key_capture: if props.on_navigation.is_some() { "enter space left right home end" } else { "enter space" },
            layout_weight: 1.0,
            height: TABS_TRIGGER_HEIGHT,
            align_items: "center",
            justify_content: "center",
            padding_top: spacing::XXS,
            padding_right: spacing::SM,
            padding_bottom: spacing::XXS,
            padding_left: spacing::SM,
            border_radius: theme.radii.md,
            border_width: 1.0,
            border_color: TRANSPARENT,
            background_color: background,
            focus_on_touch: true,
            shadow: if props.active { shadow::SM } else { "none" },
            onclick: move |_| on_press.call(()),
            onkey: move |event| {
                if event.data().activates() {
                    event.stop_propagation();
                    on_press.call(());
                } else if event.data().is_down() {
                    if let Some(handler) = props.on_navigation { handler.call(event.data().as_ref().clone()); }
                }
            },
            onhover: move |event| hovering.set(event.data().is_hovering),
            text {
                content: props.label.clone(),
                font_size: typography::SM,
                font_weight: 500,
                font_color: theme.colors.foreground,
                line_height: 20.0,
            }
        }
    }
}

/// Props for [`TabsContent`].
#[derive(Props, Clone, PartialEq)]
pub struct TabsContentProps {
    /// Whether this panel is the active one (others are hidden via `None`).
    pub active: bool,
    pub children: Element,
}

/// A tab panel. Kept mounted; visibility is toggled so layout stays stable.
#[component]
pub fn TabsContent(props: TabsContentProps) -> Element {
    rsx! {
        column {
            accessibility_role: "group",
            accessibility_mode: if props.active { "auto" } else { "disabled_for_descendants" },
            width: "100%",
            visibility: if props.active { "visible" } else { "none" },
            {props.children}
        }
    }
}

/// Props for [`Tabs`].
#[derive(Props, Clone, PartialEq)]
pub struct TabsProps {
    pub labels: Vec<String>,
    pub panels: Vec<Element>,
    /// Controlled active index. When `Some`, the tabs are controlled.
    #[props(default)]
    pub active: Option<usize>,
    #[props(default)]
    pub default_active: usize,
    #[props(default)]
    pub on_change: EventHandler<usize>,
}

/// A complete tabbed container — renders a [`TabsList`] of [`TabsTrigger`]s
/// and a stack of [`TabsContent`] panels, toggling visibility by active index.
#[component]
pub fn Tabs(props: TabsProps) -> Element {
    let controlled = props.active.is_some();
    let local = use_signal(|| props.default_active);
    let active = props.active.unwrap_or_else(|| *local.read());
    let on_change = props.on_change;
    let tab_count = props.labels.len();
    let focus = arkit_hooks::use_keyboard_focus_list(tab_count);

    let triggers: Vec<Element> = props
        .labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let mut local = local;
            let navigation = focus.clone();
            rsx! {
                TabsTrigger {
                    key: "{index}",
                    label: label.clone(),
                    active: active == index,
                    native_ref: Some(focus.native_ref(index)),
                    on_navigation: move |key: dioxus_elements::event::KeyData| {
                        if let Some(next) = navigation.navigate(index, key.key, dioxus_elements::event::KeyboardNavigation::Horizontal, &vec![true; tab_count]) {
                            if !controlled { local.set(next); }
                            on_change.call(next);
                        }
                    },
                    on_press: move |_| {
                        if !controlled {
                            local.set(index);
                        }
                        on_change.call(index);
                    },
                }
            }
        })
        .collect();

    let panels: Vec<Element> = props
        .panels
        .iter()
        .enumerate()
        .map(|(index, panel)| {
            rsx! {
                TabsContent {
                    key: "{index}",
                    active: active == index,
                    {panel.clone()}
                }
            }
        })
        .collect();

    rsx! {
        column {
            width: "100%",
            TabsList {
                {triggers.into_iter()}
            }
            row {
                margin_top: spacing::SM,
                stack {
                    width: "100%",
                    {panels.into_iter()}
                }
            }
        }
    }
}
