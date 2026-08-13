//! ColorUI form group row.

use crate::theme::use_colorui_theme;
use arkit_prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct FormGroupProps {
    pub title: String,
    pub children: Element,
}

#[component]
pub fn FormGroup(props: FormGroupProps) -> Element {
    let theme = use_colorui_theme().tokens();
    rsx! {
        row {
            width: "100%",
            min_height: 50.0,
            padding_left: 15.0,
            padding_right: 15.0,
            align_items: "center",
            justify_content: "space-between",
            background_color: theme.colors.card,
            text {
                content: props.title.clone(),
                font_size: 15.0,
                font_color: theme.colors.foreground,
                margin_right: 12.0,
            }
            row {
                layout_weight: 1.0,
                justify_content: "end",
                align_items: "center",
                {props.children}
            }
        }
    }
}
