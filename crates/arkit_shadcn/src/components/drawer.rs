//! Drawer — a panel that slides in from a screen side (default: bottom).
//!
//! The panel is projected into the root modal layer so its backdrop and edge
//! placement are independent of the caller's layout. The `side` prop accepts
//! `"top"`, `"bottom"`, `"left"`, or `"right"`. Top and bottom drawers span
//! the viewport and stay attached to their edge; side drawers use a compact
//! desktop width. The swipe handle is reserved for bottom drawers.

use super::floating_layer::{side_from_name, FloatingSide, OVERLAY_BACKDROP};
use super::motion::{
    slide_in_from, AnimatedEdgeModal, OVERLAY_ENTER_MS, OVERLAY_EXIT_MS, SHEET_DISTANCE,
};
use crate::theme::*;
use arkit_prelude::*;
use dioxus_core_macro::component;

const DRAWER_SIDE_WIDTH: f32 = 384.0;

fn drawer_border_radius(side: FloatingSide, radius: f32) -> String {
    // ArkUI's four-value order is top-left, top-right, bottom-left,
    // bottom-right. Corners touching the viewport edge remain square.
    match side {
        FloatingSide::Top => format!("0,0,{radius},{radius}"),
        FloatingSide::Bottom => format!("{radius},{radius},0,0"),
        FloatingSide::Left | FloatingSide::Right => "0".to_string(),
    }
}

fn drawer_border_width(side: FloatingSide) -> &'static str {
    match side {
        FloatingSide::Top => "0,0,1,0",
        FloatingSide::Bottom => "1,0,0,0",
        FloatingSide::Left => "0,1,0,0",
        FloatingSide::Right => "0,0,0,1",
    }
}

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
            viewport_inset: 0.0,
            panel_width: if horizontal && desktop { Some(DRAWER_SIDE_WIDTH) } else { None },
            panel_width_fraction: if horizontal && !desktop { Some(0.75) } else { None },
            preset: Some(slide_in_from(side)),
            duration_ms: Some(OVERLAY_ENTER_MS),
            exit_duration_ms: Some(OVERLAY_EXIT_MS),
            distance: Some(SHEET_DISTANCE),
            stack {
                accessibility_role: "dialog",
                accessibility_text: title.clone(),
                width: "100%",
                height: if horizontal { "100%" } else { "auto" },
                max_height: if horizontal { "100%" } else { "80%" },
                padding_top: spacing::LG,
                padding_right: spacing::LG,
                padding_bottom: spacing::LG,
                padding_left: spacing::LG,
                border_radius: drawer_border_radius(side, theme.radii.lg),
                border_width: drawer_border_width(side),
                border_color: theme.colors.border,
                background_color: theme.colors.background,
                shadow: "sm",
                column {
                    width: "100%",
                    align_items: "stretch",
                    if side == FloatingSide::Bottom {
                        row {
                            width: "100%",
                            height: 24.0,
                            justify_content: "center",
                            align_items: "center",
                            row {
                                width: 100.0,
                                height: 8.0,
                                border_radius: theme.radii.full,
                                background_color: theme.colors.muted,
                            }
                        }
                    }
                    column {
                        width: "100%",
                accessibility_role: "dialog",
                accessibility_text: title.clone(),
                        align_items: "stretch",
                        margin_top: if horizontal { 0.0 } else { spacing::SM },
                        text {
                            width: "100%",
                            font_size: typography::LG,
                            font_weight: 600_i32,
                            font_color: theme.colors.foreground,
                            line_height: 24.0,
                            text_align: if !desktop && !horizontal { "center" } else { "start" },
                            "{title}"
                        }
                    }
                    column {
                        width: "100%",
                        align_items: "stretch",
                        margin_top: spacing::LG,
                        {children}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_only_the_corners_away_from_the_viewport_edge() {
        assert_eq!(drawer_border_radius(FloatingSide::Top, 8.0), "0,0,8,8");
        assert_eq!(drawer_border_radius(FloatingSide::Bottom, 8.0), "8,8,0,0");
        assert_eq!(drawer_border_radius(FloatingSide::Left, 8.0), "0");
        assert_eq!(drawer_border_radius(FloatingSide::Right, 8.0), "0");
    }

    #[test]
    fn draws_only_the_border_facing_the_content() {
        assert_eq!(drawer_border_width(FloatingSide::Top), "0,0,1,0");
        assert_eq!(drawer_border_width(FloatingSide::Bottom), "1,0,0,0");
        assert_eq!(drawer_border_width(FloatingSide::Left), "0,1,0,0");
        assert_eq!(drawer_border_width(FloatingSide::Right), "0,0,0,1");
    }
}
