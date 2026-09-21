//! Drawer — a panel that slides in from a screen side (default: bottom).
//!
//! The panel is projected into the root modal layer so its backdrop and edge
//! placement are independent of the caller's layout. The `side` prop accepts
//! `"top"`, `"bottom"`, `"left"`, or `"right"`; horizontal PC drawers are
//! capped at 640vp. Touch layouts retain the drag handle, while PC omits it.

use super::dialog::DialogHeader;
use super::floating_layer::{side_from_name, FloatingSide, OVERLAY_BACKDROP};
use super::motion::{
    slide_in_from, AnimatedEdgeModal, OVERLAY_ENTER_MS, OVERLAY_EXIT_MS, SHEET_DISTANCE,
};
use crate::theme::*;
use arkit_prelude::*;
use dioxus_core_macro::component;

const DRAWER_MAX_WIDTH: f32 = 640.0;

/// Drawer panel anchored to a screen side.
#[component]
pub fn Drawer(
    title: String,
    side: Option<String>,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let theme = use_theme();
    let desktop = arkit_hooks::use_adaptive_layout().is_pc();
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let side = side_from_name(side.as_deref().unwrap_or("bottom"));
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
            viewport_inset: if desktop { spacing::LG } else { 0.0 },
            panel_width: if horizontal && desktop { Some(DRAWER_MAX_WIDTH) } else { None },
            preset: Some(slide_in_from(side)),
            duration_ms: Some(OVERLAY_ENTER_MS),
            exit_duration_ms: Some(OVERLAY_EXIT_MS),
            distance: Some(SHEET_DISTANCE),
            stack {
                accessibility_role: "dialog",
                accessibility_text: title.clone(),
                width: if horizontal && desktop {
                    format!("{DRAWER_MAX_WIDTH}")
                } else {
                    "100%".to_string()
                },
                max_width: DRAWER_MAX_WIDTH,
                height: if horizontal { "100%" } else { "auto" },
                padding_top: spacing::LG,
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
                    if !desktop {
                        row {
                            width: "100%",
                            height: 24.0,
                            justify_content: "center",
                            align_items: "center",
                            onclick: move |_| close.call(()),
                            row {
                                width: 40.0,
                                height: 4.0,
                                border_radius: theme.radii.full,
                                background_color: theme.colors.muted_foreground,
                                opacity: 0.4_f32,
                            }
                        }
                    }
                    column {
                        width: "100%",
                        margin_top: if desktop { 0.0 } else { spacing::LG },
                        DialogHeader {
                            title: title.clone(),
                            description: String::new(),
                        }
                    }
                    column {
                        width: "100%",
                        margin_top: spacing::LG,
                        {children}
                    }
                }
            }
        }
    }
}
