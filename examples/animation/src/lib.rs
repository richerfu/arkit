//! Complete interactive showcase for the unified Animation v2 engine.

mod easing_lab;
mod interaction_lab;
mod lifecycle_lab;
mod orchestration_lab;
mod timeline_lab;

use arkit::entry;
use arkit::prelude::*;

use easing_lab::EasingLab;
use interaction_lab::InteractionLab;
use lifecycle_lab::LifecycleLab;
use orchestration_lab::OrchestrationLab;
use timeline_lab::TimelineLab;

const BACKGROUND: u32 = 0xfff8fafcu32;
const SURFACE: u32 = 0xffffffffu32;
const BORDER: u32 = 0xffdbe4f0u32;
const TEXT: u32 = 0xff0f172au32;
const MUTED: u32 = 0xff64748bu32;
const PRIMARY_DARK: u32 = 0xff312e81u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
enum ShowcasePage {
    Timeline,
    Easing,
    Lifecycle,
    Interaction,
    Orchestration,
}

impl ShowcasePage {
    const ALL: [(Self, &'static str); 5] = [
        (Self::Timeline, "Timeline"),
        (Self::Easing, "Easing"),
        (Self::Lifecycle, "Lifecycle"),
        (Self::Interaction, "Input"),
        (Self::Orchestration, "Scope"),
    ];
}

#[entry]
fn app() -> Element {
    let mut page = use_signal(|| ShowcasePage::Timeline);
    let selected = page();
    let scroll_reset = format!("0,0,0,{}", selected as i32);

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            background_color: BACKGROUND,
            column {
                percent_width: 1.0,
                padding_top: 16.0,
                padding_right: 16.0,
                padding_bottom: 12.0,
                padding_left: 16.0,
                background_color: SURFACE,
                text {
                    font_size: 26.0,
                    font_weight: 700,
                    font_color: TEXT,
                    "Animation v2 Lab"
                }
                text {
                    margin_top: 4.0,
                    font_size: 13.0,
                    font_color: MUTED,
                    "Timeline · controls · easing · layout · presence · drag · scroll · scope"
                }
                row {
                    margin_top: 12.0,
                    percent_width: 1.0,
                    for (target, label) in ShowcasePage::ALL {
                        button {
                            margin_right: 6.0,
                            percent_width: 0.19,
                            height: 38.0,
                            padding: 0.0,
                            font_size: 12.0,
                            background_color: if selected == target { PRIMARY_DARK } else { 0xffeef2ffu32 },
                            font_color: if selected == target { 0xffffffffu32 } else { PRIMARY_DARK },
                            onclick: move |_| page.set(target),
                            "{label}"
                        }
                    }
                }
            }
            column {
                key: "{selected:?}",
                percent_width: 1.0,
                layout_weight: 1.0,
                scroll {
                    percent_width: 1.0,
                    percent_height: 1.0,
                    scroll_bar: true,
                    scroll_offset: "{scroll_reset}",
                    column {
                        percent_width: 1.0,
                        padding: 14.0,
                        if selected == ShowcasePage::Timeline {
                            TimelineLab {}
                        }
                        if selected == ShowcasePage::Easing {
                            EasingLab {}
                        }
                        if selected == ShowcasePage::Lifecycle {
                            LifecycleLab {}
                        }
                        if selected == ShowcasePage::Interaction {
                            InteractionLab {}
                        }
                        if selected == ShowcasePage::Orchestration {
                            OrchestrationLab {}
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn Section(
    title: &'static str,
    description: &'static str,
    children: Element,
) -> Element {
    rsx! {
        column {
            margin_bottom: 14.0,
            percent_width: 1.0,
            padding: 14.0,
            background_color: SURFACE,
            border_width: 1.0,
            border_color: BORDER,
            border_radius: 16.0,
            text {
                font_size: 18.0,
                font_weight: 700,
                font_color: TEXT,
                "{title}"
            }
            text {
                margin_top: 4.0,
                margin_bottom: 12.0,
                font_size: 12.0,
                font_color: MUTED,
                "{description}"
            }
            {children}
        }
    }
}

#[component]
pub(crate) fn ActionButton(
    label: &'static str,
    on_press: EventHandler<()>,
    #[props(default)] active: bool,
) -> Element {
    rsx! {
        button {
            margin_right: 6.0,
            margin_bottom: 6.0,
            height: 38.0,
            padding_left: 12.0,
            padding_right: 12.0,
            font_size: 12.0,
            background_color: if active { PRIMARY_DARK } else { 0xffeef2ffu32 },
            font_color: if active { 0xffffffffu32 } else { PRIMARY_DARK },
            onclick: move |_| on_press.call(()),
            "{label}"
        }
    }
}

#[component]
pub(crate) fn Metric(label: &'static str, value: String) -> Element {
    rsx! {
        column {
            margin_right: 8.0,
            margin_bottom: 8.0,
            padding_top: 8.0,
            padding_right: 10.0,
            padding_bottom: 8.0,
            padding_left: 10.0,
            background_color: 0xfff1f5f9u32,
            border_radius: 10.0,
            text { font_size: 10.0, font_color: MUTED, "{label}" }
            text { margin_top: 2.0, font_size: 12.0, font_weight: 700, font_color: TEXT, "{value}" }
        }
    }
}

pub(crate) fn cubic_out() -> Easing {
    Easing::Builtin(BuiltinEase::Cubic(EaseDirection::Out))
}

pub(crate) fn target(name: &str) -> AnimationSelector {
    AnimationSelector::Target(TargetName::owned(name))
}

pub(crate) fn color(argb: u32) -> LinearRgba {
    LinearRgba::from_argb(argb)
}

/// Start or continue in the forward direction.
pub(crate) fn play_forward(controls: &AnimationControls) {
    if controls.direction() == Some(PlaybackDirection::Reverse) {
        controls.reverse();
    }
    controls.play();
}

/// Start a new forward pass regardless of the previous terminal direction.
pub(crate) fn restart_forward(controls: &AnimationControls) {
    if controls.direction() == Some(PlaybackDirection::Reverse) {
        controls.reverse();
    }
    controls.restart();
}

/// Reverse a running animation in place, or start a visible pass in the
/// opposite direction when the instance is not currently running.
///
/// `AnimationControls::reverse` intentionally changes direction only. Demo
/// controls use this helper so a button labelled as a playback action never
/// becomes a silent no-op at an idle or terminal boundary.
pub(crate) fn reverse_and_play(controls: &AnimationControls, end: TimePoint) {
    let snapshot = controls.snapshot();
    match snapshot.map(|value| value.state) {
        Some(PlaybackState::Running) => controls.reverse(),
        Some(PlaybackState::Paused) => {
            controls.reverse();
            controls.resume();
        }
        _ => {
            let next_direction = snapshot
                .map(|value| value.direction.reversed())
                .unwrap_or(PlaybackDirection::Reverse);
            controls.reset();
            controls.seek(if next_direction == PlaybackDirection::Reverse {
                end
            } else {
                TimePoint::ZERO
            });
            controls.reverse();
            controls.play();
        }
    }
}
