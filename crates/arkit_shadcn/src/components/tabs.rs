//! Tabs — shadcn-style tabbed navigation.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original tabs-list surface (`muted` background, `lg`
//! radius, `36.0` height, `3.0` padding), the trigger styling (`30.0` height,
//! `md` radius, transparent border, active = `background`), and the
//! visibility-toggled panel stack.

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
            align_items: "center",
            justify_content: "center",
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
    pub on_press: EventHandler<()>,
}

/// A single tab trigger. Highlights with the `background` color when active.
#[component]
pub fn TabsTrigger(props: TabsTriggerProps) -> Element {
    let theme = use_theme();
    let background = if props.active {
        theme.colors.background
    } else {
        TRANSPARENT
    };
    let on_press = props.on_press;
    rsx! {
        row {
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
            onclick: move |_| on_press.call(()),
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
            percent_width: 1.0,
            visibility: if props.active { 0 } else { 2 },
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

    let triggers: Vec<Element> = props
        .labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let mut local = local;
            rsx! {
                TabsTrigger {
                    key: "{index}",
                    label: label.clone(),
                    active: active == index,
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
            percent_width: 1.0,
            TabsList {
                {triggers.into_iter()}
            }
            row {
                margin_top: spacing::SM,
                stack {
                    percent_width: 1.0,
                    {panels.into_iter()}
                }
            }
        }
    }
}
