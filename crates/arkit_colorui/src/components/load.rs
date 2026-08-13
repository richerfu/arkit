//! ColorUI load more + modal spinner.

use crate::theme::use_colorui_theme;
use arkit_component::components::Spinner;
use arkit_hooks::{ModalPortal, ModalPresentation};
use arkit_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadState {
    #[default]
    Loading,
    Over,
    Error,
}

#[derive(Props, Clone, PartialEq)]
pub struct LoadProps {
    #[props(default)]
    pub state: LoadState,
    pub loading_text: Option<String>,
    pub over_text: Option<String>,
    pub error_text: Option<String>,
}

#[component]
pub fn Load(props: LoadProps) -> Element {
    let theme = use_colorui_theme().tokens();
    let (icon, label) = match props.state {
        LoadState::Loading => (
            Some("loader"),
            props
                .loading_text
                .clone()
                .unwrap_or_else(|| "加载中...".into()),
        ),
        LoadState::Over => (
            Some("check"),
            props
                .over_text
                .clone()
                .unwrap_or_else(|| "没有更多了".into()),
        ),
        LoadState::Error => (
            Some("circle-x"),
            props
                .error_text
                .clone()
                .unwrap_or_else(|| "加载失败".into()),
        ),
    };
    rsx! {
        row {
            width: "100%",
            justify_content: "center",
            align_items: "center",
            padding_top: 16.0,
            padding_bottom: 16.0,
            if let Some(name) = icon {
                {arkit_icon::icon(name, 16.0, theme.colors.muted_foreground)}
            }
            text {
                content: label,
                font_size: 14.0,
                font_color: theme.colors.muted_foreground,
                margin_left: 6.0,
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct LoadModalProps {
    pub open: bool,
    pub content: Option<String>,
}

#[component]
pub fn LoadModal(props: LoadModalProps) -> Element {
    let theme = use_colorui_theme().tokens();
    rsx! {
        ModalPortal {
            open: props.open,
            presentation: ModalPresentation::CenteredDialog,
            dismiss_on_backdrop: false,
            backdrop_color: 0x80000000u32,
            on_dismiss: move |_| {},
            column {
                width: 130.0,
                height: 130.0,
                align_items: "center",
                justify_content: "center",
                background_color: theme.colors.card,
                border_radius: 10.0,
                Spinner { size: 28.0, color: Some(0xFFF37B1D) }
                text {
                    content: props.content.clone().unwrap_or_else(|| "加载中...".into()),
                    font_size: 14.0,
                    font_color: theme.colors.foreground,
                    margin_top: 12.0,
                }
            }
        }
    }
}
