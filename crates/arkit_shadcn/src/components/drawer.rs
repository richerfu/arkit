//! Drawer — a panel that slides in from a screen side (default: bottom).
//!
//! Migrated from the legacy Elm builder API. The `side` prop (`"top"` /
//! `"bottom"` / `"left"` / `"right"`) selects the capture-layer alignment
//! (left=3, right=5, top=1, bottom=7). The panel preserves the original
//! styling: `DRAWER_MAX_WIDTH` 640 cap, `spacing` padding
//! (`[LG, XXL, XXL, XXL]`), `lg` radius, 1px top border, `background`/`border`
//! tokens, small outer shadow, and a 40×4 drag handle (full radius,
//! `muted_foreground` at 0.4 opacity) that dismisses on tap.

use super::dialog::DialogHeader;
use super::floating_layer::{side_alignment, side_from_name, OVERLAY_BACKDROP};
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
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let side = side_from_name(side.as_deref().unwrap_or("bottom"));
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
                width: "100%",
                max_width: DRAWER_MAX_WIDTH,
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
                    column {
                        width: "100%",
                        margin_top: spacing::LG,
                        DialogHeader {
                            title: title,
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
