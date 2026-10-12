use std::cell::Cell;
use std::rc::Rc;

use arkit_animation_core::{
    Angle, AnimatableValue, Easing, Length, SpringSpec, TimeSpan, TimelinePosition,
};
use arkit_prelude::*;

use crate::api::{Animation, Timeline};
use crate::hooks::{use_animation, use_animation_target};
use crate::presence::{
    use_animate_presence, PresenceKey, PresenceMode, PresencePhase, OVERLAY_PRESENCE_KEY,
};
use crate::properties::{OPACITY, ROTATION, SCALE_X, SCALE_Y, TRANSLATE_X, TRANSLATE_Y};
use crate::selector::AnimationSelector;

/// Default slide distance for [`MountTransition`] (page-scale motion).
const MOUNT_DISTANCE: f32 = 48.0;
/// Default slide distance for overlay presence (toasts, sheets, menus).
const OVERLAY_DISTANCE: f32 = 24.0;
const ROTATION_DEGREES: f32 = 32.0;
const ZOOM_IN_SCALE: f32 = 0.96;
const ZOOM_OUT_SCALE: f32 = 1.04;
const ROTATE_SCALE: f32 = 0.92;
const DEFAULT_ENTER_MS: i32 = 200;
const DEFAULT_EXIT_MS: i32 = 150;
const DEFAULT_MOUNT_MS: i32 = 180;

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

#[derive(Debug, Clone, Copy, PartialEq)]
struct TransitionValues {
    opacity: f32,
    translate_x: f32,
    translate_y: f32,
    scale: f32,
    rotation: f32,
}

impl TransitionPreset {
    fn hidden(self, distance: f32) -> TransitionValues {
        let distance = if distance.is_finite() { distance } else { 0.0 };
        match self {
            Self::Fade => TransitionValues::hidden(),
            Self::SlideUp => TransitionValues::hidden().translate(0.0, distance),
            Self::SlideDown => TransitionValues::hidden().translate(0.0, -distance),
            Self::SlideLeft => TransitionValues::hidden().translate(distance, 0.0),
            Self::SlideRight => TransitionValues::hidden().translate(-distance, 0.0),
            Self::ZoomIn => TransitionValues::hidden().scale(ZOOM_IN_SCALE),
            Self::ZoomOut => TransitionValues::hidden().scale(ZOOM_OUT_SCALE),
            Self::RotateClockwise => TransitionValues::hidden()
                .scale(ROTATE_SCALE)
                .rotate(-ROTATION_DEGREES),
            Self::RotateCounterClockwise => TransitionValues::hidden()
                .scale(ROTATE_SCALE)
                .rotate(ROTATION_DEGREES),
        }
    }
}

impl TransitionValues {
    const fn hidden() -> Self {
        Self {
            opacity: 0.0,
            translate_x: 0.0,
            translate_y: 0.0,
            scale: 1.0,
            rotation: 0.0,
        }
    }

    const fn shown() -> Self {
        Self {
            opacity: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            scale: 1.0,
            rotation: 0.0,
        }
    }

    const fn translate(mut self, x: f32, y: f32) -> Self {
        self.translate_x = x;
        self.translate_y = y;
        self
    }

    const fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    const fn rotate(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
}

/// System-style settling curve shared by overlay entrances and toast stacks.
pub fn enter_easing() -> Easing {
    Easing::Spring(SpringSpec::from_response(0.55, 0.825).expect("valid system enter spring"))
}

fn exit_easing() -> Easing {
    Easing::Spring(SpringSpec::from_response(0.347, 0.99).expect("valid system exit spring"))
}

// Opacity changes below ~4% are not perceptible; ending the timeline there
// removes the invisible settling tail that kept leaving pages on screen.
const PAGE_REST_SPEED: f32 = 1.0;
const PAGE_REST_DISPLACEMENT: f32 = 0.04;

/// Delay before an entering page starts fading in, so the leaving page is
/// mostly gone first and the two pages' content barely overlaps.
pub const PAGE_ENTER_DELAY_MS: i32 = 40;

/// Spring for an entering page fade; use its natural duration unless overridden.
pub fn page_enter_spring() -> SpringSpec {
    // Critical damping keeps opacity monotonic. With the 40 ms delay the page is
    // half visible at ~140 ms and 90% at ~265 ms, done by ~390 ms. A 0.6 s
    // response ran ~925 ms and left both pages overlapping on device.
    SpringSpec::from_response(0.36, 1.0)
        .and_then(|spring| spring.with_rest_thresholds(PAGE_REST_SPEED, PAGE_REST_DISPLACEMENT))
        .expect("valid page enter spring")
}

/// Spring for a leaving page fade: faster than the entrance so the old page
/// clears (~225 ms) before the new one is substantially visible.
pub fn page_exit_spring() -> SpringSpec {
    SpringSpec::from_response(0.2, 1.0)
        .and_then(|spring| spring.with_rest_thresholds(PAGE_REST_SPEED, PAGE_REST_DISPLACEMENT))
        .expect("valid page exit spring")
}

/// Natural duration of `spring` in whole milliseconds.
pub fn spring_duration_ms(spring: SpringSpec) -> i32 {
    (spring.natural_duration_seconds() * 1000.0).round() as i32
}

fn configure_tween(animation: Animation, easing: Easing, delay: TimeSpan) -> Animation {
    animation.configure_last(easing, Default::default(), Default::default(), delay, 0)
}

fn tween_property<T: AnimatableValue>(
    animation: Animation,
    property: &arkit_animation_core::Property<T>,
    from: Option<T>,
    to: T,
    duration: TimeSpan,
    delay: TimeSpan,
    easing: Easing,
) -> Animation {
    let animation = if let Some(from) = from {
        animation.tween(property, from, to, duration)
    } else {
        animation.tween_from_current(property, to, duration)
    };
    configure_tween(animation, easing, delay)
}

#[allow(clippy::too_many_arguments)]
fn transition_timeline(
    target: arkit_animation_core::TargetName,
    preset: TransitionPreset,
    duration: TimeSpan,
    delay: TimeSpan,
    phase: PresencePhase,
    distance: f32,
    spring: Option<SpringSpec>,
    resume: bool,
) -> Timeline {
    let hidden = preset.hidden(distance);
    let shown = TransitionValues::shown();
    // `resume` re-enters an interrupted exit from its current values instead of
    // snapping back to hidden first (e.g. A → B → A before A finished leaving).
    let (from, to, easing, from_current) = match phase {
        PresencePhase::Leaving => (shown, hidden, exit_easing(), true),
        PresencePhase::Entering | PresencePhase::Present => (hidden, shown, enter_easing(), resume),
    };
    let easing = spring.map(Easing::Spring).unwrap_or(easing);
    let explicit = |value: f32| if from_current { None } else { Some(value) };
    let mut animation = tween_property(
        Animation::new(AnimationSelector::Target(target)),
        &OPACITY,
        explicit(from.opacity),
        to.opacity,
        duration,
        delay,
        easing.clone(),
    );
    if from.translate_x != to.translate_x {
        animation = tween_property(
            animation,
            &TRANSLATE_X,
            explicit(from.translate_x).map(Length::vp),
            Length::vp(to.translate_x),
            duration,
            delay,
            easing.clone(),
        );
    }
    if from.translate_y != to.translate_y {
        animation = tween_property(
            animation,
            &TRANSLATE_Y,
            explicit(from.translate_y).map(Length::vp),
            Length::vp(to.translate_y),
            duration,
            delay,
            easing.clone(),
        );
    }
    if from.scale != to.scale {
        animation = tween_property(
            animation,
            &SCALE_X,
            explicit(from.scale),
            to.scale,
            duration,
            delay,
            easing.clone(),
        );
        animation = tween_property(
            animation,
            &SCALE_Y,
            explicit(from.scale),
            to.scale,
            duration,
            delay,
            easing.clone(),
        );
    }
    if from.rotation != to.rotation {
        animation = tween_property(
            animation,
            &ROTATION,
            explicit(from.rotation).map(Angle::degrees),
            Angle::degrees(to.rotation),
            duration,
            delay,
            easing,
        );
    }
    Timeline::new().add(animation, TimelinePosition::START)
}

fn wrap_transition(
    target_ref: arkit_arkui::NativeElementRef,
    fill: bool,
    opacity: f32,
    children: Element,
) -> Element {
    // Transparent: this wrapper is only an animation target. Hit testing must
    // reach children (and holes in those children) instead of being eaten here.
    if fill {
        rsx! {
            column {
                native_ref: target_ref,
                opacity,
                width: "100%",
                height: "100%",
                align_items: "start",
                hit_test_behavior: "transparent",
                {children}
            }
        }
    } else {
        // Stretch to the modal shell width so descendants can use `width: "100%"`
        // without collapsing; height stays content-sized for vertical centering.
        rsx! {
            column {
                native_ref: target_ref,
                opacity,
                width: "100%",
                align_items: "stretch",
                hit_test_behavior: "transparent",
                {children}
            }
        }
    }
}

// Keep the first native paint hidden until the engine owns opacity. Updating
// the declarative endpoint after its first frame is safe under that ownership
// and prevents terminal cleanup from restoring the initial hidden value.
fn use_transition_opacity(
    controls: &crate::AnimationControls,
    phase: PresencePhase,
    duration: TimeSpan,
) -> Signal<f32> {
    let opacity = use_signal(|| {
        if phase == PresencePhase::Entering && duration != TimeSpan::ZERO {
            TransitionValues::hidden().opacity
        } else {
            TransitionValues::shown().opacity
        }
    });
    controls.on_render(move |_| {
        let mut opacity = opacity;
        let endpoint = if phase == PresencePhase::Leaving {
            0.0
        } else {
            1.0
        };
        if *opacity.peek() != endpoint {
            opacity.set(endpoint);
        }
    });
    opacity
}

/// Play a one-shot enter timeline when the wrapper mounts (or `replay_id` changes).
///
/// Prefer [`PresenceTransition`] / [`VisibleTransition`] when the same node
/// also needs a hide animation. This component is enter-only.
/// `spring` overrides the shared system enter curve without changing duration.
#[component]
pub fn MountTransition(
    children: Element,
    #[props(default)] preset: Option<TransitionPreset>,
    #[props(default)] spring: Option<SpringSpec>,
    #[props(default)] duration_ms: Option<i32>,
    #[props(default)] delay_ms: Option<i32>,
    #[props(default)] distance: Option<f32>,
    #[props(default)] replay_id: Option<u64>,
    #[props(default)] fill: Option<bool>,
) -> Element {
    let name = use_hook(|| {
        arkit_animation_core::TargetName::owned(format!(
            "mount-transition-{:?}",
            current_scope_id()
        ))
    });
    let target = use_animation_target(name.as_str().to_owned());
    let target_ref = target.native_ref();
    let duration = TimeSpan::from_millis(duration_ms.unwrap_or(DEFAULT_MOUNT_MS).max(0) as u64);
    let delay = TimeSpan::from_millis(delay_ms.unwrap_or(0).max(0) as u64);
    let selected_preset = preset.unwrap_or_default();
    let selected_distance = distance.unwrap_or(MOUNT_DISTANCE);
    let controls = use_animation(transition_timeline(
        name.clone(),
        selected_preset,
        duration,
        delay,
        PresencePhase::Entering,
        selected_distance,
        spring,
        false,
    ));
    let opacity = use_transition_opacity(&controls, PresencePhase::Entering, duration);
    let request = (
        selected_preset,
        duration,
        delay,
        selected_distance.to_bits(),
        replay_id.unwrap_or(0),
        spring,
    );
    let mut active = use_signal(|| None);
    use_effect(use_reactive((&request,), move |(request,)| {
        if !target.is_ready() || !controls.is_ready() || *active.peek() == Some(request) {
            return;
        }
        controls.set_timeline(transition_timeline(
            name.clone(),
            request.0,
            request.1,
            request.2,
            PresencePhase::Entering,
            f32::from_bits(request.3),
            request.5,
            false,
        ));
        active.set(Some(request));
        controls.restart();
    }));
    wrap_transition(target_ref, fill.unwrap_or(false), opacity(), children)
}

/// Play enter or exit for a presence-tracked child and report the terminal phase.
///
/// Call [`crate::PresenceHandle::mark_present`] on `Entering` and
/// [`crate::PresenceHandle::settle_exit`] on `Leaving`. `Present` is a rest
/// state and does not start a timeline.
/// `spring` overrides both shared system curves without changing durations.
#[component]
pub fn PresenceTransition(
    children: Element,
    phase: PresencePhase,
    #[props(default)] preset: Option<TransitionPreset>,
    #[props(default)] spring: Option<SpringSpec>,
    #[props(default)] duration_ms: Option<i32>,
    #[props(default)] exit_duration_ms: Option<i32>,
    #[props(default)] delay_ms: Option<i32>,
    #[props(default)] distance: Option<f32>,
    #[props(default)] fill: Option<bool>,
    #[props(default)] on_terminal: Option<EventHandler<PresencePhase>>,
) -> Element {
    let name = use_hook(|| {
        arkit_animation_core::TargetName::owned(format!(
            "presence-transition-{:?}",
            current_scope_id()
        ))
    });
    let target = use_animation_target(name.as_str().to_owned());
    let target_ref = target.native_ref();
    let enter_duration =
        TimeSpan::from_millis(duration_ms.unwrap_or(DEFAULT_ENTER_MS).max(0) as u64);
    let exit_duration =
        TimeSpan::from_millis(exit_duration_ms.unwrap_or(DEFAULT_EXIT_MS).max(0) as u64);
    let delay = TimeSpan::from_millis(delay_ms.unwrap_or(0).max(0) as u64);
    let selected_preset = preset.unwrap_or_default();
    let selected_distance = distance.unwrap_or(OVERLAY_DISTANCE);
    let duration = if phase == PresencePhase::Leaving {
        exit_duration
    } else {
        enter_duration
    };
    let controls = use_animation(transition_timeline(
        name.clone(),
        selected_preset,
        duration,
        delay,
        phase,
        selected_distance,
        spring,
        false,
    ));
    let mut opacity = use_transition_opacity(&controls, phase, duration);
    let request = (
        phase,
        selected_preset,
        enter_duration,
        exit_duration,
        delay,
        selected_distance.to_bits(),
        spring,
    );
    let mut active = use_signal(|| None);
    let started_phase = use_hook(|| Rc::new(Cell::new(None::<PresencePhase>)));
    let started_for_complete = started_phase.clone();
    controls.on_complete(move || {
        if started_for_complete.get() != Some(phase) {
            return;
        }
        if let Some(handler) = on_terminal {
            handler.call(phase);
        }
    });
    use_effect(use_reactive((&request,), move |(request,)| {
        if !target.is_ready() || !controls.is_ready() || *active.peek() == Some(request) {
            return;
        }
        active.set(Some(request));
        let phase = request.0;
        if phase == PresencePhase::Present {
            started_phase.set(None);
            if *opacity.peek() != 1.0 {
                opacity.set(1.0);
            }
            return;
        }
        let duration = if phase == PresencePhase::Leaving {
            request.3
        } else {
            request.2
        };
        let resume = started_phase.get() == Some(PresencePhase::Leaving);
        if duration == TimeSpan::ZERO {
            // Settle synchronously; a zero-length timeline is not guaranteed to
            // report completion, and presence would otherwise never settle.
            started_phase.set(Some(phase));
            if let Some(handler) = on_terminal {
                handler.call(phase);
            }
            return;
        }
        controls.set_timeline(transition_timeline(
            name.clone(),
            request.1,
            duration,
            request.4,
            phase,
            f32::from_bits(request.5),
            request.6,
            resume,
        ));
        started_phase.set(Some(phase));
        controls.restart();
    }));
    wrap_transition(target_ref, fill.unwrap_or(false), opacity(), children)
}

/// Keep `children` mounted until the hide timeline finishes.
///
/// This is the boolean overlay helper: dialogs, sheets, tooltips, menus.
/// For keyed collections (Sonner, lists) use [`use_animate_presence`] plus
/// [`PresenceTransition`] instead.
#[component]
pub fn VisibleTransition(
    visible: bool,
    children: Element,
    #[props(default)] preset: Option<TransitionPreset>,
    #[props(default)] duration_ms: Option<i32>,
    #[props(default)] exit_duration_ms: Option<i32>,
    #[props(default)] delay_ms: Option<i32>,
    #[props(default)] distance: Option<f32>,
    #[props(default)] fill: Option<bool>,
) -> Element {
    let visibility = use_presence_visibility(visible);
    if !visibility.mounted {
        return rsx! {};
    }
    rsx! {
        PresenceTransition {
            phase: visibility.phase,
            on_terminal: visibility.on_terminal,
            preset,
            duration_ms,
            exit_duration_ms,
            delay_ms,
            distance,
            fill,
            {children}
        }
    }
}

/// Snapshot of a boolean presence gate.
#[derive(Clone)]
pub struct PresenceVisibility {
    pub mounted: bool,
    pub phase: PresencePhase,
    pub on_terminal: EventHandler<PresencePhase>,
}

/// Track a single `visible` flag and keep the node until exit settles.
///
/// Use this when the hide animation must live *inside* a portal that would
/// otherwise unmount with `open: false`.
#[track_caller]
pub fn use_presence_visibility(visible: bool) -> PresenceVisibility {
    let presence = use_animate_presence(
        PresenceMode::Sync,
        if visible {
            vec![(PresenceKey::new(OVERLAY_PRESENCE_KEY), ())]
        } else {
            Vec::new()
        },
    );
    let entries = presence.entries();
    let mounted = !entries.is_empty();
    let phase = entries
        .first()
        .map(|entry| entry.phase)
        .unwrap_or(PresencePhase::Present);
    let on_terminal = EventHandler::new(move |phase: PresencePhase| {
        let key = PresenceKey::new(OVERLAY_PRESENCE_KEY);
        match phase {
            PresencePhase::Entering => {
                presence.mark_present(&key);
            }
            PresencePhase::Leaving => {
                presence.settle_exit(&key);
            }
            PresencePhase::Present => {}
        }
    });
    PresenceVisibility {
        mounted,
        phase,
        on_terminal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_timelines_only_animate_opacity_with_the_given_spring() {
        for (phase, page_spring) in [
            (PresencePhase::Entering, page_enter_spring()),
            (PresencePhase::Leaving, page_exit_spring()),
        ] {
            let timeline = transition_timeline(
                crate::TargetName::static_name("page"),
                TransitionPreset::Fade,
                TimeSpan::from_millis(300),
                TimeSpan::ZERO,
                phase,
                MOUNT_DISTANCE,
                Some(page_spring),
                false,
            );
            let [arkit_animation_core::TimelineNode::Animation { animation, .. }] =
                timeline.source().nodes.as_slice()
            else {
                panic!("expected one page animation");
            };
            assert_eq!(animation.tweens.len(), 1);
            assert_eq!(animation.tweens[0].property.as_str(), "opacity");
            assert!(
                matches!(animation.tweens[0].easing, Easing::Spring(spring) if spring == page_spring)
            );
        }
    }

    #[test]
    fn page_fades_are_short_and_the_exit_clears_first() {
        let enter = spring_duration_ms(page_enter_spring()) + PAGE_ENTER_DELAY_MS;
        let exit = spring_duration_ms(page_exit_spring());
        assert!(enter <= 420, "enter {enter} ms");
        assert!(
            exit < enter,
            "exit {exit} ms must finish before enter {enter} ms"
        );
    }

    #[test]
    fn initial_native_opacity_is_hidden_only_for_animated_entrances() {
        fn app() -> Element {
            crate::engine::use_animation_host_provider();
            rsx! {
                MountTransition { preset: TransitionPreset::Fade, "mount" }
                PresenceTransition { phase: PresencePhase::Entering, "enter" }
                PresenceTransition { phase: PresencePhase::Present, fill: true, "present" }
                PresenceTransition { phase: PresencePhase::Entering, duration_ms: 0, "instant" }
            }
        }
        let mut dom = dioxus_core::VirtualDom::new(app);
        let mutations = dom.rebuild_to_vec();
        let opacities = mutations
            .edits
            .iter()
            .filter_map(|edit| match edit {
                dioxus_core::Mutation::SetAttribute {
                    name: "opacity",
                    value: dioxus_core::AttributeValue::Float(value),
                    ..
                } => Some(*value),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(opacities, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn fade_hides_with_opacity_only() {
        let hidden = TransitionPreset::Fade.hidden(80.0);
        assert_eq!(hidden.opacity, 0.0);
        assert_eq!(hidden.translate_x, 0.0);
        assert_eq!(hidden.translate_y, 0.0);
        assert_eq!(hidden.scale, 1.0);
    }

    #[test]
    fn slide_presets_move_from_the_named_edge() {
        assert_eq!(TransitionPreset::SlideUp.hidden(24.0).translate_y, 24.0);
        assert_eq!(TransitionPreset::SlideDown.hidden(24.0).translate_y, -24.0);
        assert_eq!(TransitionPreset::SlideLeft.hidden(24.0).translate_x, 24.0);
        assert_eq!(TransitionPreset::SlideRight.hidden(24.0).translate_x, -24.0);
    }

    #[test]
    fn zoom_in_is_a_subtle_scale_not_a_pop() {
        let hidden = TransitionPreset::ZoomIn.hidden(0.0);
        assert!(hidden.scale > 0.9 && hidden.scale < 1.0);
        assert_eq!(hidden.opacity, 0.0);
    }

    #[test]
    fn shown_values_are_identity() {
        let shown = TransitionValues::shown();
        assert_eq!(shown.opacity, 1.0);
        assert_eq!(shown.translate_x, 0.0);
        assert_eq!(shown.translate_y, 0.0);
        assert_eq!(shown.scale, 1.0);
        assert_eq!(shown.rotation, 0.0);
    }
}
