use std::num::NonZeroU16;
use std::sync::Arc;

use arkit::prelude::*;

use crate::{cubic_out, restart_forward, reverse_and_play, target, ActionButton, Metric, Section};

const LANES: [(&str, &str, u32); 6] = [
    ("lab-ease-cubic", "Cubic out", 0xff4f46e5u32),
    ("lab-ease-spring", "Spring", 0xff7c3aedu32),
    ("lab-ease-steps", "Steps", 0xff0891b2u32),
    ("lab-ease-bezier", "Cubic Bézier", 0xff0f766eu32),
    ("lab-ease-linear", "Linear points", 0xffea580cu32),
    ("lab-ease-irregular", "Seeded irregular", 0xffdb2777u32),
];
const EASING_DURATION_MS: u64 = 1_200;
const LANE_TRAVEL_VP: f32 = 164.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaggerMode {
    Radial,
    AxisX,
    Reverse,
    Jitter,
}

#[component]
pub(crate) fn EasingLab() -> Element {
    let controls = use_animation(easing_timeline());

    rsx! {
        Section {
            title: "Easing matrix",
            description: "Six easing families run over the same duration and distance: built-in, spring, steps, cubic Bézier, linear points and deterministic irregular easing.",
            column {
                percent_width: 1.0,
                padding: 10.0,
                background_color: 0xfff8fafcu32,
                border_radius: 12.0,
                for (name, label, tint) in LANES {
                    EasingLane { name, label, tint }
                }
            }
            flex {
                margin_top: 12.0,
                percent_width: 1.0,
                flex_wrap: "wrap",
                ActionButton {
                    label: "Run all",
                    on_press: {
                        let controls = controls.clone();
                        move |_| restart_forward(&controls)
                    }
                }
                ActionButton { label: "Pause", on_press: { let controls = controls.clone(); move |_| controls.pause() } }
                ActionButton { label: "Resume", on_press: { let controls = controls.clone(); move |_| controls.resume() } }
                ActionButton {
                    label: "Reverse + play",
                    on_press: {
                        let controls = controls.clone();
                        move |_| reverse_and_play(&controls, point_ms(EASING_DURATION_MS))
                    }
                }
                AnimationStateMetric { controls: controls.clone() }
            }
        }
        StaggerDemo {}
    }
}

#[component]
fn AnimationStateMetric(controls: AnimationControls) -> Element {
    let snapshot = use_animation_snapshot(&controls);
    let state = snapshot()
        .map(|value| format!("{:?}", value.state))
        .unwrap_or_else(|| {
            if controls.is_ready() {
                "Ready".to_string()
            } else {
                "Resolving".to_string()
            }
        });
    rsx! { Metric { label: "State", value: state } }
}

#[component]
fn EasingLane(name: &'static str, label: &'static str, tint: u32) -> Element {
    rsx! {
        row {
            margin_bottom: 7.0,
            percent_width: 1.0,
            align_items: "center",
            text { width: 90.0, font_size: 11.0, font_color: 0xff475569u32, "{label}" }
            column {
                layout_weight: 1.0,
                height: 28.0,
                align_items: "start",
                justify_content: "center",
                background_color: 0xffe2e8f0u32,
                border_radius: 14.0,
                clip: false,
                EasingChip { name, tint }
            }
        }
    }
}

#[component]
fn EasingChip(name: &'static str, tint: u32) -> Element {
    let _target = use_animation_target(name);
    rsx! {
        column {
            width: 28.0,
            height: 28.0,
            background_color: tint,
            border_radius: 14.0,
        }
    }
}

fn easing_timeline() -> Timeline {
    let linear_points = Easing::linear_points(Arc::<[LinearPoint]>::from([
        LinearPoint::new(0.0, 0.0),
        LinearPoint::new(0.25, 0.08),
        LinearPoint::new(0.55, 0.82),
        LinearPoint::new(0.75, 0.68),
        LinearPoint::new(1.0, 1.0),
    ]))
    .expect("constant linear points are ordered");
    let easings = [
        cubic_out(),
        Easing::Spring(SpringSpec::default()),
        Easing::Steps {
            count: NonZeroU16::new(6).expect("non-zero step count"),
            jump: JumpMode::End,
        },
        Easing::cubic_bezier(0.34, 1.56, 0.64, 1.0).expect("constant bezier is valid"),
        linear_points,
        Easing::Irregular(IrregularEase {
            seed: 42,
            strength: 0.32,
            points: NonZeroU16::new(18).expect("non-zero point count"),
        }),
    ];
    LANES
        .iter()
        .zip(easings)
        .fold(Timeline::new(), |timeline, ((name, _, _), easing)| {
            timeline.add(
                Animation::new(target(name))
                    .tween(
                        &TRANSLATE_X,
                        Length::vp(0.0),
                        // Keep headroom for overshooting spring/Bézier eases;
                        // their intermediate value may legitimately exceed 1.
                        Length::vp(LANE_TRAVEL_VP),
                        TimeSpan::from_millis(EASING_DURATION_MS),
                    )
                    .configure_last(
                        easing,
                        Composition::Replace,
                        Modifier::Identity,
                        TimeSpan::ZERO,
                        0,
                    ),
                TimelinePosition::START,
            )
        })
}

#[component]
fn StaggerDemo() -> Element {
    let mut mode = use_signal(|| StaggerMode::Radial);
    let selected = mode();
    let controls = use_animation(stagger_timeline(selected));
    let distribution = stagger_distribution(selected);
    let delay_summary = (0..12)
        .map(|index| distribution.delay(index, 12).to_string())
        .collect::<Vec<_>>()
        .join(", ");

    rsx! {
        Section {
            title: "Grid stagger distribution",
            description: "A 4×3 target grid demonstrates center/radial distance, axis distribution, reverse direction and seeded jitter. Delays are deterministic typed TimeSpan values.",
            column {
                width: 280.0,
                align_items: "center",
                for row_index in 0..3 {
                    row {
                        for column_index in 0..4 {
                            StaggerDot { index: row_index * 4 + column_index }
                        }
                    }
                }
            }
            flex {
                margin_top: 12.0,
                percent_width: 1.0,
                flex_wrap: "wrap",
                for (candidate, label) in [
                    (StaggerMode::Radial, "Radial"),
                    (StaggerMode::AxisX, "Axis X"),
                    (StaggerMode::Reverse, "Reverse"),
                    (StaggerMode::Jitter, "Jitter"),
                ] {
                    ActionButton {
                        label,
                        active: selected == candidate,
                        on_press: {
                            let controls = controls.clone();
                            move |_| {
                                mode.set(candidate);
                                controls.set_timeline(stagger_timeline(candidate));
                                restart_forward(&controls);
                            }
                        }
                    }
                }
                ActionButton {
                    label: "Replay",
                    on_press: {
                        let controls = controls.clone();
                        move |_| restart_forward(&controls)
                    }
                }
            }
            text {
                margin_top: 6.0,
                font_size: 10.0,
                font_color: 0xff64748bu32,
                "delays(ms): {delay_summary}"
            }
        }
    }
}

fn point_ms(milliseconds: u64) -> TimePoint {
    TimePoint::from_nanos(milliseconds * 1_000_000)
}

#[component]
fn StaggerDot(index: usize) -> Element {
    let name = format!("lab-stagger-{index}");
    let _target = use_animation_target(name);
    let tint = if index % 2 == 0 {
        0xff4f46e5u32
    } else {
        0xff06b6d4u32
    };
    rsx! {
        column {
            margin: 6.0,
            width: 48.0,
            height: 48.0,
            align_items: "center",
            justify_content: "center",
            background_color: tint,
            border_radius: 14.0,
            text { font_size: 11.0, font_weight: 700, font_color: 0xffffffffu32, "{index}" }
        }
    }
}

fn stagger_distribution(mode: StaggerMode) -> Stagger {
    let base = stagger(70)
        .grid(StaggerGrid::new(4, 3))
        .from_center()
        .easing(Easing::Builtin(BuiltinEase::Cubic(EaseDirection::InOut)));
    match mode {
        StaggerMode::Radial => base.axis(StaggerAxis::Radial),
        StaggerMode::AxisX => base.axis(StaggerAxis::X),
        StaggerMode::Reverse => base.axis(StaggerAxis::Radial).reverse(),
        StaggerMode::Jitter => base.axis(StaggerAxis::Radial).jitter(0.65, 0xA11CE),
    }
}

fn stagger_timeline(mode: StaggerMode) -> Timeline {
    let distribution = stagger_distribution(mode);
    (0..12).fold(Timeline::new(), |timeline, index| {
        let delay = distribution.delay_span(index, 12);
        let animation = Animation::new(target(&format!("lab-stagger-{index}")))
            .tween(&OPACITY, 0.15, 1.0, TimeSpan::from_millis(480))
            .configure_last(
                cubic_out(),
                Composition::Replace,
                Modifier::Identity,
                delay,
                0,
            )
            .tween(&SCALE_X, 0.35, 1.0, TimeSpan::from_millis(480))
            .configure_last(
                Easing::Spring(SpringSpec::default()),
                Composition::Replace,
                Modifier::Identity,
                delay,
                0,
            )
            .tween(&SCALE_Y, 0.35, 1.0, TimeSpan::from_millis(480))
            .configure_last(
                Easing::Spring(SpringSpec::default()),
                Composition::Replace,
                Modifier::Identity,
                delay,
                0,
            )
            .tween(
                &TRANSLATE_Y,
                Length::vp(36.0),
                Length::vp(0.0),
                TimeSpan::from_millis(480),
            )
            .configure_last(
                cubic_out(),
                Composition::Replace,
                Modifier::Identity,
                delay,
                0,
            );
        timeline.add(animation, TimelinePosition::START)
    })
}
