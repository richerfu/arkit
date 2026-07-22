use arkit::prelude::*;

use crate::{
    color, cubic_out, play_forward, restart_forward, reverse_and_play, target, ActionButton,
    Metric, Section,
};

const CARD_TARGET: &str = "lab-timeline-card";
const BADGE_TARGET: &str = "lab-timeline-badge";
const TIMELINE_DURATION_MS: u64 = 2_360;

#[component]
pub(crate) fn TimelineLab() -> Element {
    let event = use_signal(|| "idle".to_string());
    let begin_count = use_signal(|| 0_u32);
    let loop_count = use_signal(|| 0_u32);
    let complete_count = use_signal(|| 0_u32);
    let cancel_count = use_signal(|| 0_u32);
    let call_count = use_signal(|| 0_u32);
    let command = use_signal(|| "none".to_string());
    let alternate = use_signal(|| true);
    let duration_ms = use_signal(|| TIMELINE_DURATION_MS);
    let controls = use_animation(demo_timeline(call_count));

    controls.on_begin(move || {
        let mut begin_count = begin_count;
        let mut event = event;
        begin_count += 1;
        event.set("begin".to_string());
    });
    controls.on_pause(move || {
        let mut event = event;
        event.set("pause".to_string());
    });
    controls.on_loop(move |iteration| {
        let mut loop_count = loop_count;
        let mut event = event;
        loop_count.set(iteration);
        event.set(format!("loop #{iteration}"));
    });
    controls.on_complete(move || {
        let mut complete_count = complete_count;
        let mut event = event;
        complete_count += 1;
        event.set("complete".to_string());
    });
    controls.on_cancel(move || {
        let mut cancel_count = cancel_count;
        let mut event = event;
        cancel_count += 1;
        event.set("cancel".to_string());
    });
    rsx! {
        Section {
            title: "Multi-target timeline",
            description: "Labels, relative positions, nested timelines, set/call nodes, keyframes, alternate iterations and one root clock.",
            column {
                width: "100%",
                height: 210.0,
                align_items: "center",
                justify_content: "center",
                background_color: "#FFEEF2FF",
                border_radius: 14.0,
                row {
                    align_items: "center",
                    TimelineCard {}
                    TimelineBadge {}
                }
            }
            flex {
                margin_top: 12.0,
                width: "100%",
                flex_wrap: "wrap",
                ActionButton {
                    label: "Play forward",
                    on_press: {
                        let controls = controls.clone();
                        move |_| play_forward(&controls)
                    }
                }
                ActionButton { label: "Pause", on_press: { let controls = controls.clone(); move |_| controls.pause() } }
                ActionButton { label: "Resume", on_press: { let controls = controls.clone(); move |_| controls.resume() } }
                ActionButton {
                    label: "Restart forward",
                    on_press: {
                        let controls = controls.clone();
                        move |_| restart_forward(&controls)
                    }
                }
                ActionButton {
                    label: "Reverse + play",
                    on_press: {
                        let controls = controls.clone();
                        move |_| reverse_and_play(&controls, point_ms(duration_ms()))
                    }
                }
                ActionButton { label: "Complete", on_press: { let controls = controls.clone(); move |_| controls.complete() } }
                ActionButton { label: "Cancel", on_press: { let controls = controls.clone(); move |_| controls.cancel() } }
                ActionButton { label: "Reset", on_press: { let controls = controls.clone(); move |_| controls.reset() } }
                ActionButton { label: "Revert", on_press: { let controls = controls.clone(); move |_| controls.revert() } }
            }
        }

        Section {
            title: "Seek and runtime controls",
            description: "The same instance is scrubbed, stretched, rate-adjusted and switched between alternate/non-alternate playback without a component timer.",
            flex {
                width: "100%",
                flex_wrap: "wrap",
                ActionButton { label: "Seek 0%", on_press: { let controls = controls.clone(); move |_| controls.seek(TimePoint::ZERO) } }
                ActionButton {
                    label: "Seek 50%",
                    on_press: {
                        let controls = controls.clone();
                        move |_| controls.seek(point_ms(duration_ms() / 2))
                    }
                }
                ActionButton { label: "Seek + events", on_press: { let controls = controls.clone(); move |_| controls.seek_with_events(point_ms(1_260)) } }
                ActionButton {
                    label: "Stretch 3s",
                    on_press: {
                        let controls = controls.clone();
                        move |_| {
                            controls.stretch(TimeSpan::from_millis(3_000));
                            let mut duration_ms = duration_ms;
                            duration_ms.set(3_000);
                            let mut command = command;
                            command.set("duration = 3s".to_string());
                        }
                    }
                }
                ActionButton {
                    label: "Rate 0.5x",
                    on_press: {
                        let controls = controls.clone();
                        move |_| {
                            controls.set_playback_rate(PlaybackRate::new(0.5).expect("constant rate"));
                            let mut command = command;
                            command.set("playback rate = 0.5x".to_string());
                        }
                    }
                }
                ActionButton {
                    label: "Rate 1.5x",
                    on_press: {
                        let controls = controls.clone();
                        move |_| {
                            controls.set_playback_rate(PlaybackRate::new(1.5).expect("constant rate"));
                            let mut command = command;
                            command.set("playback rate = 1.5x".to_string());
                        }
                    }
                }
                ActionButton {
                    label: "Alternate",
                    active: alternate(),
                    on_press: {
                        let controls = controls.clone();
                        move |_| {
                            let enabled = !alternate();
                            controls.set_alternate(enabled);
                            let mut alternate = alternate;
                            alternate.set(enabled);
                            let mut command = command;
                            command.set(format!("alternate = {enabled}"));
                        }
                    }
                }
                ActionButton {
                    label: "Refresh",
                    on_press: {
                        let controls = controls.clone();
                        move |_| {
                            controls.refresh();
                            let mut command = command;
                            command.set("targets refreshed".to_string());
                        }
                    }
                }
            }
            flex {
                margin_top: 6.0,
                width: "100%",
                flex_wrap: "wrap",
                TimelineRuntimeMetrics { controls: controls.clone(), last_event: event() }
                TimelineResolutionMetrics { controls: controls.clone() }
                Metric { label: "Command", value: command() }
            }
            TimelinePlanReadout { controls: controls.clone() }
        }

        Section {
            title: "Lifecycle callbacks",
            description: "begin / render / loop / complete / cancel and Timeline::call are observable independently from Dioxus rendering.",
            flex {
                width: "100%",
                flex_wrap: "wrap",
                Metric { label: "Begin", value: begin_count().to_string() }
                Metric { label: "Loops", value: loop_count().to_string() }
                Metric { label: "Complete", value: complete_count().to_string() }
                Metric { label: "Cancel", value: cancel_count().to_string() }
                Metric { label: "Call node", value: call_count().to_string() }
            }
        }
    }
}

#[component]
fn TimelineResolutionMetrics(controls: AnimationControls) -> Element {
    let snapshot = use_animation_snapshot(&controls);
    let _ = snapshot();
    let backend = controls
        .lowering_report()
        .map(|value| format!("{:?}", value.selected))
        .unwrap_or_else(|| "Pending".to_string());
    rsx! {
        Metric { label: "Backend", value: backend }
        Metric { label: "Ready", value: controls.is_ready().to_string() }
    }
}

#[component]
fn TimelinePlanReadout(controls: AnimationControls) -> Element {
    let snapshot = use_animation_snapshot(&controls);
    let _ = snapshot();
    let plan = controls
        .lowering_report()
        .map(|value| {
            format!(
                "{} targets · {} properties · {} tweens",
                value.target_count, value.property_count, value.tween_count
            )
        })
        .unwrap_or_else(|| "Waiting for targets".to_string());
    rsx! {
        text {
            margin_top: 4.0,
            font_size: 11.0,
            font_color: "#FF475569",
            "{plan}"
        }
    }
}

#[component]
fn TimelineRuntimeMetrics(controls: AnimationControls, last_event: String) -> Element {
    let snapshot = use_animation_snapshot(&controls);
    let render_count = use_signal(|| 0_u64);
    controls.on_render(move || {
        let mut render_count = render_count;
        render_count += 1;
    });
    let state = snapshot()
        .map(|value| format!("{:?}", value.state))
        .unwrap_or_else(|| {
            if controls.is_ready() {
                "Ready".to_string()
            } else {
                "Resolving".to_string()
            }
        });
    let direction = snapshot()
        .map(|value| format!("{:?}", value.direction))
        .unwrap_or_else(|| "-".to_string());
    let elapsed = snapshot()
        .map(|value| format!("{:.0} ms", time_point_millis(value.elapsed)))
        .unwrap_or_else(|| "0 ms".to_string());
    rsx! {
        Metric { label: "State", value: state }
        Metric { label: "Direction", value: direction }
        Metric { label: "Elapsed", value: elapsed }
        Metric { label: "Render commits", value: render_count().to_string() }
        Metric { label: "Last event", value: last_event }
    }
}

#[component]
fn TimelineCard() -> Element {
    let _target = use_animation_target(CARD_TARGET);
    rsx! {
        column {
            width: 150.0,
            height: 112.0,
            align_items: "center",
            justify_content: "center",
            background_color: "#FF4F46E5",
            border_radius: 22.0,
            text { font_size: 17.0, font_weight: 700, font_color: "#FFFFFFFF", "Timeline" }
            text { margin_top: 5.0, font_size: 11.0, font_color: "#FFE0E7FF", "keyframes + color" }
        }
    }
}

#[component]
fn TimelineBadge() -> Element {
    let _target = use_animation_target(BADGE_TARGET);
    rsx! {
        column {
            margin_left: 18.0,
            width: 74.0,
            height: 74.0,
            align_items: "center",
            justify_content: "center",
            background_color: "#FF0F766E",
            border_radius: 37.0,
            text { font_size: 12.0, font_weight: 700, font_color: "#FFFFFFFF", "label" }
            text { font_size: 10.0, font_color: "#FFCCFBF1", "+180ms" }
        }
    }
}

fn demo_timeline(marker: Signal<u32>) -> Timeline {
    let duration = TimeSpan::from_millis(1_100);
    let card = Animation::new(target(CARD_TARGET))
        .tween(&TRANSLATE_X, Length::vp(-48.0), Length::vp(48.0), duration)
        .configure_last(
            cubic_out(),
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        )
        .keyframes(
            &SCALE_X,
            [
                PropertyKeyframe::new(0.0, 0.72).easing(cubic_out()),
                PropertyKeyframe::new(0.55, 1.12).easing(Easing::Spring(SpringSpec::default())),
                PropertyKeyframe::new(1.0, 1.0),
            ],
            duration,
        )
        .expect("constant keyframes are valid")
        .keyframes(
            &SCALE_Y,
            [
                PropertyKeyframe::new(0.0, 0.72).easing(cubic_out()),
                PropertyKeyframe::new(0.55, 1.12).easing(Easing::Spring(SpringSpec::default())),
                PropertyKeyframe::new(1.0, 1.0),
            ],
            duration,
        )
        .expect("constant keyframes are valid")
        .keyframes(
            &ROTATION,
            [
                PropertyKeyframe::new(0.0, Angle::degrees(-10.0)),
                PropertyKeyframe::new(0.5, Angle::degrees(7.0)),
                PropertyKeyframe::new(1.0, Angle::degrees(0.0)),
            ],
            duration,
        )
        .expect("constant keyframes are valid")
        .tween(
            &BACKGROUND_COLOR,
            color(0xff0f766eu32),
            color(0xff7c3aedu32),
            duration,
        )
        .configure_last(
            Easing::Builtin(BuiltinEase::Sine(EaseDirection::InOut)),
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        );

    let badge = Animation::new(target(BADGE_TARGET))
        .tween(&OPACITY, 0.1, 1.0, TimeSpan::from_millis(560))
        .configure_last(
            cubic_out(),
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        )
        .tween(
            &TRANSLATE_Y,
            Length::vp(48.0),
            Length::vp(-18.0),
            TimeSpan::from_millis(560),
        )
        .configure_last(
            Easing::Spring(SpringSpec::default()),
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        );

    let nested = Timeline::new().add(
        Animation::new(target(BADGE_TARGET))
            .tween(
                &ROTATION,
                Angle::degrees(0.0),
                Angle::degrees(360.0),
                TimeSpan::from_millis(420),
            )
            .configure_last(
                Easing::Builtin(BuiltinEase::Back {
                    direction: EaseDirection::Out,
                    overshoot: 1.7,
                }),
                Composition::Replace,
                Modifier::Identity,
                TimeSpan::ZERO,
                0,
            ),
        TimelinePosition::START,
    );

    Timeline::new()
        .label(LabelName::owned("intro"), TimelinePosition::START)
        .add(
            card,
            TimelinePosition::Label {
                label: LabelName::owned("intro"),
                offset: TimeOffset::ZERO,
            },
        )
        .add(
            badge,
            TimelinePosition::Label {
                label: LabelName::owned("intro"),
                offset: TimeOffset::from_millis(180),
            },
        )
        .set(
            target(BADGE_TARGET),
            &BORDER_COLOR,
            color(0xffffffffu32),
            Timeline::at(580).expect("constant position"),
        )
        .timer(
            TimeSpan::from_millis(90),
            TimelinePosition::PreviousEnd(TimeOffset::ZERO),
        )
        .nested(
            nested,
            TimelinePosition::PreviousEnd(TimeOffset::from_millis(-120)),
        )
        .call(
            move || {
                let mut marker = marker;
                marker += 1;
            },
            CallPolicy::BothDirections,
            TimelinePosition::PreviousEnd(TimeOffset::ZERO),
        )
        .iterations(IterationCount::finite(2).expect("non-zero iteration count"))
        .alternate(true)
        .loop_delay(TimeSpan::from_millis(160))
        .execution_policy(ExecutionPolicy::Auto)
}

fn point_ms(milliseconds: u64) -> TimePoint {
    TimePoint::from_nanos(milliseconds * 1_000_000)
}

fn time_point_millis(point: TimePoint) -> f64 {
    point.as_nanos() as f64 / 1_000_000.0
}
