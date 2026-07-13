use arkit::prelude::*;

use crate::{color, cubic_out, target, ActionButton, Metric, Section};

const SCOPE_A: &str = "lab-scope-a";
const PROPERTY_TARGET: &str = "lab-property-card";
const TYPOGRAPHY_TARGET: &str = "lab-property-type";

#[component]
pub(crate) fn OrchestrationLab() -> Element {
    rsx! {
        ScopeDemo {}
        PropertyDemo {}
        CapabilityDemo {}
    }
}

#[component]
fn ScopeDemo() -> Element {
    let scope = use_animation_scope(AnimationScopeDefaults {
        playback: PlaybackSettings {
            playback_rate: PlaybackRate::new(0.8).expect("constant playback rate"),
            ..PlaybackSettings::default()
        },
        cleanup: ScopeCleanupPolicy::Revert,
        keep_time: false,
    });
    use_context_provider(|| scope);

    rsx! {
        Section {
            title: "AnimationScope orchestration",
            description: "A scope owns its control, shared PlaybackSettings, named methods, refresh and deterministic revert-on-drop cleanup.",
            ScopedTarget {}
        }
    }
}

#[component]
fn ScopedTarget() -> Element {
    let scope = use_context::<AnimationScope>();
    let target = use_animation_target(SCOPE_A);
    let controls = use_scoped_animation(&scope, scope_timeline());
    let snapshot = use_animation_snapshot(&controls);
    let scope_event = use_signal(|| "idle".to_string());
    let play_controls = controls.clone();
    scope.method(ScopeMethodName::owned("play"), move || {
        play_controls.restart();
    });
    let reverse_controls = controls.clone();
    scope.method(ScopeMethodName::owned("reverse"), move || {
        reverse_controls.reverse();
        reverse_controls.resume();
    });
    let state = snapshot()
        .map(|value| format!("{:?}", value.state))
        .unwrap_or_else(|| {
            if target.is_ready() && controls.is_ready() {
                "Ready".to_string()
            } else {
                "Resolving".to_string()
            }
        });
    rsx! {
        column {
            percent_width: 1.0,
            row {
                percent_width: 1.0,
                height: 170.0,
                align_items: "center",
                justify_content: "center",
                background_color: 0xffe2e8f0u32,
                border_radius: 14.0,
                column {
                    width: 110.0,
                    height: 88.0,
                    align_items: "center",
                    justify_content: "center",
                    background_color: 0xff4f46e5u32,
                    border_radius: 22.0,
                    text { font_size: 22.0, font_weight: 700, font_color: 0xffffffffu32, "Scope" }
                    text { margin_top: 3.0, font_size: 9.0, font_color: 0xffe0e7ffu32, "{state}" }
                }
            }
            flex {
                margin_top: 12.0,
                percent_width: 1.0,
                flex_wrap: "wrap",
                ActionButton {
                    label: "play()",
                    on_press: {
                        let scope = scope.clone();
                        move |_| {
                            let called = scope.call(&ScopeMethodName::owned("play"));
                            let mut scope_event = scope_event;
                            scope_event.set(if called { "play invoked" } else { "play missing" }.to_string());
                        }
                    }
                }
                ActionButton {
                    label: "reverse()",
                    on_press: {
                        let scope = scope.clone();
                        move |_| {
                            let called = scope.call(&ScopeMethodName::owned("reverse"));
                            let mut scope_event = scope_event;
                            scope_event.set(if called { "reverse invoked" } else { "reverse missing" }.to_string());
                        }
                    }
                }
                ActionButton { label: "Scope refresh", on_press: { let scope = scope.clone(); move |_| scope.refresh() } }
                ActionButton { label: "Scope revert", on_press: { let scope = scope.clone(); move |_| scope.revert() } }
                Metric { label: "Method", value: scope_event() }
            }
        }
    }
}

fn scope_timeline() -> Timeline {
    let duration = TimeSpan::from_millis(900);
    let first = Animation::new(target(SCOPE_A))
        .tween(&TRANSLATE_X, Length::vp(-90.0), Length::vp(80.0), duration)
        .configure_last(
            cubic_out(),
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        );
    Timeline::new().add(first, TimelinePosition::START)
}

#[component]
fn PropertyDemo() -> Element {
    let controls = use_animation(property_timeline());
    rsx! {
        Section {
            title: "Typed property schema",
            description: "Transform, layout, paint, filter, border and typography properties are resolved through typed Property<T> descriptors and one dirty-write batch.",
            column {
                percent_width: 1.0,
                height: 190.0,
                align_items: "center",
                justify_content: "center",
                background_color: 0xffe2e8f0u32,
                border_radius: 14.0,
                PropertyCard {}
                TypographyTarget {}
            }
            flex {
                margin_top: 12.0,
                percent_width: 1.0,
                flex_wrap: "wrap",
                ActionButton { label: "Animate schema", on_press: { let controls = controls.clone(); move |_| controls.restart() } }
                ActionButton { label: "Reverse", on_press: { let controls = controls.clone(); move |_| controls.reverse() } }
                ActionButton { label: "Reset", on_press: { let controls = controls.clone(); move |_| controls.reset() } }
                ActionButton { label: "Revert baseline", on_press: { let controls = controls.clone(); move |_| controls.revert() } }
            }
        }
    }
}

#[component]
fn PropertyCard() -> Element {
    let _target = use_animation_target(PROPERTY_TARGET);
    rsx! {
        column {
            width: 132.0,
            height: 82.0,
            align_items: "center",
            justify_content: "center",
            background_color: 0xff0f766eu32,
            border_width: 2.0,
            border_color: 0xffffffffu32,
            border_radius: 12.0,
            opacity: 1.0,
            text { font_size: 12.0, font_weight: 700, font_color: 0xffffffffu32, "Paint + layout" }
        }
    }
}

#[component]
fn TypographyTarget() -> Element {
    let _target = use_animation_target(TYPOGRAPHY_TARGET);
    rsx! {
        text {
            margin_top: 18.0,
            font_size: 16.0,
            font_color: 0xff312e81u32,
            "Typed typography"
        }
    }
}

fn property_timeline() -> Timeline {
    let duration = TimeSpan::from_millis(1_100);
    let card = Animation::new(target(PROPERTY_TARGET))
        .tween(&WIDTH, Length::vp(132.0), Length::vp(220.0), duration)
        .configure_last(
            cubic_out(),
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        )
        .tween(&HEIGHT, Length::vp(82.0), Length::vp(116.0), duration)
        .configure_last(
            cubic_out(),
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        )
        .tween(&BORDER_RADIUS, Length::vp(12.0), Length::vp(38.0), duration)
        .tween(&BORDER_WIDTH, Length::vp(2.0), Length::vp(7.0), duration)
        .tween(
            &BORDER_COLOR,
            color(0xffffffffu32),
            color(0xffffd166u32),
            duration,
        )
        .tween(
            &BACKGROUND_COLOR,
            color(0xff0f766eu32),
            color(0xff7c3aedu32),
            duration,
        )
        .tween(&BLUR, Length::vp(0.0), Length::vp(3.0), duration)
        .tween(&BRIGHTNESS, 0.75, 1.25, duration)
        .tween(&SATURATION, 0.35, 1.45, duration)
        .tween(&CONTRAST, 0.8, 1.2, duration);
    let typography = Animation::new(target(TYPOGRAPHY_TARGET))
        .tween(&FONT_SIZE, Length::vp(16.0), Length::vp(28.0), duration)
        .configure_last(
            Easing::Spring(SpringSpec::default()),
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        )
        .tween(&LETTER_SPACING, Length::vp(0.0), Length::vp(3.0), duration)
        .tween(
            &FONT_COLOR,
            color(0xff312e81u32),
            color(0xffe11d48u32),
            duration,
        );
    let additive_opacity = Animation::new(target(PROPERTY_TARGET))
        .tween(&OPACITY, 0.0, 0.25, duration)
        .configure_last(
            Easing::Builtin(BuiltinEase::Sine(EaseDirection::InOut)),
            Composition::Add,
            Modifier::Identity,
            TimeSpan::ZERO,
            10,
        );
    Timeline::new()
        .add(card, TimelinePosition::START)
        .add(typography, TimelinePosition::START)
        .add(additive_opacity, TimelinePosition::START)
}

#[component]
fn CapabilityDemo() -> Element {
    let requirements = CapabilityRequirements {
        seek: true,
        pause: true,
        reverse: true,
        callbacks: true,
        composition: true,
        custom_easing: true,
        ..CapabilityRequirements::default()
    };
    let report = arkit::animation::NativeLowerer
        .lower(ExecutionPolicy::Auto, requirements)
        .expect("sampled backend supports the full requirement set");
    let metrics = AnimationWindowMetrics {
        width_vp: 360.0,
        height_vp: 720.0,
        density: 3.0,
    };
    rsx! {
        Section {
            title: "Capability lowering and conditions",
            description: "ExecutionPolicy is checked against a typed capability contract. Unsupported native semantics produce explicit rejection details before sampled fallback.",
            flex {
                percent_width: 1.0,
                flex_wrap: "wrap",
                Metric { label: "Requested", value: format!("{:?}", report.requested) }
                Metric { label: "Selected", value: format!("{:?}", report.selected) }
                Metric { label: "Native rejects", value: report.rejections.len().to_string() }
                Metric { label: "Portrait", value: WindowCondition::Portrait.matches(metrics).to_string() }
                Metric { label: "Min 320vp", value: WindowCondition::MinWidth(320.0).matches(metrics).to_string() }
            }
            text {
                margin_top: 6.0,
                font_size: 10.0,
                font_color: 0xff64748bu32,
                "Required: seek, pause, reverse, callbacks, composition and custom easing"
            }
        }
    }
}
