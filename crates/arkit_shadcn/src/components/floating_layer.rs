//! Floating overlay primitive — the dioxus counterpart of the legacy
//! `floating_panel` helper.
//!
//! The legacy implementation drove ArkUI's native floating-overlay system
//! (`floating_overlay_with_surfaces`). In the dioxus migration we render
//! inline: the trigger is mounted normally, and when `open` an optional
//! full-size outside-dismiss layer is stacked over the trigger area. Passive
//! hover surfaces remain pass-through; click-opened surfaces consume an
//! outside click to dismiss while the panel itself keeps normal interaction.
//!
//! Shared constants/enums here are consumed by the overlay components
//! (`popover`, `tooltip`, `hover_card`, `dialog`, `drawer`, `sheet`,
//! `alert_dialog`).

use arkit_prelude::*;
use dioxus_core_macro::component;

use crate::theme::spacing;

/// Backdrop color for modal overlays (50% black).
pub(crate) const OVERLAY_BACKDROP: u32 = 0x80000000u32;
/// Near-transparent paint used only to make an outside-dismiss hit plane
/// concrete on all supported ArkUI versions. It is an interaction mask, not a
/// visible backdrop.
pub(crate) const FLOATING_CAPTURE_COLOR: u32 = 0x01000000u32;
/// CSS-style hit-test keywords for the `hit_test_behavior` attribute.
pub(crate) const HIT_TEST_DEFAULT: &str = "default";
/// Skip this node in hit testing; children may still receive hits.
pub(crate) const HIT_TEST_NONE: &str = "none";
/// Small outer shadow preset (`shadow: "sm"`).
pub(crate) const SHADOW_SM: &str = "sm";

// CSS-style stack `alignment` keywords.
pub(crate) const ALIGN_TOP: &str = "top";
pub(crate) const ALIGN_START: &str = "start";
pub(crate) const ALIGN_END: &str = "end";
pub(crate) const ALIGN_BOTTOM: &str = "bottom";

/// Side of the trigger the floating panel anchors to.
///
/// Maps to an ArkUI `Alignment` int used on the capture-layer `stack`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloatingSide {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

/// Cross-axis alignment of the floating panel relative to the trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloatingAlign {
    Start,
    #[default]
    Center,
    End,
}

/// Resolve a [`FloatingSide`] to a CSS `alignment` keyword.
pub(crate) fn side_alignment(side: FloatingSide) -> &'static str {
    match side {
        FloatingSide::Top => ALIGN_TOP,
        FloatingSide::Bottom => ALIGN_BOTTOM,
        FloatingSide::Left => ALIGN_START,
        FloatingSide::Right => ALIGN_END,
    }
}

/// Resolve a side name (`"top"` / `"bottom"` / `"left"` / `"right"`) to a
/// [`FloatingSide`]. Falls back to [`FloatingSide::Bottom`].
pub(crate) fn side_from_name(name: &str) -> FloatingSide {
    match name.to_ascii_lowercase().as_str() {
        "top" => FloatingSide::Top,
        "left" => FloatingSide::Left,
        "right" => FloatingSide::Right,
        _ => FloatingSide::Bottom,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FloatingPanelPlacement {
    pub x: f32,
    pub y: f32,
}

impl FloatingPanelPlacement {
    /// shadcn-style anchor: prefer the measured **trigger root** frame.
    ///
    /// Pointer target bounds are intentionally ignored — clicks often hit an
    /// inner Text/icon node whose rect is narrower/offset from the control.
    pub(crate) fn resolve(
        trigger: arkit_hooks::LayoutFrame,
        viewport: arkit_hooks::OverlayViewport,
        panel_width: f32,
        panel_height: f32,
        side: FloatingSide,
        align: FloatingAlign,
        side_offset: f32,
    ) -> Self {
        if trigger.is_measured() {
            Self::from_trigger(
                trigger,
                viewport,
                panel_width,
                panel_height,
                side,
                align,
                side_offset,
            )
        } else {
            Self::fallback(viewport)
        }
    }

    pub(crate) fn from_trigger(
        trigger: arkit_hooks::LayoutFrame,
        viewport: arkit_hooks::OverlayViewport,
        panel_width: f32,
        panel_height: f32,
        side: FloatingSide,
        align: FloatingAlign,
        side_offset: f32,
    ) -> Self {
        let scale = viewport_scale(viewport);
        let overlay = viewport.frame;
        let (overlay_x, overlay_y, viewport_width, viewport_height) =
            overlay_metrics_vp(overlay, scale, panel_width, panel_height);

        // Trigger layout frames are physical / window-space; convert into the
        // overlay-local vp space used by ArkUI width/position attributes.
        let trigger_x = ((trigger.x - overlay_x).max(0.0)) / scale;
        let trigger_y = ((trigger.y - overlay_y).max(0.0)) / scale;
        let trigger_width = trigger.width / scale;
        let trigger_height = trigger.height / scale;

        // Keep panels on-screen with a small edge pad; do NOT force page-level
        // LG inset here — that was shifting start-aligned panels off the trigger.
        let edge = spacing::SM;
        let min_x = viewport.safe_area.left.max(0.0) + edge;
        let min_y = viewport.safe_area.top.max(0.0) + edge;
        let max_x =
            (viewport_width - viewport.safe_area.right.max(0.0) - panel_width - edge).max(min_x);
        let max_y =
            (viewport_height - viewport.safe_area.bottom.max(0.0) - panel_height - edge).max(min_y);

        let raw_x = match align {
            FloatingAlign::Start => trigger_x,
            FloatingAlign::Center => trigger_x + ((trigger_width - panel_width) / 2.0),
            FloatingAlign::End => trigger_x + trigger_width - panel_width,
        };
        let raw_y = match side {
            FloatingSide::Top => trigger_y - panel_height - side_offset,
            FloatingSide::Bottom => trigger_y + trigger_height + side_offset,
            FloatingSide::Left | FloatingSide::Right => trigger_y,
        };

        let x = clamp_preserving_start(raw_x, min_x, max_x, align);
        let y = raw_y.clamp(min_y, max_y);

        Self { x, y }
    }

    pub(crate) fn fallback(viewport: arkit_hooks::OverlayViewport) -> Self {
        Self {
            x: viewport.safe_area.left + spacing::SM,
            y: (viewport.safe_area.top + spacing::SM).max(96.0),
        }
    }
}

/// Resolve the frame a floating panel should anchor to, right now.
///
/// [`arkit_hooks::use_layout_frame`] observation only re-fires on
/// layout-driven area changes: scrolling an ancestor `scroll` moves the
/// trigger in the window without re-laying-out (or re-notifying) it, so a
/// cached frame anchors panels at the trigger's pre-scroll position. Panels
/// therefore re-query the native node on every render while open — the frame
/// is fresh exactly when the anchor matters — and fall back to the last
/// observed frame when the node cannot be queried (e.g. between mounts).
pub(crate) fn live_trigger_frame(
    reference: &arkit_arkui::NativeElementRef,
    fallback: arkit_hooks::LayoutFrame,
) -> arkit_hooks::LayoutFrame {
    reference
        .current_layout_frame()
        .filter(|frame| frame.is_measured())
        .unwrap_or(fallback)
}

/// Clamp horizontal placement while preferring start alignment when possible.
fn clamp_preserving_start(raw_x: f32, min_x: f32, max_x: f32, align: FloatingAlign) -> f32 {
    match align {
        FloatingAlign::Start => {
            if raw_x < min_x {
                min_x
            } else if raw_x > max_x {
                max_x
            } else {
                raw_x
            }
        }
        FloatingAlign::Center | FloatingAlign::End => raw_x.clamp(min_x, max_x),
    }
}

pub(crate) fn viewport_scale(viewport: arkit_hooks::OverlayViewport) -> f32 {
    if viewport.scale.is_finite() && viewport.scale > 0.0 {
        viewport.scale
    } else {
        let ratio = ohos_display_binding::default_display_virtual_pixel_ratio();
        if ratio.is_finite() && ratio > 0.0 {
            ratio
        } else {
            1.0
        }
    }
}

pub(crate) fn overlay_metrics_vp(
    overlay: arkit_hooks::LayoutFrame,
    scale: f32,
    panel_width: f32,
    panel_height: f32,
) -> (f32, f32, f32, f32) {
    let scale = scale.max(f32::EPSILON);
    if overlay.is_measured() {
        (
            overlay.x,
            overlay.y,
            (overlay.width / scale).max(panel_width + spacing::SM * 2.0),
            (overlay.height / scale).max(panel_height + spacing::SM * 2.0),
        )
    } else {
        (
            0.0,
            0.0,
            (ohos_display_binding::default_display_width() as f32 / scale)
                .max(panel_width + spacing::SM * 2.0),
            (ohos_display_binding::default_display_height() as f32 / scale)
                .max(panel_height + spacing::SM * 2.0),
        )
    }
}

/// Trigger width in vp for same-width panels (Select).
pub(crate) fn trigger_width_vp(
    trigger: arkit_hooks::LayoutFrame,
    viewport: arkit_hooks::OverlayViewport,
    fallback: f32,
) -> f32 {
    if !trigger.is_measured() {
        return fallback;
    }
    let scale = viewport_scale(viewport);
    (trigger.width / scale.max(f32::EPSILON)).max(1.0)
}

/// Generic floating layer: renders `trigger` and, when open, a capture layer
/// holding `children` (the panel) aligned to `side`.
///
/// `hover` selects the trigger activation mode: when `true`, hovering the
/// trigger opens the panel (tooltip/hover-card behaviour); otherwise the
/// trigger toggles on click (popover behaviour). Hover panels do not block the
/// application below them; click-opened panels consume one outside click to
/// dismiss without activating an obscured or retained route below.
#[component]
pub fn FloatingLayer(
    trigger: Element,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    side: Option<FloatingSide>,
    hover: Option<bool>,
    children: Element,
) -> Element {
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let hover = hover.unwrap_or(false);
    let alignment = side_alignment(side.unwrap_or_default());

    let open_up = EventHandler::new(move |_: ()| {
        if !controlled {
            internal.set(true);
        }
    });
    let toggle = EventHandler::new(move |_: ()| {
        let next = !current;
        if !controlled {
            internal.set(next);
        }
        if !next {
            if let Some(handler) = on_close {
                handler.call(());
            }
        }
    });
    let close = EventHandler::new(move |_: ()| {
        if !controlled {
            internal.set(false);
        }
        if let Some(handler) = on_close {
            handler.call(());
        }
    });

    rsx! {
        stack {
            width: "100%",
            height: "100%",
            alignment: "top",
            hit_test_behavior: "none",
            if hover {
                row {
                    onclick: move |_| toggle.call(()),
                    on_hover: move |_| open_up.call(()),
                    {trigger}
                }
            } else {
                row {
                    onclick: move |_| toggle.call(()),
                    {trigger}
                }
            }
            if current {
                if hover {
                    stack {
                        width: "100%",
                        height: "100%",
                        alignment: alignment,
                        hit_test_behavior: "none",
                        {children}
                    }
                } else {
                    stack {
                        width: "100%",
                        height: "100%",
                        background_color: FLOATING_CAPTURE_COLOR,
                        alignment: alignment,
                        hit_test_behavior: "default",
                        onclick: move |_| close.call(()),
                        stack {
                            onclick: move |evt| evt.stop_propagation(),
                            {children}
                        }
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
    fn side_alignment_maps_to_arkui_alignment_ints() {
        assert_eq!(side_alignment(FloatingSide::Top), ALIGN_TOP);
        assert_eq!(side_alignment(FloatingSide::Bottom), ALIGN_BOTTOM);
        assert_eq!(side_alignment(FloatingSide::Left), ALIGN_START);
        assert_eq!(side_alignment(FloatingSide::Right), ALIGN_END);
    }

    #[test]
    fn side_from_name_defaults_to_bottom() {
        assert_eq!(side_from_name("right"), FloatingSide::Right);
        assert_eq!(side_from_name("nonsense"), FloatingSide::Bottom);
    }

    #[test]
    fn pass_through_mode_skips_only_the_layout_shell() {
        assert_eq!(HIT_TEST_DEFAULT, "default");
        assert_eq!(HIT_TEST_NONE, "none");
    }

    #[test]
    fn placement_is_clamped_inside_safe_viewport() {
        let viewport = arkit_hooks::OverlayViewport {
            frame: arkit_hooks::LayoutFrame {
                x: 0.0,
                y: 0.0,
                width: 400.0,
                height: 800.0,
            },
            safe_area: arkit_hooks::EdgeInsets {
                top: 40.0,
                right: 10.0,
                bottom: 30.0,
                left: 20.0,
            },
            scale: 1.0,
        };
        let placement = FloatingPanelPlacement::from_trigger(
            arkit_hooks::LayoutFrame {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            viewport,
            100.0,
            50.0,
            FloatingSide::Top,
            FloatingAlign::Start,
            4.0,
        );

        assert!(placement.x >= 20.0 + spacing::SM);
        assert!(placement.y >= 40.0 + spacing::SM);
    }

    #[test]
    fn resolve_prefers_measured_trigger_over_fallback() {
        let viewport = arkit_hooks::OverlayViewport {
            frame: arkit_hooks::LayoutFrame {
                x: 0.0,
                y: 0.0,
                width: 400.0,
                height: 800.0,
            },
            safe_area: arkit_hooks::EdgeInsets::default(),
            scale: 1.0,
        };
        let trigger = arkit_hooks::LayoutFrame {
            x: 80.0,
            y: 120.0,
            width: 200.0,
            height: 40.0,
        };
        let placement = FloatingPanelPlacement::resolve(
            trigger,
            viewport,
            200.0,
            100.0,
            FloatingSide::Bottom,
            FloatingAlign::Start,
            4.0,
        );
        assert!((placement.x - 80.0).abs() < 0.5);
        assert!((placement.y - (120.0 + 40.0 + 4.0)).abs() < 0.5);
    }

    #[test]
    fn start_align_keeps_trigger_left_when_it_fits() {
        let viewport = arkit_hooks::OverlayViewport {
            frame: arkit_hooks::LayoutFrame {
                x: 0.0,
                y: 0.0,
                width: 400.0,
                height: 800.0,
            },
            safe_area: arkit_hooks::EdgeInsets {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 16.0,
            },
            scale: 1.0,
        };
        let placement = FloatingPanelPlacement::from_trigger(
            arkit_hooks::LayoutFrame {
                x: 50.0,
                y: 100.0,
                width: 180.0,
                height: 40.0,
            },
            viewport,
            180.0,
            80.0,
            FloatingSide::Bottom,
            FloatingAlign::Start,
            4.0,
        );
        assert!((placement.x - 50.0).abs() < 0.5);
    }

    #[test]
    fn trigger_width_converts_physical_to_vp() {
        let viewport = arkit_hooks::OverlayViewport {
            frame: arkit_hooks::LayoutFrame {
                x: 0.0,
                y: 0.0,
                width: 1080.0,
                height: 2400.0,
            },
            safe_area: arkit_hooks::EdgeInsets::default(),
            scale: 3.0,
        };
        let trigger = arkit_hooks::LayoutFrame {
            x: 100.0,
            y: 200.0,
            width: 540.0,
            height: 120.0,
        };
        assert!((trigger_width_vp(trigger, viewport, 180.0) - 180.0).abs() < 0.01);
        let placement = FloatingPanelPlacement::resolve(
            trigger,
            viewport,
            180.0,
            80.0,
            FloatingSide::Bottom,
            FloatingAlign::Start,
            4.0,
        );
        assert!((placement.x - (100.0 / 3.0)).abs() < 0.01);
        assert!((placement.y - (200.0 / 3.0 + 120.0 / 3.0 + 4.0)).abs() < 0.01);
    }
}
