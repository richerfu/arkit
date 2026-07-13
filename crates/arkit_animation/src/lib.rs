//! ArkUI animation API, ported to a dioxus 0.7 hook form.
//!
//! [`use_animation`] returns [`AnimationControls`] bound to the native ArkUI
//! node backing the current dioxus element (via
//! [`arkit_hooks::use_ark_node`]). [`AnimationControls::play`] drives the real
//! `ArkUINode::animate_to(&animation)` call so that attribute changes made
//! inside the `play` closure animate per the [`Motion`] spec.
//!
//! [`Motion`] is a builder for the ArkUI `Animation` option object (duration,
//! delay, iterations, tempo, curve, mode), ported from the legacy crate.
//! [`AnimationState`] covers the common opacity/translation/scale/rotation
//! properties plus optional color, radius, blur, and size properties, while
//! [`MountTransition`] provides ready-to-use entrance effects for mounted
//! Dioxus subtrees. [`Timeline`] and [`use_timeline`] provide frame-driven
//! keyframes, per-segment easing, repeat/alternate playback, seeking,
//! reversing, callbacks, and imperative Dioxus event controls.
//! [`TimelineGroup`] drives multiple [`use_animation_target`] nodes from one
//! clock with labels and relative track positions. [`stagger`] distributes
//! delays or values across independently rendered list items.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use arkit_prelude::*;
use ohos_arkui_binding::animate::options::Animation;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::r#type::animation_mode::AnimationMode;
use ohos_arkui_binding::r#type::curve::Curve;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;

mod group;
mod stagger;
mod timeline;

pub use group::{
    use_animation_target, use_timeline_group, AnimationTarget, TimelineGroup,
    TimelineGroupControls, TimelineGroupError, TimelineTrack,
};
pub use stagger::{stagger, Stagger, StaggerDirection, StaggerFrom};
pub use timeline::{
    use_timeline, Easing, PlaybackState, Timeline, TimelineControls, TimelineKeyframe,
};

thread_local! {
    static RETAINED_ANIMATIONS: RefCell<Vec<RetainedAnimation>> = const { RefCell::new(Vec::new()) };
}

struct RetainedAnimation {
    until: Instant,
    _animation: Rc<Animation>,
}

fn retain_animation(animation: Rc<Animation>, motion: Motion) {
    let now = Instant::now();
    let until = now + motion.retention_duration();
    RETAINED_ANIMATIONS.with(|state| {
        let mut retained = state.borrow_mut();
        retained.retain(|entry| entry.until > now);
        retained.push(RetainedAnimation {
            until,
            _animation: animation,
        });
    });
}

/// A builder for the ArkUI `Animation` option object.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motion {
    duration_ms: i32,
    delay_ms: i32,
    iterations: i32,
    tempo: f32,
    curve: Curve,
    mode: AnimationMode,
}

impl Motion {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn duration_ms(mut self, value: i32) -> Self {
        self.duration_ms = value.max(0);
        self
    }

    pub fn delay_ms(mut self, value: i32) -> Self {
        self.delay_ms = value.max(0);
        self
    }

    pub fn iterations(mut self, value: i32) -> Self {
        self.iterations = value;
        self
    }

    pub fn tempo(mut self, value: f32) -> Self {
        self.tempo = value.max(0.0);
        self
    }

    pub fn curve(mut self, value: Curve) -> Self {
        self.curve = value;
        self
    }

    pub fn mode(mut self, value: AnimationMode) -> Self {
        self.mode = value;
        self
    }

    /// Build the native `Animation` option object from this motion spec.
    pub fn build_animation(self) -> Animation {
        let animation = Animation::new();
        animation.duration(self.duration_ms);
        animation.delay(self.delay_ms);
        animation.iterations(self.iterations);
        animation.tempo(self.tempo);
        animation.curve(self.curve);
        animation.mode(self.mode);
        animation
    }

    fn retention_duration(self) -> Duration {
        if self.iterations < 0 {
            return Duration::from_millis(60_000);
        }

        let delay_ms = self.delay_ms.max(0) as u64;
        let duration_ms = self.duration_ms.max(0) as f64;
        let iterations = self.iterations.max(1) as f64;
        let tempo = self.tempo.max(0.01) as f64;
        let animated_ms = ((duration_ms * iterations) / tempo).ceil() as u64;
        Duration::from_millis((delay_ms + animated_ms + 1_000).clamp(1_000, 60_000))
    }
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            duration_ms: 200,
            delay_ms: 0,
            iterations: 1,
            tempo: 1.0,
            curve: Curve::EaseOut,
            mode: AnimationMode::Normal,
        }
    }
}

/// A reusable set of animatable visual properties.
///
/// The default value is the identity state: fully opaque, untranslated,
/// unscaled, and unrotated. Builder methods can be chained to describe either
/// the starting state or target state of an animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationState {
    pub opacity: f32,
    pub translate_x: f32,
    pub translate_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation_degrees: f32,
    /// Optional background color in ArkUI ARGB format (`0xAARRGGBB`).
    pub background_color: Option<u32>,
    /// Optional text color in ArkUI ARGB format (`0xAARRGGBB`).
    pub font_color: Option<u32>,
    /// Optional uniform corner radius in viewport pixels.
    pub border_radius: Option<f32>,
    /// Optional node blur radius.
    pub blur: Option<f32>,
    /// Optional explicit layout width in viewport pixels.
    pub width: Option<f32>,
    /// Optional explicit layout height in viewport pixels.
    pub height: Option<f32>,
}

impl AnimationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn opacity(mut self, value: f32) -> Self {
        self.opacity = value.clamp(0.0, 1.0);
        self
    }

    pub fn translate(mut self, x: f32, y: f32) -> Self {
        self.translate_x = x;
        self.translate_y = y;
        self
    }

    pub fn scale(mut self, x: f32, y: f32) -> Self {
        self.scale_x = x;
        self.scale_y = y;
        self
    }

    pub fn uniform_scale(self, value: f32) -> Self {
        self.scale(value, value)
    }

    /// Set a rotation around the Z axis, in degrees.
    pub fn rotate(mut self, degrees: f32) -> Self {
        self.rotation_degrees = degrees;
        self
    }

    pub fn background_color(mut self, argb: u32) -> Self {
        self.background_color = Some(argb);
        self
    }

    pub fn font_color(mut self, argb: u32) -> Self {
        self.font_color = Some(argb);
        self
    }

    pub fn border_radius(mut self, value: f32) -> Self {
        self.border_radius = Some(value.max(0.0));
        self
    }

    pub fn blur(mut self, value: f32) -> Self {
        self.blur = Some(value.max(0.0));
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width.max(0.0));
        self.height = Some(height.max(0.0));
        self
    }

    pub fn width(mut self, value: f32) -> Self {
        self.width = Some(value.max(0.0));
        self
    }

    pub fn height(mut self, value: f32) -> Self {
        self.height = Some(value.max(0.0));
        self
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_degrees: 0.0,
            background_color: None,
            font_color: None,
            border_radius: None,
            blur: None,
            width: None,
            height: None,
        }
    }
}

/// Relative changes applied to the previous keyframe by
/// [`Timeline::to_relative`]. Translation/rotation/layout values are additive;
/// scale values are multiplicative.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationDelta {
    opacity: f32,
    translate_x: f32,
    translate_y: f32,
    scale_x: f32,
    scale_y: f32,
    rotation_degrees: f32,
    border_radius: f32,
    blur: f32,
    width: f32,
    height: f32,
}

impl AnimationDelta {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn opacity_by(mut self, value: f32) -> Self {
        self.opacity = value;
        self
    }

    pub fn translate_by(mut self, x: f32, y: f32) -> Self {
        self.translate_x = x;
        self.translate_y = y;
        self
    }

    pub fn scale_by(mut self, x: f32, y: f32) -> Self {
        self.scale_x = x;
        self.scale_y = y;
        self
    }

    pub fn uniform_scale_by(self, value: f32) -> Self {
        self.scale_by(value, value)
    }

    pub fn rotate_by(mut self, degrees: f32) -> Self {
        self.rotation_degrees = degrees;
        self
    }

    pub fn border_radius_by(mut self, value: f32) -> Self {
        self.border_radius = value;
        self
    }

    pub fn blur_by(mut self, value: f32) -> Self {
        self.blur = value;
        self
    }

    pub fn resize_by(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    fn apply(self, state: AnimationState) -> AnimationState {
        AnimationState {
            opacity: (state.opacity + self.opacity).clamp(0.0, 1.0),
            translate_x: state.translate_x + self.translate_x,
            translate_y: state.translate_y + self.translate_y,
            scale_x: state.scale_x * self.scale_x,
            scale_y: state.scale_y * self.scale_y,
            rotation_degrees: state.rotation_degrees + self.rotation_degrees,
            background_color: state.background_color,
            font_color: state.font_color,
            border_radius: state
                .border_radius
                .map(|value| (value + self.border_radius).max(0.0)),
            blur: state.blur.map(|value| (value + self.blur).max(0.0)),
            width: state.width.map(|value| (value + self.width).max(0.0)),
            height: state.height.map(|value| (value + self.height).max(0.0)),
        }
    }
}

impl Default for AnimationDelta {
    fn default() -> Self {
        Self {
            opacity: 0.0,
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_degrees: 0.0,
            border_radius: 0.0,
            blur: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

pub(crate) fn apply_state(node: &ArkUINode, state: AnimationState) {
    let attributes = [
        (
            "opacity",
            node.set_attribute(ArkUINodeAttributeType::Opacity, state.opacity.into()),
        ),
        (
            "translate",
            node.set_attribute(
                ArkUINodeAttributeType::Translate,
                vec![state.translate_x, state.translate_y, 0.0_f32].into(),
            ),
        ),
        (
            "scale",
            node.set_attribute(
                ArkUINodeAttributeType::Scale,
                vec![state.scale_x, state.scale_y].into(),
            ),
        ),
        (
            "rotate",
            node.set_attribute(
                ArkUINodeAttributeType::Rotate,
                vec![0.0_f32, 0.0, 1.0, state.rotation_degrees, 0.0].into(),
            ),
        ),
    ];

    for (name, result) in attributes {
        if let Err(error) = result {
            ohos_hilog_binding::warn(format!("arkit_animation: setting {name} failed: {error:?}"));
        }
    }

    let optional_attributes = [
        state.background_color.map(|value| {
            (
                "background_color",
                node.set_attribute(ArkUINodeAttributeType::BackgroundColor, value.into()),
            )
        }),
        state.font_color.map(|value| {
            (
                "font_color",
                node.set_attribute(ArkUINodeAttributeType::FontColor, value.into()),
            )
        }),
        state.border_radius.map(|value| {
            (
                "border_radius",
                node.set_attribute(ArkUINodeAttributeType::BorderRadius, vec![value; 4].into()),
            )
        }),
        state.blur.map(|value| {
            (
                "blur",
                node.set_attribute(ArkUINodeAttributeType::Blur, value.into()),
            )
        }),
        state.width.map(|value| {
            (
                "width",
                node.set_attribute(ArkUINodeAttributeType::Width, value.into()),
            )
        }),
        state.height.map(|value| {
            (
                "height",
                node.set_attribute(ArkUINodeAttributeType::Height, value.into()),
            )
        }),
    ];

    for (name, result) in optional_attributes.into_iter().flatten() {
        if let Err(error) = result {
            ohos_hilog_binding::warn(format!("arkit_animation: setting {name} failed: {error:?}"));
        }
    }
}

/// Signal-backed controls for an animation bound to a native ArkUI node.
///
/// `is_running` flips while `play`'s `animate_to` block is in flight. `progress`
/// is a coarse `0.0..=1.0` signal set on start/finish (ArkUI's `animate_to` is
/// synchronous-block scoped: the attribute mutations inside the closure are
/// what animate; it does not stream per-frame progress to Rust).
#[derive(Clone, Copy)]
pub struct AnimationControls {
    motion: Motion,
    node_ref: arkit_hooks::ArkNodeRef,
    progress: Signal<f32>,
    is_running: Signal<bool>,
}

impl AnimationControls {
    /// Current coarse progress in `0.0..=1.0`.
    pub fn progress(&self) -> f32 {
        (self.progress)()
    }

    /// Whether an animation is currently in flight.
    pub fn is_running(&self) -> bool {
        (self.is_running)()
    }

    /// The underlying [`Motion`] spec.
    pub fn motion(&self) -> Motion {
        self.motion
    }

    /// Whether the current component scope has resolved to a native ArkUI node.
    ///
    /// This reads the node ref reactively, so effects that call this method
    /// rerun after the renderer resolves the scope on first mount.
    pub fn is_ready(&self) -> bool {
        self.node_ref.get().is_some()
    }

    /// Apply a visual state immediately, without animation.
    ///
    /// This is useful for establishing an animation's starting state before
    /// calling [`AnimationControls::animate_to`]. It is a no-op until the
    /// component's native node has mounted.
    pub fn set(&self, state: AnimationState) {
        let Some(node) = self.node_ref.peek() else {
            return;
        };
        apply_state(&node.borrow(), state);
    }

    /// Animate the common visual properties to `state` using this control's
    /// [`Motion`].
    pub fn animate_to(&self, state: AnimationState) {
        self.play(move |node| apply_state(node, state));
    }

    /// Animate to `state` after ArkUI has committed one frame.
    ///
    /// Use this after [`AnimationControls::set`] when establishing a new
    /// starting state. Deferring the target mutation prevents ArkUI from
    /// coalescing the start and end values into a single frame.
    pub fn animate_to_next_frame(&self, state: AnimationState) {
        let controls = *self;
        self.next_frame(move || controls.animate_to(state));
    }

    fn next_frame(&self, callback: impl Fn() + 'static) {
        let Some(node) = self.node_ref.peek() else {
            return;
        };
        let callback: Rc<dyn Fn()> = Rc::new(callback);
        let frame_callback = callback.clone();
        let result = node
            .borrow()
            .post_frame_callback(move |_, _| frame_callback());
        if let Err(error) = result {
            ohos_hilog_binding::warn(format!(
                "arkit_animation: post_frame_callback failed: {error:?}"
            ));
            callback();
        }
    }

    /// Start the animation. Attribute changes made inside `apply` are animated
    /// on the backing node per the [`Motion`] spec (ArkUI `animate_to`).
    ///
    /// If the backing node has not been resolved yet (e.g. the element is not
    /// yet mounted), this is a no-op.
    pub fn play(&self, apply: impl FnOnce(&ArkUINode) + 'static) {
        let Some(node) = self.node_ref.peek() else {
            return;
        };
        let mut is_running = self.is_running;
        let mut progress = self.progress;
        is_running.set(true);
        progress.set(0.0);

        let animation = Rc::new(self.motion.build_animation());
        let apply = Rc::new(RefCell::new(Some(
            Box::new(apply) as Box<dyn FnOnce(&ArkUINode)>
        )));
        let update_node = node.clone();
        let update_apply = apply.clone();
        animation.update(move || {
            let Some(apply) = update_apply.borrow_mut().take() else {
                return;
            };
            let node = update_node.borrow();
            apply(&node);
        });
        retain_animation(animation.clone(), self.motion);

        let result = node.borrow().animate_to(&animation);
        if let Err(error) = result {
            ohos_hilog_binding::warn(format!("arkit_animation: animate_to failed: {error:?}"));
            if let Some(apply) = apply.borrow_mut().take() {
                let node = node.borrow();
                apply(&node);
            }
        }

        // ArkUI's animate_to block completes synchronously on return; the
        // animation itself plays out on the UI thread. Mark the Rust-side
        // state machine as started; completion is best-effort (no per-frame
        // callback wired here).
        progress.set(1.0);
        is_running.set(false);
    }

    /// Stop tracking progress (resets the signal). ArkUI animations cannot be
    /// cancelled mid-flight via this API; this only resets the Rust state.
    pub fn stop(&self) {
        let mut is_running = self.is_running;
        let mut progress = self.progress;
        is_running.set(false);
        progress.set(0.0);
    }
}

/// Create an [`AnimationControls`] bound to the given [`Motion`] and the native
/// node backing the current dioxus element.
///
/// Requires an ancestor to have called `arkit_hooks::use_ark_host_provider()`
/// (so `use_ark_node` can resolve the node after render).
#[must_use]
pub fn use_animation(motion: Motion) -> AnimationControls {
    let node_ref = arkit_hooks::use_ark_node();
    let progress = use_signal(|| 0.0_f32);
    let is_running = use_signal(|| false);
    AnimationControls {
        motion,
        node_ref,
        progress,
        is_running,
    }
}

/// Built-in transition presets for mounted Dioxus subtrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionPreset {
    Fade,
    #[default]
    SlideUp,
    SlideDown,
    SlideLeft,
    SlideRight,
    ZoomIn,
    ZoomOut,
    RotateClockwise,
    RotateCounterClockwise,
}

impl TransitionPreset {
    /// Starting visual state for this entrance effect.
    pub fn initial_state(self) -> AnimationState {
        const SLIDE_DISTANCE: f32 = 24.0;
        const ROTATION_DEGREES: f32 = 14.0;

        match self {
            Self::Fade => AnimationState::new().opacity(0.0),
            Self::SlideUp => AnimationState::new()
                .opacity(0.0)
                .translate(0.0, SLIDE_DISTANCE),
            Self::SlideDown => AnimationState::new()
                .opacity(0.0)
                .translate(0.0, -SLIDE_DISTANCE),
            Self::SlideLeft => AnimationState::new()
                .opacity(0.0)
                .translate(SLIDE_DISTANCE, 0.0),
            Self::SlideRight => AnimationState::new()
                .opacity(0.0)
                .translate(-SLIDE_DISTANCE, 0.0),
            Self::ZoomIn => AnimationState::new().opacity(0.0).uniform_scale(0.82),
            Self::ZoomOut => AnimationState::new().opacity(0.0).uniform_scale(1.18),
            Self::RotateClockwise => AnimationState::new()
                .opacity(0.0)
                .uniform_scale(0.92)
                .rotate(-ROTATION_DEGREES),
            Self::RotateCounterClockwise => AnimationState::new()
                .opacity(0.0)
                .uniform_scale(0.92)
                .rotate(ROTATION_DEGREES),
        }
    }
}

/// Animate a subtree when its Dioxus component is mounted.
///
/// Routers can wrap an `Outlet` without changing router semantics. Change
/// `replay_id` when a mounted subtree should replay without being remounted.
#[component]
pub fn MountTransition(
    children: Element,
    #[props(default)] preset: Option<TransitionPreset>,
    #[props(default)] duration_ms: Option<i32>,
    #[props(default)] delay_ms: Option<i32>,
    /// Changing this value replays the transition without remounting the child.
    #[props(default)]
    replay_id: Option<u64>,
    #[props(default)] fill: Option<bool>,
) -> Element {
    let preset = preset.unwrap_or_default();
    let duration_ms = duration_ms.unwrap_or(180);
    let delay_ms = delay_ms.unwrap_or(0);
    let replay_id = replay_id.unwrap_or(0);
    let fill = fill.unwrap_or(false);
    let controls = use_animation(Motion::new().duration_ms(duration_ms).delay_ms(delay_ms));
    let entered = use_signal(|| false);
    let mut active_request = use_signal(|| None::<(TransitionPreset, i32, i32, u64)>);
    let has_entered = entered();
    let request = (preset, duration_ms, delay_ms, replay_id);

    use_effect(move || {
        if !controls.is_ready() || *active_request.peek() == Some(request) {
            return;
        }

        active_request.set(Some(request));
        let mut entered = entered;
        entered.set(false);
        controls.set(preset.initial_state());
        controls.next_frame(move || {
            if *active_request.peek() != Some(request) {
                return;
            }
            controls.animate_to(AnimationState::default());
            let mut entered = entered;
            entered.set(true);
        });
    });

    if fill {
        rsx! {
            column {
                percent_width: 1.0,
                percent_height: 1.0,
                align_items: "start",
                opacity: if has_entered { 1.0_f32 } else { 0.0_f32 },
                {children}
            }
        }
    } else {
        rsx! {
            column {
                align_items: "start",
                opacity: if has_entered { 1.0_f32 } else { 0.0_f32 },
                {children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_state_defaults_to_identity() {
        assert_eq!(AnimationState::new(), AnimationState::default());
        assert_eq!(AnimationState::new().opacity(-1.0).opacity, 0.0);
        assert_eq!(AnimationState::new().opacity(2.0).opacity, 1.0);
    }

    #[test]
    fn transition_presets_have_distinct_starting_transforms() {
        assert!(TransitionPreset::SlideUp.initial_state().translate_y > 0.0);
        assert!(TransitionPreset::SlideDown.initial_state().translate_y < 0.0);
        assert!(TransitionPreset::SlideLeft.initial_state().translate_x > 0.0);
        assert!(TransitionPreset::SlideRight.initial_state().translate_x < 0.0);
        assert!(TransitionPreset::ZoomIn.initial_state().scale_x < 1.0);
        assert!(TransitionPreset::ZoomOut.initial_state().scale_x > 1.0);
        assert!(
            TransitionPreset::RotateClockwise
                .initial_state()
                .rotation_degrees
                < 0.0
        );
        assert!(
            TransitionPreset::RotateCounterClockwise
                .initial_state()
                .rotation_degrees
                > 0.0
        );
    }
}
