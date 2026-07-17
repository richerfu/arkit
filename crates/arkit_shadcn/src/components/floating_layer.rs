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
/// ArkUI `HitTestMode::Default`: this node and its children own the hit.
pub(crate) const HIT_TEST_DEFAULT: i32 = 0;
/// ArkUI `HitTestMode::None`: this node is skipped but its children can hit.
pub(crate) const HIT_TEST_NONE: i32 = 3;
/// ArkUI `ShadowType::OuterDefaultSm` (small outer shadow).
pub(crate) const SHADOW_SM: i32 = 1;

// ArkUI `Alignment` enum values (used as the `alignment` int attribute on
// `stack`).
pub(crate) const ALIGN_TOP: i32 = 1;
pub(crate) const ALIGN_START: i32 = 3;
pub(crate) const ALIGN_END: i32 = 5;
pub(crate) const ALIGN_BOTTOM: i32 = 7;

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

/// Resolve a [`FloatingSide`] to its ArkUI `Alignment` int.
pub(crate) fn side_alignment(side: FloatingSide) -> i32 {
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
    pub(crate) fn from_trigger(
        trigger: arkit_hooks::LayoutFrame,
        viewport: arkit_hooks::OverlayViewport,
        panel_width: f32,
        panel_height: f32,
        side: FloatingSide,
        align: FloatingAlign,
        side_offset: f32,
    ) -> Self {
        let ratio = display_vp_ratio();
        let overlay = viewport.frame;
        let viewport_width = if overlay.is_measured() {
            overlay.width / ratio
        } else {
            ohos_display_binding::default_display_width() as f32 / ratio
        }
        .max(panel_width + (spacing::LG * 2.0));
        let viewport_height = if overlay.is_measured() {
            overlay.height / ratio
        } else {
            ohos_display_binding::default_display_height() as f32 / ratio
        }
        .max(panel_height + (spacing::LG * 2.0));

        let overlay_x = if overlay.is_measured() {
            overlay.x
        } else {
            0.0
        };
        let overlay_y = if overlay.is_measured() {
            overlay.y
        } else {
            0.0
        };
        let trigger_x = (trigger.x - overlay_x).max(0.0) / ratio;
        let trigger_y = (trigger.y - overlay_y).max(0.0) / ratio;
        let trigger_width = trigger.width / ratio;
        let trigger_height = trigger.height / ratio;
        let min_x = viewport.safe_area.left + spacing::LG;
        let min_y = viewport.safe_area.top + spacing::LG;
        let max_x =
            (viewport_width - viewport.safe_area.right - panel_width - spacing::LG).max(min_x);
        let max_y =
            (viewport_height - viewport.safe_area.bottom - panel_height - spacing::LG).max(min_y);

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

        Self {
            x: raw_x.clamp(min_x, max_x),
            y: raw_y.clamp(min_y, max_y),
        }
    }

    pub(crate) fn from_pointer(
        pointer: dioxus_elements::event::PointerPayload,
        viewport: arkit_hooks::OverlayViewport,
        panel_width: f32,
        panel_height: f32,
        side: FloatingSide,
        align: FloatingAlign,
        side_offset: f32,
    ) -> Option<Self> {
        let trigger = if pointer.has_target_bounds() {
            arkit_hooks::LayoutFrame {
                x: pointer.target_x,
                y: pointer.target_y,
                width: pointer.target_width,
                height: pointer.target_height,
            }
        } else if pointer.has_window_position() {
            arkit_hooks::LayoutFrame {
                x: pointer.window_x,
                y: pointer.window_y,
                width: 1.0,
                height: 1.0,
            }
        } else {
            return None;
        };
        Some(Self::from_trigger(
            trigger,
            viewport,
            panel_width,
            panel_height,
            side,
            align,
            side_offset,
        ))
    }

    pub(crate) fn fallback(viewport: arkit_hooks::OverlayViewport) -> Self {
        Self {
            x: viewport.safe_area.left + spacing::LG,
            y: (viewport.safe_area.top + spacing::LG).max(96.0),
        }
    }
}

fn display_vp_ratio() -> f32 {
    let ratio = ohos_display_binding::default_display_virtual_pixel_ratio();
    if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    }
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
            percent_width: 1.0,
            percent_height: 1.0,
            alignment: ALIGN_TOP,
            hit_test_behavior: HIT_TEST_NONE,
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
                        percent_width: 1.0,
                        percent_height: 1.0,
                        alignment: alignment,
                        hit_test_behavior: HIT_TEST_NONE,
                        {children}
                    }
                } else {
                    stack {
                        percent_width: 1.0,
                        percent_height: 1.0,
                        background_color: FLOATING_CAPTURE_COLOR,
                        alignment: alignment,
                        hit_test_behavior: HIT_TEST_DEFAULT,
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
        assert_eq!(HIT_TEST_DEFAULT, 0);
        assert_eq!(HIT_TEST_NONE, 3);
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

        assert!(placement.x >= 20.0 + spacing::LG);
        assert!(placement.y >= 40.0 + spacing::LG);
    }
}
