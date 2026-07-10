//! Sidebar — shadcn-style sidebar layout.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. [`Sidebar`] renders a 180px-wide panel-surfaced navigation column
//! beside its content. [`SidebarItem`] renders a `Ghost`/`Secondary` button
//! depending on the active state.

use super::button::{Button, ButtonVariant};
use crate::theme::*;
use arkit_prelude::*;

const SIDEBAR_WIDTH: f32 = 180.0;

/// Props for [`Sidebar`].
#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    pub sidebar: Element,
    pub children: Element,
}

/// A sidebar navigation panel beside main content.
#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            percent_width: 1.0,
            column {
                width: SIDEBAR_WIDTH,
                background_color: theme.colors.popover,
                border_width: 1.0,
                border_color: theme.colors.border,
                border_radius: theme.radii.md,
                shadow: 1,
                {props.sidebar}
            }
            {props.children}
        }
    }
}

/// Props for [`SidebarItem`].
#[derive(Props, Clone, PartialEq)]
pub struct SidebarItemProps {
    pub title: String,
    pub active: Option<bool>,
    pub onclick: Option<EventHandler<()>>,
}

/// A single sidebar navigation entry.
#[component]
pub fn SidebarItem(props: SidebarItemProps) -> Element {
    let active = props.active.unwrap_or(false);
    let onclick = props.onclick;
    let title = props.title.clone();
    let variant = if active {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Ghost
    };

    rsx! {
        Button {
            variant: variant,
            onclick: move |_| {
                if let Some(handler) = onclick {
                    handler.call(());
                }
            },
            "{title}"
        }
    }
}
