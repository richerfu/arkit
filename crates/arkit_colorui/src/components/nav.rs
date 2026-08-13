//! ColorUI scroll / underline nav.

use arkit_component::style::PaletteColor;

use crate::theme::use_colorui_theme;
use arkit_prelude::*;

use crate::theme::swatch;

#[derive(Props, Clone, PartialEq)]
pub struct NavProps {
    pub children: Element,
}

#[component]
pub fn Nav(props: NavProps) -> Element {
    let theme = use_colorui_theme().tokens();
    rsx! {
        scroll {
            width: "100%",
            scroll_enabled: true,
            row {
                width: "100%",
                background_color: theme.colors.card,
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NavItemProps {
    pub title: String,
    #[props(default)]
    pub current: bool,
    #[props(default)]
    pub color: PaletteColor,
    pub onclick: Option<EventHandler<()>>,
}

#[component]
pub fn NavItem(props: NavItemProps) -> Element {
    let theme = use_colorui_theme().tokens();
    let hue = swatch(props.color);
    let ink = if props.current {
        hue.fill
    } else {
        theme.colors.foreground
    };
    let onclick = props.onclick;
    rsx! {
        column {
            padding_left: 10.0,
            padding_right: 10.0,
            align_items: "center",
            onclick: move |_| {
                if let Some(handler) = onclick {
                    handler.call(());
                }
            },
            text {
                content: props.title.clone(),
                height: 45.0,
                font_size: 15.0,
                font_color: ink,
            }
            row {
                width: 16.0,
                height: 2.0,
                background_color: if props.current { hue.fill } else { 0x00000000 },
            }
        }
    }
}
