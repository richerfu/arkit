//! Sheet — a panel that slides in from a screen side (default: right).
//!
//! The panel is projected into the root modal layer, is 384vp wide on PC for
//! left/right presentation, and has a close (`✕`) button instead of a drag
//! handle. The `side` prop accepts `"top"`, `"bottom"`, `"left"`, or `"right"`.
//! Side sheets follow shadcn's flush viewport-edge shape rather than inheriting
//! the caller's rounded content canvas.

use super::dialog::DialogHeader;
use super::floating_layer::{side_from_name, FloatingSide, OVERLAY_BACKDROP};
use super::motion::{
    slide_in_from, AnimatedEdgeModal, OVERLAY_ENTER_MS, OVERLAY_EXIT_MS, SHEET_DISTANCE,
};
use super::ARKUI_BORDER_STYLE_SOLID;
use crate::icon::icon_placeholder;
use crate::theme::*;
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
    let adaptive = arkit_hooks::use_adaptive_layout();
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let side = side_from_name(side.as_deref().unwrap_or("right"));
    let horizontal = matches!(side, FloatingSide::Left | FloatingSide::Right);

    let close = EventHandler::new(move |_: ()| {
        if !controlled {
            internal.set(false);
        }
        if let Some(handler) = on_close {
            handler.call(());
        }
    });

    rsx! {
        AnimatedEdgeModal {
            open: current,
            side,
            on_dismiss: close,
            backdrop_color: OVERLAY_BACKDROP,
            viewport_inset: 0.0,
            panel_width: if horizontal && adaptive.is_pc() { Some(SHEET_WIDTH) } else { None },
            preset: Some(slide_in_from(side)),
            duration_ms: Some(OVERLAY_ENTER_MS),
            exit_duration_ms: Some(OVERLAY_EXIT_MS),
            distance: Some(SHEET_DISTANCE),
            stack {
                accessibility_role: "dialog",
                accessibility_text: title.clone(),
                width: if horizontal && adaptive.is_pc() {
                    format!("{SHEET_WIDTH}")
                } else {
                    "100%".to_string()
                },
                max_width: if horizontal && adaptive.is_pc() { SHEET_WIDTH },
                height: if horizontal { "100%" } else { "auto" },
                padding_top: spacing::XXL,
                padding_right: spacing::XXL,
                padding_bottom: spacing::XXL,
                padding_left: spacing::XXL,
                border_radius: 0.0,
                border_width: 1.0,
                border_color: theme.colors.border,
                background_color: theme.colors.background,
                shadow: "sm",
                column {
                    width: "100%",
                    height: "100%",
                    DialogHeader {
                        title: title.clone(),
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
                        accessibility_text: "Close sheet",
                        width: 28.0,
                        height: 28.0,
                        padding: 0.0,
                        background_color: "#00000000",
                        border_width: 0.0,
                        border_style: ARKUI_BORDER_STYLE_SOLID,
                        border_radius: theme.radii.md,
                        clip: true,
                        focusable: true,
                        focus_on_touch: true,
                        alignment: "center",
                        opacity: 0.7_f32,
                        onclick: move |_| close.call(()),
                        onkey: move |event| {
                            if event.data().activates() {
                                close.call(());
                            }
                        },
                        {icon_placeholder("x", 16.0, theme.colors.foreground)}
                    }
                }
            }
        }
    }
}
