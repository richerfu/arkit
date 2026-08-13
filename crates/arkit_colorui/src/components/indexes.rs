//! ColorUI alphabet index rail.

use crate::theme::use_colorui_theme;
use arkit_prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct IndexesProps {
    pub letters: Vec<String>,
    pub current: Option<String>,
    pub on_select: Option<EventHandler<String>>,
}

#[component]
pub fn Indexes(props: IndexesProps) -> Element {
    let theme = use_colorui_theme().tokens();
    let current = props.current.clone();
    let on_select = props.on_select;
    rsx! {
        column {
            align_items: "center",
            padding_top: 8.0,
            padding_bottom: 8.0,
            for letter in props.letters.iter().cloned() {
                {
                    let selected = current.as_deref() == Some(letter.as_str());
                    let pick = letter.clone();
                    rsx! {
                        text {
                            content: letter,
                            font_size: 10.0,
                            font_color: if selected {
                                theme.colors.primary
                            } else {
                                theme.colors.muted_foreground
                            },
                            padding_top: 1.0,
                            padding_bottom: 1.0,
                            onclick: move |_| {
                                if let Some(handler) = on_select {
                                    handler.call(pick.clone());
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
