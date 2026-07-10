//! NavigationMenu — shadcn-style horizontal navigation menu.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. [`NavigationMenu`] renders a `shadow-sm` column with `SM` padding,
//! 1px `border`, `lg` radius, `background` fill, containing a row of items.
//! [`NavigationItem`] renders a `Ghost`/`Secondary` button depending on the
//! active state.

use super::button::{Button, ButtonVariant};
use crate::theme::*;
use arkit_prelude::*;

/// Props for [`NavigationMenu`].
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuProps {
    pub children: Element,
}

/// A horizontal navigation menu container.
#[component]
pub fn NavigationMenu(props: NavigationMenuProps) -> Element {
    let theme = use_theme();
    rsx! {
        column {
            padding_top: spacing::SM,
            padding_right: spacing::SM,
            padding_bottom: spacing::SM,
            padding_left: spacing::SM,
            border_radius: theme.radii.lg,
            border_width: 1.0,
            border_color: theme.colors.border,
            background_color: theme.colors.background,
            shadow: 1,
            row {
                {props.children}
            }
        }
    }
}

/// Props for [`NavigationItem`].
#[derive(Props, Clone, PartialEq)]
pub struct NavigationItemProps {
    pub title: String,
    pub active: Option<bool>,
    pub onclick: Option<EventHandler<()>>,
}

/// A single navigation menu entry.
#[component]
pub fn NavigationItem(props: NavigationItemProps) -> Element {
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
