//! Window metrics and safe-area state shared by the runtime and component tree.
//!
//! OpenHarmony reports avoid areas in physical window pixels. Arkit converts
//! those rectangles into effective insets relative to the mounted XComponent
//! surface. Intersecting with the surface is important: a non-edge-to-edge
//! host has already moved the surface away from system bars and must therefore
//! observe zero additional inset.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

use openharmony_ability::{AvoidArea, AvoidAreaType, OpenHarmonyApp, Rect};

/// A rectangle in physical window pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicalRect {
    pub top: i32,
    pub left: i32,
    pub width: i32,
    pub height: i32,
}

impl PhysicalRect {
    pub fn right(self) -> i32 {
        self.left.saturating_add(self.width.max(0))
    }

    pub fn bottom(self) -> i32 {
        self.top.saturating_add(self.height.max(0))
    }

    pub fn is_empty(self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

impl From<Rect> for PhysicalRect {
    fn from(rect: Rect) -> Self {
        Self {
            top: rect.top,
            left: rect.left,
            width: rect.width,
            height: rect.height,
        }
    }
}

/// Insets in ArkUI virtual pixels (vp).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub fn max(self, other: Self) -> Self {
        Self {
            top: self.top.max(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
            left: self.left.max(other.left),
        }
    }
}

/// Current window geometry exposed to every Arkit component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowMetrics {
    /// Main window rectangle in physical pixels when supplied by the host.
    pub window_rect: PhysicalRect,
    /// Mounted XComponent surface rectangle in physical window pixels.
    pub content_rect: PhysicalRect,
    /// Physical-pixel to ArkUI-vp scale.
    pub scale: f32,
    /// Visual content avoidance: system bars, cutouts and navigation indicator.
    pub safe_area: EdgeInsets,
    /// System gesture exclusion area. Interactive controls may consume this
    /// without forcing all visual content away from the screen edge.
    pub gesture_area: EdgeInsets,
    /// Effective software-keyboard overlap reported by the window avoid area.
    pub ime_area: EdgeInsets,
    /// Raw keyboard height callback converted to vp. This remains separate
    /// from `ime_area` because resize-mode windows may already avoid the IME.
    pub keyboard_height: f32,
    avoid_areas: AvoidAreas,
}

impl Default for WindowMetrics {
    fn default() -> Self {
        Self {
            window_rect: PhysicalRect::default(),
            content_rect: PhysicalRect::default(),
            scale: 1.0,
            safe_area: EdgeInsets::ZERO,
            gesture_area: EdgeInsets::ZERO,
            ime_area: EdgeInsets::ZERO,
            keyboard_height: 0.0,
            avoid_areas: AvoidAreas::default(),
        }
    }
}

impl WindowMetrics {
    pub(crate) fn from_app(app: &OpenHarmonyApp, keyboard_height_px: Option<i32>) -> Self {
        let scale = normalized_scale(app.scale());
        let content_rect = PhysicalRect::from(app.content_rect());
        let window_rect = PhysicalRect::from(app.window_rect());
        let mut avoid_areas = AvoidAreas::default();
        for (area_type, area) in app.avoid_areas() {
            avoid_areas.set(area_type, area);
        }

        let mut metrics = Self {
            window_rect,
            content_rect,
            scale,
            safe_area: EdgeInsets::ZERO,
            gesture_area: EdgeInsets::ZERO,
            ime_area: EdgeInsets::ZERO,
            keyboard_height: keyboard_height_px.unwrap_or_default().max(0) as f32 / scale,
            avoid_areas,
        };
        metrics.recompute_insets();
        metrics
    }

    fn with_content_rect(mut self, content_rect: PhysicalRect) -> Self {
        self.content_rect = content_rect;
        self.recompute_insets();
        self
    }

    fn recompute_insets(&mut self) {
        self.safe_area =
            self.avoid_areas
                .visual()
                .into_iter()
                .fold(EdgeInsets::ZERO, |combined, area| {
                    combined.max(effective_avoid_insets(self.content_rect, area, self.scale))
                });
        self.gesture_area = effective_avoid_insets(
            self.content_rect,
            self.avoid_areas.system_gesture,
            self.scale,
        );
        self.ime_area =
            effective_avoid_insets(self.content_rect, self.avoid_areas.keyboard, self.scale);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct AvoidAreas {
    system: AvoidArea,
    cutout: AvoidArea,
    system_gesture: AvoidArea,
    keyboard: AvoidArea,
    navigation_indicator: AvoidArea,
}

impl AvoidAreas {
    fn set(&mut self, area_type: AvoidAreaType, area: AvoidArea) {
        match area_type {
            AvoidAreaType::System => self.system = area,
            AvoidAreaType::Cutout => self.cutout = area,
            AvoidAreaType::SystemGesture => self.system_gesture = area,
            AvoidAreaType::Keyboard => self.keyboard = area,
            AvoidAreaType::NavigationIndicator => self.navigation_indicator = area,
            AvoidAreaType::Unknown(_) => {}
        }
    }

    fn visual(self) -> [AvoidArea; 3] {
        [self.system, self.cutout, self.navigation_indicator]
    }
}

/// Root-content policy used by the Arkit application wrapper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SafeAreaPolicy {
    /// Keep business content inside the effective visual safe area.
    #[default]
    Safe,
    /// Let business content fill the XComponent surface. Window metrics and
    /// framework-owned overlay avoidance remain available.
    EdgeToEdge,
}

/// Shared root context updated by the OpenHarmony event loop.
#[derive(Clone)]
pub struct WindowMetricsHandle(Rc<WindowMetricsHandleInner>);

type WindowMetricsSubscriber = Rc<dyn Fn(WindowMetrics)>;

struct WindowMetricsHandleInner {
    metrics: Cell<WindowMetrics>,
    reported_content_rect: Cell<Option<PhysicalRect>>,
    subscribers: RefCell<BTreeMap<usize, WindowMetricsSubscriber>>,
    next_subscriber_id: Cell<usize>,
}

impl WindowMetricsHandle {
    pub(crate) fn new(metrics: WindowMetrics) -> Self {
        Self(Rc::new(WindowMetricsHandleInner {
            metrics: Cell::new(metrics),
            reported_content_rect: Cell::new(None),
            subscribers: RefCell::new(BTreeMap::new()),
            next_subscriber_id: Cell::new(0),
        }))
    }

    pub fn get(&self) -> WindowMetrics {
        self.0.metrics.get()
    }

    pub(crate) fn update(&self, mut metrics: WindowMetrics) -> bool {
        if metrics.content_rect.is_empty() {
            if let Some(reported) = self.0.reported_content_rect.get() {
                metrics = metrics.with_content_rect(reported);
            }
        }
        self.update_and_notify(metrics)
    }

    /// Report the measured full-screen Arkit host frame.
    ///
    /// ContentSlot hosts do not receive XComponent surface callbacks, so the
    /// framework root supplies this frame after its first native layout pass.
    pub fn report_content_rect(&self, content_rect: PhysicalRect) -> bool {
        if content_rect.is_empty() {
            return false;
        }
        self.0.reported_content_rect.set(Some(content_rect));
        self.update_and_notify(self.get().with_content_rect(content_rect))
    }

    /// Subscribe to window snapshots. The returned guard removes the callback
    /// when the last clone is dropped.
    pub fn subscribe(
        &self,
        callback: impl Fn(WindowMetrics) + 'static,
    ) -> WindowMetricsSubscription {
        let id = self.0.next_subscriber_id.get();
        self.0.next_subscriber_id.set(id.wrapping_add(1));
        self.0
            .subscribers
            .borrow_mut()
            .insert(id, Rc::new(callback));
        WindowMetricsSubscription {
            _inner: Rc::new(WindowMetricsSubscriptionInner {
                handle: Rc::downgrade(&self.0),
                id,
            }),
        }
    }

    fn update_and_notify(&self, metrics: WindowMetrics) -> bool {
        if self.get() == metrics {
            return false;
        }
        self.0.metrics.set(metrics);
        let subscribers = self
            .0
            .subscribers
            .borrow()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for subscriber in subscribers {
            subscriber(metrics);
        }
        #[cfg(debug_assertions)]
        super::log_window_metrics(metrics);
        true
    }
}

/// Lifetime guard for a [`WindowMetricsHandle`] subscription.
#[derive(Clone)]
pub struct WindowMetricsSubscription {
    _inner: Rc<WindowMetricsSubscriptionInner>,
}

struct WindowMetricsSubscriptionInner {
    handle: Weak<WindowMetricsHandleInner>,
    id: usize,
}

impl Drop for WindowMetricsSubscriptionInner {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.upgrade() {
            handle.subscribers.borrow_mut().remove(&self.id);
        }
    }
}

fn normalized_scale(scale: f32) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

fn effective_avoid_insets(content: PhysicalRect, area: AvoidArea, scale: f32) -> EdgeInsets {
    if !area.visible || content.is_empty() {
        return EdgeInsets::ZERO;
    }

    EdgeInsets {
        top: side_overlap(content, area.top_rect.into(), Side::Top) / scale,
        right: side_overlap(content, area.right_rect.into(), Side::Right) / scale,
        bottom: side_overlap(content, area.bottom_rect.into(), Side::Bottom) / scale,
        left: side_overlap(content, area.left_rect.into(), Side::Left) / scale,
    }
}

#[derive(Clone, Copy)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

fn side_overlap(content: PhysicalRect, avoid: PhysicalRect, side: Side) -> f32 {
    if avoid.is_empty() {
        return 0.0;
    }

    let horizontal = overlap(content.left, content.right(), avoid.left, avoid.right());
    let vertical = overlap(content.top, content.bottom(), avoid.top, avoid.bottom());
    if horizontal <= 0 || vertical <= 0 {
        return 0.0;
    }

    match side {
        Side::Top | Side::Bottom => vertical as f32,
        Side::Right | Side::Left => horizontal as f32,
    }
}

fn overlap(a_start: i32, a_end: i32, b_start: i32, b_end: i32) -> i32 {
    a_end.min(b_end).saturating_sub(a_start.max(b_start)).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, width: i32, height: i32) -> Rect {
        Rect {
            left,
            top,
            width,
            height,
        }
    }

    fn avoid(top: Rect, bottom: Rect, left: Rect, right: Rect) -> AvoidArea {
        AvoidArea {
            visible: true,
            top_rect: top,
            bottom_rect: bottom,
            left_rect: left,
            right_rect: right,
        }
    }

    #[test]
    fn non_fullscreen_surface_does_not_double_apply_system_bars() {
        let content = PhysicalRect {
            left: 0,
            top: 96,
            width: 1080,
            height: 2200,
        };
        let area = avoid(
            rect(0, 0, 1080, 96),
            rect(0, 2296, 1080, 104),
            Rect::default(),
            Rect::default(),
        );

        assert_eq!(effective_avoid_insets(content, area, 3.0), EdgeInsets::ZERO);
    }

    #[test]
    fn edge_to_edge_surface_reports_vp_insets() {
        let content = PhysicalRect {
            left: 0,
            top: 0,
            width: 1080,
            height: 2400,
        };
        let area = avoid(
            rect(0, 0, 1080, 96),
            rect(0, 2296, 1080, 104),
            Rect::default(),
            Rect::default(),
        );

        assert_eq!(
            effective_avoid_insets(content, area, 2.0),
            EdgeInsets {
                top: 48.0,
                right: 0.0,
                bottom: 52.0,
                left: 0.0,
            }
        );
    }

    #[test]
    fn landscape_cutout_is_intersected_on_the_correct_edge() {
        let content = PhysicalRect {
            left: 0,
            top: 0,
            width: 2400,
            height: 1080,
        };
        let area = avoid(
            Rect::default(),
            Rect::default(),
            rect(0, 0, 120, 1080),
            rect(2320, 0, 80, 1080),
        );

        assert_eq!(
            effective_avoid_insets(content, area, 2.0),
            EdgeInsets {
                top: 0.0,
                right: 40.0,
                bottom: 0.0,
                left: 60.0,
            }
        );
    }

    #[test]
    fn hidden_avoid_area_is_ignored() {
        let content = PhysicalRect {
            left: 0,
            top: 0,
            width: 100,
            height: 100,
        };
        let mut area = avoid(
            rect(0, 0, 100, 10),
            Rect::default(),
            Rect::default(),
            Rect::default(),
        );
        area.visible = false;

        assert_eq!(effective_avoid_insets(content, area, 1.0), EdgeInsets::ZERO);
    }

    #[test]
    fn adjacent_avoid_area_outside_content_has_no_overlap() {
        let content = PhysicalRect {
            left: 100,
            top: 100,
            width: 800,
            height: 600,
        };
        let area = avoid(
            rect(100, 0, 800, 100),
            rect(100, 700, 800, 100),
            rect(0, 100, 100, 600),
            rect(900, 100, 100, 600),
        );

        assert_eq!(effective_avoid_insets(content, area, 1.0), EdgeInsets::ZERO);
    }

    #[test]
    fn measured_content_rect_repairs_content_slot_metrics() {
        let mut metrics = WindowMetrics {
            scale: 2.0,
            ..WindowMetrics::default()
        };
        metrics.avoid_areas.system = avoid(
            rect(0, 0, 1000, 100),
            Rect::default(),
            Rect::default(),
            Rect::default(),
        );
        let handle = WindowMetricsHandle::new(metrics);

        assert!(handle.report_content_rect(PhysicalRect {
            left: 0,
            top: 0,
            width: 1000,
            height: 2000,
        }));
        assert_eq!(handle.get().safe_area.top, 50.0);
    }

    #[test]
    fn subscribers_receive_changed_metrics() {
        let handle = WindowMetricsHandle::new(WindowMetrics::default());
        let observed = Rc::new(Cell::new(PhysicalRect::default()));
        let observed_from_callback = observed.clone();
        let _subscription = handle.subscribe(move |metrics| {
            observed_from_callback.set(metrics.content_rect);
        });
        let expected = PhysicalRect {
            left: 12,
            top: 24,
            width: 800,
            height: 600,
        };

        assert!(handle.report_content_rect(expected));
        assert_eq!(observed.get(), expected);
    }
}
