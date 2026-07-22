//! Collapsible — shadcn-style expand/collapse container.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original trigger row (title + `chevrons-up-down`
//! ghost icon, space-between layout, `LG` horizontal padding), the `SM` top
//! margin on the expanded body, and the open/close toggle.

use crate::theme::*;
use arkit_prelude::*;

use super::ARKUI_BORDER_STYLE_SOLID;

/// Props for [`Collapsible`].
#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleProps {
    pub title: String,
    pub children: Element,
    /// Controlled open state. When `Some`, the collapsible is controlled.
    #[props(default)]
    pub open: Option<bool>,
    #[props(default)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: EventHandler<bool>,
}

/// A collapsible section. Clicking the header row toggles the body's
/// visibility. Supports controlled and uncontrolled open state.
#[component]
pub fn Collapsible(props: CollapsibleProps) -> Element {
    let theme = use_theme();
    let controlled = props.open.is_some();
    let mut local = use_signal(|| props.default_open);
    let open = props.open.unwrap_or_else(|| *local.read());
    let on_change = props.on_open_change;

    rsx! {
        column {
            width: "100%",
            row {
                width: "100%",
                align_items: "center",
                justify_content: "space_between",
                padding_top: 0.0,
                padding_right: spacing::LG,
                padding_bottom: 0.0,
                padding_left: spacing::LG,
                onclick: move |_| {
                    let next = !open;
                    if !controlled {
                        local.set(next);
                    }
                    on_change.call(next);
                },
                text {
                    content: props.title.clone(),
                    font_size: typography::SM,
                    font_weight: 600,
                    font_color: theme.colors.foreground,
                    line_height: 20.0,
                }
                button {
                    width: 32.0,
                    height: 32.0,
                    padding_top: 0.0,
                    padding_right: 0.0,
                    padding_bottom: 0.0,
                    padding_left: 0.0,
                    background_color: "#00000000",
                    border_width: 0.0,
                    border_style: ARKUI_BORDER_STYLE_SOLID,
                    focusable: false,
                    focus_on_touch: false,
                    alignment: "center",
                    {arkit_icon::icon("chevrons-up-down".to_string(), 16.0, theme.colors.foreground)}
                }
            }
            if open {
                arkit_animation::MountTransition {
                    preset: Some(arkit_animation::TransitionPreset::SlideUp),
                    duration_ms: Some(140),
                    row {
                        margin_top: spacing::SM,
                        {props.children}
                    }
                }
            }
        }
    }
}
