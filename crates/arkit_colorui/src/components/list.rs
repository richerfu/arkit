//! ColorUI list — menu, avatar-menu, and icon grid.

use crate::theme::use_colorui_theme;
use arkit_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListKind {
    #[default]
    Menu,
    Avatar,
    Grid,
}

#[derive(Props, Clone, PartialEq)]
pub struct ListProps {
    #[props(default)]
    pub kind: ListKind,
    #[props(default = 4)]
    pub columns: u32,
    pub children: Element,
}

#[component]
pub fn List(props: ListProps) -> Element {
    let theme = use_colorui_theme().tokens();
    match props.kind {
        ListKind::Grid => rsx! {
            flex {
                width: "100%",
                flex_wrap: "wrap",
                background_color: theme.colors.card,
                {props.children}
            }
        },
        _ => rsx! {
            column {
                width: "100%",
                background_color: theme.colors.card,
                {props.children}
            }
        },
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ListItemProps {
    pub title: String,
    pub note: Option<String>,
    pub icon: Option<String>,
    pub avatar: Option<String>,
    pub arrow: Option<bool>,
    pub onclick: Option<EventHandler<()>>,
    pub action: Option<Element>,
}

#[component]
pub fn ListItem(props: ListItemProps) -> Element {
    let theme = use_colorui_theme().tokens();
    let onclick = props.onclick;
    rsx! {
        row {
            width: "100%",
            min_height: 50.0,
            padding_left: 15.0,
            padding_right: 15.0,
            align_items: "center",
            justify_content: "space-between",
            background_color: theme.colors.card,
            border_width: 0.0,
            onclick: move |_| {
                if let Some(handler) = onclick {
                    handler.call(());
                }
            },
            row {
                align_items: "center",
                layout_weight: 1.0,
                if let Some(src) = props.avatar.clone() {
                    image {
                        src,
                        width: 40.0,
                        height: 40.0,
                        border_radius: 999.0,
                        margin_right: 12.0,
                        object_fit: "cover",
                    }
                } else if let Some(icon) = props.icon.as_ref() {
                    row {
                        width: 28.0,
                        margin_right: 8.0,
                        align_items: "center",
                        justify_content: "center",
                        {arkit_icon::icon(icon, 20.0, theme.colors.primary)}
                    }
                }
                column {
                    align_items: "start",
                    text {
                        content: props.title.clone(),
                        font_size: 15.0,
                        font_color: theme.colors.foreground,
                    }
                    if let Some(note) = props.note.clone() {
                        text {
                            content: note,
                            font_size: 12.0,
                            font_color: theme.colors.muted_foreground,
                            margin_top: 2.0,
                        }
                    }
                }
            }
            row {
                align_items: "center",
                if let Some(action) = props.action {
                    {action}
                }
                if props.arrow.unwrap_or(false) {
                    text {
                        content: "›",
                        font_size: 18.0,
                        font_color: 0xFF8799A3u32,
                        margin_left: 6.0,
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct GridListProps {
    #[props(default = 4)]
    pub columns: u32,
    pub children: Element,
}

#[component]
pub fn GridList(props: GridListProps) -> Element {
    rsx! {
        List { kind: ListKind::Grid, columns: props.columns, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct GridItemProps {
    pub title: String,
    pub icon: Option<String>,
    #[props(default = 4)]
    pub columns: u32,
    pub onclick: Option<EventHandler<()>>,
}

#[component]
pub fn GridItem(props: GridItemProps) -> Element {
    let theme = use_colorui_theme().tokens();
    let width = format!("{}%", 100.0 / props.columns.max(1) as f32);
    let onclick = props.onclick;
    rsx! {
        column {
            width,
            padding_top: 16.0,
            padding_bottom: 16.0,
            align_items: "center",
            justify_content: "center",
            onclick: move |_| {
                if let Some(handler) = onclick {
                    handler.call(());
                }
            },
            if let Some(icon) = props.icon.as_ref() {
                {arkit_icon::icon(icon, 24.0, theme.colors.primary)}
            }
            text {
                content: props.title.clone(),
                font_size: 13.0,
                font_color: theme.colors.muted_foreground,
                margin_top: 6.0,
            }
        }
    }
}
