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
}

/// Animate a subtree when its Dioxus component is mounted.
///
/// This component deliberately follows normal Dioxus composition: callers key
/// it to remount and replay the transition, and routers can wrap an `Outlet`
/// without changing router semantics.
#[component]
pub fn MountTransition(
    children: Element,
    #[props(default)] preset: Option<TransitionPreset>,
    #[props(default)] duration_ms: Option<i32>,
    #[props(default)] delay_ms: Option<i32>,
    #[props(default)] fill: Option<bool>,
) -> Element {
    let _preset = preset.unwrap_or_default();
    let duration_ms = duration_ms.unwrap_or(180);
    let delay_ms = delay_ms.unwrap_or(0);
    let fill = fill.unwrap_or(false);
    let controls = use_animation(Motion::new().duration_ms(duration_ms).delay_ms(delay_ms));
    let mut entered = use_signal(|| false);
    let has_entered = entered();

    use_effect(move || {
        if has_entered || !controls.is_ready() {
            return;
        }

        controls.play(|node| {
            if let Err(error) = node.set_attribute(ArkUINodeAttributeType::Opacity, 1.0_f32.into())
            {
                ohos_hilog_binding::warn(format!(
                    "arkit_animation: setting transition opacity failed: {error:?}"
                ));
            }
        });
        entered.set(true);
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
