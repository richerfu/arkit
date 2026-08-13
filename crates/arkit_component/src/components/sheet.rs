//! Sheet — a panel that slides in from a screen side (default: right).
//!
//! Migrated from the legacy Elm builder API. Same shape as `Drawer` but
//! narrow (`SHEET_WIDTH` 384), full-height, with a close (`✕`) button instead
//! of a drag handle. The `side` prop (`"top"` / `"bottom"` / `"left"` /
//! `"right"`) selects the capture-layer alignment (left=3, right=5, top=1,
//! bottom=7). Original styling preserved: `spacing::XXL` padding, `lg` radius,
//! 1px border, `background`/`border` tokens, small outer shadow.

use super::dialog::DialogHeader;
use super::floating_layer::{side_alignment, side_from_name, OVERLAY_BACKDROP};
use super::ARKUI_BORDER_STYLE_SOLID;
use crate::icon::icon_placeholder;
use crate::style::*;
use arkit_prelude::*;
use dioxus_core_macro::component;

const SHEET_WIDTH: f32 = 384.0;

/// Sheet panel anchored to a screen side.
#[component]
pub fn Sheet(
    title: String,
    side: Option<String>,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let theme = use_theme();
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let side = side_from_name(side.as_deref().unwrap_or("right"));
    let alignment = side_alignment(side);

    let close = EventHandler::new(move |_: ()| {
        if !controlled {
            internal.set(false);
        }
        if let Some(handler) = on_close {
            handler.call(());
        }
    });

    if !current {
        return rsx! {};
    }

    rsx! {
        stack {
            width: "100%",
            height: "100%",
            background_color: OVERLAY_BACKDROP,
            alignment: alignment,
            onclick: move |_| close.call(()),
            stack {
                onclick: move |evt| { evt.stop_propagation(); },
                width: SHEET_WIDTH,
                height: "100%",
                padding_top: spacing::XXL,
                padding_right: spacing::XXL,
                padding_bottom: spacing::XXL,
                padding_left: spacing::XXL,
                border_radius: theme.radii.lg,
                border_width: 1.0,
                border_color: theme.colors.border,
                background_color: theme.colors.background,
                shadow: "sm",
                column {
                    width: "100%",
                    height: "100%",
                    DialogHeader {
                        title: title,
                        description: String::new(),
                    }
                    column {
                        width: "100%",
                        margin_top: spacing::LG,
                        {children}
                    }
                }
                row {
                    width: "100%",
                    position: 0.0,
                    justify_content: "end",
                    button {
                        button_type: "normal",
                        width: 28.0,
                        height: 28.0,
                        padding: 0.0,
                        background_color: "#00000000",
                        border_width: 0.0,
                        border_style: ARKUI_BORDER_STYLE_SOLID,
                        border_radius: theme.radii.md,
                        clip: true,
                        focusable: false,
                        focus_on_touch: false,
                        alignment: "center",
                        opacity: 0.7_f32,
                        onclick: move |_| close.call(()),
                        {icon_placeholder("x", 16.0, theme.colors.foreground)}
                    }
                }
            }
        }
    }
}
