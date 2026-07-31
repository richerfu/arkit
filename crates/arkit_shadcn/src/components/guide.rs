//! Guide — a multi-step product tour anchored to measured page targets.
//!
//! `GuideTarget` gives its render callback an exact native ref and registers
//! the frame of the element carrying that ref without adding a layout wrapper.
//! `Guide` owns the controlled/uncontrolled step lifecycle and declares a
//! root-projected spotlight and anchored explanation panel.

use std::cell::RefCell;
use std::rc::Rc;

use super::floating_layer::{
    viewport_scale, FloatingSide, FLOATING_CAPTURE_COLOR, HIT_TEST_DEFAULT, HIT_TEST_NONE,
    SHADOW_SM,
};
use super::{Button, ButtonSize, ButtonVariant};
use crate::i18n::use_component_i18n;
use crate::theme::*;
use arkit_prelude::*;

const GUIDE_DEFAULT_PANEL_WIDTH: f32 = 320.0;
const GUIDE_ESTIMATED_PANEL_HEIGHT: f32 = 220.0;
const GUIDE_DEFAULT_SPOTLIGHT_PADDING: f32 = 8.0;
const GUIDE_DEFAULT_SIDE_OFFSET: f32 = 12.0;
const GUIDE_DEFAULT_BACKDROP: u32 = 0xA6000000;

/// Preferred side of the highlighted target for the guide panel.
///
/// The panel automatically flips to the opposite side when the preferred side
/// does not have enough room.
pub type GuideSide = FloatingSide;

/// One guide step and the target it describes.
#[derive(Debug, Clone, PartialEq)]
pub struct GuideStep {
    /// Identifier of a descendant [`GuideTarget`].
    pub target: String,
    pub title: String,
    pub description: String,
    pub side: GuideSide,
}

impl GuideStep {
    pub fn new(
        target: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            target: target.into(),
            title: title.into(),
            description: description.into(),
            side: GuideSide::Bottom,
        }
    }

    pub const fn side(mut self, side: GuideSide) -> Self {
        self.side = side;
        self
    }
}

/// Built-in action labels. Omit this prop to use the active component locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuideLabels {
    pub previous: String,
    pub next: String,
    pub skip: String,
    pub finish: String,
}

/// Geometry and mask styling for [`Guide`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuideStyle {
    pub panel_width: f32,
    /// Stable height estimate used for collision-aware placement.
    pub estimated_panel_height: f32,
    pub spotlight_padding: f32,
    pub side_offset: f32,
    pub backdrop_color: u32,
}

impl Default for GuideStyle {
    fn default() -> Self {
        Self {
            panel_width: GUIDE_DEFAULT_PANEL_WIDTH,
            estimated_panel_height: GUIDE_ESTIMATED_PANEL_HEIGHT,
            spotlight_padding: GUIDE_DEFAULT_SPOTLIGHT_PADDING,
            side_offset: GUIDE_DEFAULT_SIDE_OFFSET,
            backdrop_color: GUIDE_DEFAULT_BACKDROP,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct GuideProps {
    pub steps: Vec<GuideStep>,
    /// Controlled open state. Omit to let the component own the lifecycle.
    pub open: Option<bool>,
    #[props(default)]
    pub default_open: bool,
    /// Controlled zero-based step index. Omit to use internal state.
    pub step: Option<usize>,
    #[props(default)]
    pub default_step: usize,
    pub labels: Option<GuideLabels>,
    #[props(default)]
    pub style: GuideStyle,
    /// Let pointer events reach the highlighted target through the mask.
    #[props(default)]
    pub allow_target_interaction: bool,
    pub on_open_change: Option<EventHandler<bool>>,
    pub on_step_change: Option<EventHandler<usize>>,
    pub on_skip: Option<EventHandler<()>>,
    pub on_finish: Option<EventHandler<()>>,
    pub children: Element,
}

/// Multi-step anchored product tour.
#[component]
pub fn Guide(props: GuideProps) -> Element {
    let theme = use_theme();
    let i18n = use_component_i18n();
    let viewport = arkit_hooks::use_overlay_viewport();
    let registry = use_context_provider(GuideRegistry::new);
    let _target_revision = registry.revision();

    let mut internal_open = use_signal(|| props.default_open);
    let mut internal_step = use_signal(|| props.default_step);
    let open_controlled = props.open.is_some();
    let step_controlled = props.step.is_some();
    let current_open = props.open.unwrap_or_else(|| *internal_open.read());
    let current_step = normalize_step(
        props.step.unwrap_or_else(|| *internal_step.read()),
        props.steps.len(),
    );

    let labels = props.labels.clone().unwrap_or_else(|| GuideLabels {
        previous: i18n.guide_previous(),
        next: i18n.guide_next(),
        skip: i18n.guide_skip(),
        finish: i18n.guide_finish(),
    });

    let default_step = normalize_step(props.default_step, props.steps.len());
    let total_steps = props.steps.len();
    let on_action = EventHandler::new(move |action: GuideAction| match action {
        GuideAction::Previous => {
            let next = current_step.saturating_sub(1);
            if !step_controlled {
                internal_step.set(next);
            }
            if let Some(handler) = props.on_step_change {
                handler.call(next);
            }
        }
        GuideAction::Next if current_step + 1 < total_steps => {
            let next = current_step + 1;
            if !step_controlled {
                internal_step.set(next);
            }
            if let Some(handler) = props.on_step_change {
                handler.call(next);
            }
        }
        GuideAction::Next => {
            if !open_controlled {
                internal_open.set(false);
            }
            if !step_controlled {
                internal_step.set(default_step);
            }
            if let Some(handler) = props.on_open_change {
                handler.call(false);
            }
            if let Some(handler) = props.on_finish {
                handler.call(());
            }
        }
        GuideAction::Skip => {
            if !open_controlled {
                internal_open.set(false);
            }
            if !step_controlled {
                internal_step.set(default_step);
            }
            if let Some(handler) = props.on_open_change {
                handler.call(false);
            }
            if let Some(handler) = props.on_skip {
                handler.call(());
            }
        }
    });

    let active_step = props.steps.get(current_step).cloned();
    let target_frame = active_step
        .as_ref()
        .and_then(|step| registry.frame(&step.target));
    let snapshot = GuideOverlaySnapshot {
        open: current_open,
        step: active_step,
        current_step,
        total_steps,
        target_frame,
        viewport,
        labels,
        style: props.style,
        theme,
        allow_target_interaction: props.allow_target_interaction,
    };

    let portal = snapshot
        .open
        .then(|| {
            let step = snapshot.step.clone()?;
            let frame = snapshot.target_frame?;
            let geometry =
                GuideGeometry::resolve(frame, snapshot.viewport, snapshot.style, step.side)?;
            Some(guide_overlay_content(snapshot, step, geometry, on_action))
        })
        .flatten();

    rsx! {
        {props.children}
        if let Some(portal) = portal {
            arkit_hooks::Portal {
                layer: arkit_hooks::OverlayLayer::Floating,
                {portal}
            }
        }
    }
}

/// Registers one exact rendered element as a guide target.
///
/// `id` values must be unique within the nearest [`Guide`].
/// The render callback must attach its argument to exactly one native element's
/// `native_ref` attribute. This explicit render prop keeps `GuideTarget`
/// layout-transparent and preserves the target's parent flex contract.
#[component]
pub fn GuideTarget(
    id: String,
    render: dioxus_core::Callback<arkit_arkui::NativeElementRef, Element>,
) -> Element {
    let target_ref = arkit_hooks::use_native_element_ref();
    let registry = try_use_context::<GuideRegistry>();
    let frame_registry = registry.clone();
    let frame_id = id.clone();
    arkit_hooks::use_layout_frame(target_ref.clone(), move |frame| {
        if let Some(registry) = frame_registry.as_ref() {
            registry.update(&frame_id, frame);
        }
    });

    use_drop(move || {
        if let Some(registry) = registry.as_ref() {
            registry.remove(&id);
        }
    });

    render.call(target_ref)
}

#[derive(Clone)]
struct GuideRegistry {
    frames: Rc<RefCell<Vec<(String, arkit_hooks::LayoutFrame)>>>,
    revision: Signal<u64>,
}

impl GuideRegistry {
    fn new() -> Self {
        Self {
            frames: Rc::new(RefCell::new(Vec::new())),
            revision: Signal::new(0),
        }
    }

    fn revision(&self) -> u64 {
        (self.revision)()
    }

    fn frame(&self, id: &str) -> Option<arkit_hooks::LayoutFrame> {
        self.frames
            .borrow()
            .iter()
            .find(|(registered, _)| registered == id)
            .map(|(_, frame)| *frame)
    }

    fn update(&self, id: &str, frame: arkit_hooks::LayoutFrame) {
        let mut frames = self.frames.borrow_mut();
        if let Some((_, current)) = frames.iter_mut().find(|(registered, _)| registered == id) {
            if *current == frame {
                return;
            }
            *current = frame;
        } else {
            frames.push((id.to_owned(), frame));
        }
        drop(frames);
        self.bump_revision();
    }

    fn remove(&self, id: &str) {
        let removed = {
            let mut frames = self.frames.borrow_mut();
            let before = frames.len();
            frames.retain(|(registered, _)| registered != id);
            frames.len() != before
        };
        if removed {
            self.bump_revision();
        }
    }

    fn bump_revision(&self) {
        let mut revision = self.revision;
        let next = (*revision.peek()).wrapping_add(1);
        revision.set(next);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GuideRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl GuideRect {
    fn right(self) -> f32 {
        self.x + self.width
    }

    fn bottom(self) -> f32 {
        self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GuideGeometry {
    viewport_width: f32,
    viewport_height: f32,
    spotlight: GuideRect,
    panel: GuideRect,
}

impl GuideGeometry {
    fn resolve(
        target: arkit_hooks::LayoutFrame,
        viewport: arkit_hooks::OverlayViewport,
        style: GuideStyle,
        preferred_side: GuideSide,
    ) -> Option<Self> {
        if !target.is_measured() {
            return None;
        }

        let scale = viewport_scale(viewport).max(f32::EPSILON);
        let overlay = viewport.frame;
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
        let viewport_width = if overlay.is_measured() {
            overlay.width / scale
        } else {
            ohos_display_binding::default_display_width() as f32 / scale
        }
        .max(1.0);
        let viewport_height = if overlay.is_measured() {
            overlay.height / scale
        } else {
            ohos_display_binding::default_display_height() as f32 / scale
        }
        .max(1.0);

        let padding = finite_non_negative(style.spotlight_padding);
        let target_rect = GuideRect {
            x: (target.x - overlay_x) / scale,
            y: (target.y - overlay_y) / scale,
            width: (target.width / scale).max(0.0),
            height: (target.height / scale).max(0.0),
        };
        let spotlight_x = (target_rect.x - padding).clamp(0.0, viewport_width);
        let spotlight_y = (target_rect.y - padding).clamp(0.0, viewport_height);
        let spotlight_right = (target_rect.right() + padding).clamp(spotlight_x, viewport_width);
        let spotlight_bottom = (target_rect.bottom() + padding).clamp(spotlight_y, viewport_height);
        let spotlight = GuideRect {
            x: spotlight_x,
            y: spotlight_y,
            width: spotlight_right - spotlight_x,
            height: spotlight_bottom - spotlight_y,
        };

        let edge = spacing::SM;
        let min_x = viewport.safe_area.left.max(0.0) + edge;
        let min_y = viewport.safe_area.top.max(0.0) + edge;
        let available_width =
            (viewport_width - min_x - viewport.safe_area.right.max(0.0) - edge).max(1.0);
        let available_height =
            (viewport_height - min_y - viewport.safe_area.bottom.max(0.0) - edge).max(1.0);
        let panel_width =
            finite_positive(style.panel_width, GUIDE_DEFAULT_PANEL_WIDTH).min(available_width);
        let panel_height =
            finite_positive(style.estimated_panel_height, GUIDE_ESTIMATED_PANEL_HEIGHT)
                .min(available_height);
        let max_x = (min_x + available_width - panel_width).max(min_x);
        let max_y = (min_y + available_height - panel_height).max(min_y);
        let side_offset = finite_non_negative(style.side_offset);
        let side = resolve_side(
            preferred_side,
            spotlight,
            panel_width,
            panel_height,
            side_offset,
            GuideRect {
                x: min_x,
                y: min_y,
                width: available_width,
                height: available_height,
            },
        );

        let (raw_x, raw_y) = match side {
            GuideSide::Top => (
                spotlight.x + (spotlight.width - panel_width) / 2.0,
                spotlight.y - side_offset - panel_height,
            ),
            GuideSide::Bottom => (
                spotlight.x + (spotlight.width - panel_width) / 2.0,
                spotlight.bottom() + side_offset,
            ),
            GuideSide::Left => (
                spotlight.x - side_offset - panel_width,
                spotlight.y + (spotlight.height - panel_height) / 2.0,
            ),
            GuideSide::Right => (
                spotlight.right() + side_offset,
                spotlight.y + (spotlight.height - panel_height) / 2.0,
            ),
        };

        Some(Self {
            viewport_width,
            viewport_height,
            spotlight,
            panel: GuideRect {
                x: raw_x.clamp(min_x, max_x),
                y: raw_y.clamp(min_y, max_y),
                width: panel_width,
                height: panel_height,
            },
        })
    }
}

fn resolve_side(
    preferred: GuideSide,
    spotlight: GuideRect,
    panel_width: f32,
    panel_height: f32,
    offset: f32,
    bounds: GuideRect,
) -> GuideSide {
    let top = spotlight.y - bounds.y;
    let bottom = bounds.bottom() - spotlight.bottom();
    let left = spotlight.x - bounds.x;
    let right = bounds.right() - spotlight.right();
    let fits = |side| match side {
        GuideSide::Top => top >= panel_height + offset,
        GuideSide::Bottom => bottom >= panel_height + offset,
        GuideSide::Left => left >= panel_width + offset,
        GuideSide::Right => right >= panel_width + offset,
    };
    if fits(preferred) {
        return preferred;
    }
    let opposite = match preferred {
        GuideSide::Top => GuideSide::Bottom,
        GuideSide::Bottom => GuideSide::Top,
        GuideSide::Left => GuideSide::Right,
        GuideSide::Right => GuideSide::Left,
    };
    if fits(opposite) {
        return opposite;
    }
    match preferred {
        GuideSide::Top | GuideSide::Bottom => {
            if bottom >= top {
                GuideSide::Bottom
            } else {
                GuideSide::Top
            }
        }
        GuideSide::Left | GuideSide::Right => {
            if right >= left {
                GuideSide::Right
            } else {
                GuideSide::Left
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct GuideOverlaySnapshot {
    open: bool,
    step: Option<GuideStep>,
    current_step: usize,
    total_steps: usize,
    target_frame: Option<arkit_hooks::LayoutFrame>,
    viewport: arkit_hooks::OverlayViewport,
    labels: GuideLabels,
    style: GuideStyle,
    theme: Theme,
    allow_target_interaction: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuideAction {
    Previous,
    Next,
    Skip,
}

fn guide_overlay_content(
    snapshot: GuideOverlaySnapshot,
    step: GuideStep,
    geometry: GuideGeometry,
    on_action: EventHandler<GuideAction>,
) -> Element {
    let spotlight = geometry.spotlight;
    let panel = geometry.panel;
    let backdrop = snapshot.style.backdrop_color;
    let radius = snapshot.theme.radii.lg;
    let progress = format!("{} / {}", snapshot.current_step + 1, snapshot.total_steps);
    let next_label = if snapshot.current_step + 1 == snapshot.total_steps {
        snapshot.labels.finish.clone()
    } else {
        snapshot.labels.next.clone()
    };

    rsx! {
        stack {
            width: "100%",
            height: "100%",
            alignment: "top-start",
            hit_test_behavior: HIT_TEST_NONE,

            {guide_mask_rect(0.0, 0.0, geometry.viewport_width, spotlight.y, backdrop)}
            {guide_mask_rect(
                0.0,
                spotlight.bottom(),
                geometry.viewport_width,
                geometry.viewport_height - spotlight.bottom(),
                backdrop,
            )}
            {guide_mask_rect(
                0.0,
                spotlight.y,
                spotlight.x,
                spotlight.height,
                backdrop,
            )}
            {guide_mask_rect(
                spotlight.right(),
                spotlight.y,
                geometry.viewport_width - spotlight.right(),
                spotlight.height,
                backdrop,
            )}

            if !snapshot.allow_target_interaction {
                row {
                    position: format!("{},{}", spotlight.x, spotlight.y),
                    width: spotlight.width,
                    height: spotlight.height,
                    background_color: FLOATING_CAPTURE_COLOR,
                    hit_test_behavior: HIT_TEST_DEFAULT,
                    onclick: move |event| event.stop_propagation(),
                }
            }

            row {
                position: format!("{},{}", spotlight.x, spotlight.y),
                width: spotlight.width,
                height: spotlight.height,
                border_width: 2.0,
                border_color: snapshot.theme.colors.ring,
                border_radius: radius,
                hit_test_behavior: HIT_TEST_NONE,
            }

            column {
                position: format!("{},{}", panel.x, panel.y),
                width: panel.width,
                min_height: panel.height,
                align_items: "start",
                padding: spacing::LG,
                border_width: 1.0,
                border_color: snapshot.theme.colors.border,
                border_radius: radius,
                background_color: snapshot.theme.colors.popover,
                shadow: SHADOW_SM,
                hit_test_behavior: HIT_TEST_DEFAULT,
                onclick: move |event| event.stop_propagation(),

                row {
                    width: "100%",
                    align_items: "center",
                    justify_content: "space_between",
                    text {
                        content: progress,
                        font_size: typography::XS,
                        font_weight: 600_i32,
                        font_color: snapshot.theme.colors.muted_foreground,
                        line_height: 18.0,
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        shadow: false,
                        onclick: move |_| on_action.call(GuideAction::Skip),
                        "{snapshot.labels.skip}"
                    }
                }
                text {
                    width: "100%",
                    margin_top: spacing::SM,
                    content: step.title,
                    font_size: typography::XL,
                    font_weight: 600_i32,
                    font_color: snapshot.theme.colors.popover_foreground,
                    line_height: 24.0,
                    max_lines: 2_i32,
                }
                text {
                    width: "100%",
                    margin_top: spacing::XS,
                    content: step.description,
                    font_size: typography::SM,
                    font_weight: 400_i32,
                    font_color: snapshot.theme.colors.muted_foreground,
                    line_height: 20.0,
                    max_lines: 3_i32,
                    text_overflow: "ellipsis",
                }
                row {
                    width: "100%",
                    margin_top: spacing::LG,
                    align_items: "center",
                    justify_content: "end",
                    if snapshot.current_step > 0 {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            shadow: false,
                            onclick: move |_| on_action.call(GuideAction::Previous),
                            "{snapshot.labels.previous}"
                        }
                        row { width: spacing::SM }
                    }
                    Button {
                        size: ButtonSize::Sm,
                        shadow: false,
                        onclick: move |_| on_action.call(GuideAction::Next),
                        "{next_label}"
                    }
                }
            }
        }
    }
}

fn guide_mask_rect(x: f32, y: f32, width: f32, height: f32, color: u32) -> Element {
    if width <= 0.0 || height <= 0.0 {
        return rsx! {};
    }
    rsx! {
        row {
            position: format!("{x},{y}"),
            width,
            height,
            background_color: color,
            hit_test_behavior: HIT_TEST_DEFAULT,
            onclick: move |event| event.stop_propagation(),
        }
    }
}

fn normalize_step(step: usize, total: usize) -> usize {
    if total == 0 {
        0
    } else {
        step.min(total - 1)
    }
}

fn finite_positive(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_empty_and_out_of_range_steps() {
        assert_eq!(normalize_step(4, 0), 0);
        assert_eq!(normalize_step(4, 3), 2);
        assert_eq!(normalize_step(1, 3), 1);
    }

    #[test]
    fn flips_vertical_side_when_preferred_side_is_crowded() {
        let bounds = GuideRect {
            x: 8.0,
            y: 8.0,
            width: 384.0,
            height: 784.0,
        };
        let target = GuideRect {
            x: 120.0,
            y: 700.0,
            width: 80.0,
            height: 40.0,
        };
        assert_eq!(
            resolve_side(GuideSide::Bottom, target, 280.0, 160.0, 12.0, bounds),
            GuideSide::Top
        );
    }

    #[test]
    fn keeps_preferred_side_when_it_fits() {
        let bounds = GuideRect {
            x: 8.0,
            y: 8.0,
            width: 384.0,
            height: 784.0,
        };
        let target = GuideRect {
            x: 160.0,
            y: 160.0,
            width: 48.0,
            height: 48.0,
        };
        assert_eq!(
            resolve_side(GuideSide::Bottom, target, 280.0, 160.0, 12.0, bounds),
            GuideSide::Bottom
        );
    }

    #[test]
    fn converts_physical_target_frame_into_overlay_vp_geometry() {
        let viewport = arkit_hooks::OverlayViewport {
            frame: arkit_hooks::LayoutFrame {
                x: 10.0,
                y: 20.0,
                width: 800.0,
                height: 1_600.0,
            },
            safe_area: arkit_runtime::EdgeInsets::default(),
            scale: 2.0,
        };
        let target = arkit_hooks::LayoutFrame {
            x: 110.0,
            y: 220.0,
            width: 100.0,
            height: 80.0,
        };

        let geometry = GuideGeometry::resolve(
            target,
            viewport,
            GuideStyle {
                panel_width: 100.0,
                estimated_panel_height: 100.0,
                ..GuideStyle::default()
            },
            GuideSide::Bottom,
        )
        .expect("measured target should resolve");

        assert_eq!(
            geometry.spotlight,
            GuideRect {
                x: 42.0,
                y: 92.0,
                width: 66.0,
                height: 56.0,
            }
        );
        assert_eq!(geometry.viewport_width, 400.0);
        assert_eq!(geometry.viewport_height, 800.0);
        assert_eq!(geometry.panel.y, 160.0);
    }
}
