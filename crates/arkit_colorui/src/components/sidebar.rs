//! Sidebar — ColorUI list panel, not a shadcn card.

use arkit_component::appearance::ButtonVariant;
use arkit_prelude::*;

use super::primitives::Button;
use crate::spec;

#[component]
pub fn Sidebar(sidebar: Element, children: Element) -> Element {
    rsx! {
        row {
            width: "100%",
            column {
                width: spec::DRAWER_WIDTH,
                background_color: spec::BG_WHITE,
                {sidebar}
            }
            {children}
        }
    }
}

#[component]
pub fn SidebarItem(
    title: String,
    active: Option<bool>,
    onclick: Option<EventHandler<()>>,
) -> Element {
    let active = active.unwrap_or(false);
    rsx! {
        Button {
            variant: if active {
                ButtonVariant::Default
            } else {
                ButtonVariant::Ghost
            },
            width: Some("100%".into()),
            onclick,
            "{title}"
        }
    }
}
