//! Animation example — common fade, slide, zoom, and rotate entrance effects.

use arkit::entry;
use arkit::prelude::*;

#[entry]
fn app() -> Element {
    let selected = use_signal(|| TransitionPreset::SlideUp);
    let mut replay = use_signal(|| 0_u32);
    let preset = selected();
    let replay_id = replay() as u64;

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            padding: 20.0,
            background_color: "#fff8fafc",

            text {
                font_size: 28.0,
                font_weight: 700,
                font_color: "#ff0f172a",
                "Animation presets"
            }
            text {
                margin_top: 6.0,
                font_size: 14.0,
                font_color: "#ff64748b",
                "Composable keyframes and easing powered by Timeline"
            }

            column {
                margin_top: 24.0,
                percent_width: 1.0,
                height: 220.0,
                align_items: "center",
                justify_content: "center",
                background_color: "#ffe2e8f0",
                border_radius: 20.0,

                GroupPreview {
                    preset,
                    replay_id,
                }
            }

            button {
                margin_top: 16.0,
                percent_width: 1.0,
                onclick: move |_| replay += 1,
                "Replay"
            }

            column {
                margin_top: 16.0,
                percent_width: 1.0,
                layout_weight: 1.0,
                scroll {
                    percent_width: 1.0,
                    percent_height: 1.0,
                    scroll_bar: true,
                    column {
                        percent_width: 1.0,
                        PresetButton { label: "Fade", preset: TransitionPreset::Fade, selected, replay }
                        PresetButton { label: "Slide up", preset: TransitionPreset::SlideUp, selected, replay }
                        PresetButton { label: "Slide down", preset: TransitionPreset::SlideDown, selected, replay }
                        PresetButton { label: "Slide left", preset: TransitionPreset::SlideLeft, selected, replay }
                        PresetButton { label: "Slide right", preset: TransitionPreset::SlideRight, selected, replay }
                        PresetButton { label: "Zoom in", preset: TransitionPreset::ZoomIn, selected, replay }
                        PresetButton { label: "Zoom out", preset: TransitionPreset::ZoomOut, selected, replay }
                        PresetButton { label: "Rotate clockwise", preset: TransitionPreset::RotateClockwise, selected, replay }
                        PresetButton { label: "Rotate counter-clockwise", preset: TransitionPreset::RotateCounterClockwise, selected, replay }
                    }
                }
            }
        }
    }
}

#[component]
fn GroupPreview(preset: TransitionPreset, replay_id: u64) -> Element {
    let timeline_group = demo_timeline_group(preset);
    let controls = use_timeline_group(timeline_group.clone());
    let mut active_request = use_signal(|| None::<(TransitionPreset, u64)>);
    let request = (preset, replay_id);
    let progress = controls.progress() * 100.0;

    use_effect(move || {
        if !controls.is_ready() || *active_request.peek() == Some(request) {
            return;
        }
        active_request.set(Some(request));
        controls.set_group(timeline_group.clone());
        controls.play();
    });

    rsx! {
        column {
            align_items: "center",
            TimelinePreview { preset, replay_id, progress }
            row {
                margin_top: 14.0,
                height: 24.0,
                align_items: "center",
                for index in 0..7 {
                    StaggerDot { index }
                }
            }
        }
    }
}

#[component]
fn TimelinePreview(preset: TransitionPreset, replay_id: u64, progress: f32) -> Element {
    let _target = use_animation_target("card");

    rsx! {
        column {
            align_items: "center",
            justify_content: "center",
            text {
                font_size: 18.0,
                font_weight: 700,
                font_color: "#ffffffff",
                "{preset_label(preset)}"
            }
            text {
                margin_top: 8.0,
                font_size: 13.0,
                font_color: "#ffe0e7ff",
                "replay #{replay_id} · {progress:.0}%"
            }
        }
    }
}

fn card_timeline(preset: TransitionPreset) -> Timeline {
    Timeline::new(
        preset
            .initial_state()
            .background_color(0xff0f766e)
            .border_radius(8.0)
            .size(150.0, 112.0),
    )
    .to_with(
        AnimationState::new()
            .uniform_scale(1.04)
            .background_color(0xff7c3aed)
            .border_radius(30.0)
            .size(190.0, 136.0),
        320,
        Easing::EaseOutCubic,
    )
    .to_with(
        AnimationState::default()
            .background_color(0xff4f46e5)
            .border_radius(18.0)
            .size(180.0, 124.0),
        140,
        Easing::EaseInOutQuad,
    )
}

#[component]
fn StaggerDot(index: usize) -> Element {
    let _target = use_animation_target(format!("dot-{index}"));
    rsx! {
        column {
            margin_left: 3.0,
            margin_right: 3.0,
        }
    }
}

fn dot_timeline() -> Timeline {
    Timeline::new(
        AnimationState::new()
            .opacity(0.0)
            .translate(0.0, 14.0)
            .uniform_scale(0.25)
            .background_color(0xfff59e0b)
            .border_radius(3.0)
            .size(18.0, 18.0),
    )
    .to_with(
        AnimationState::new()
            .background_color(0xff06b6d4)
            .border_radius(9.0)
            .size(18.0, 18.0),
        280,
        Easing::EaseOutBack,
    )
}

fn demo_timeline_group(preset: TransitionPreset) -> TimelineGroup {
    let mut group = TimelineGroup::new()
        .label_at("intro", 0)
        .label_at("dots", 100)
        .add_at("card", card_timeline(preset), 0);
    let distributor = stagger(45).from_center();
    for index in 0..7 {
        group = group
            .add_at_label(
                format!("dot-{index}"),
                dot_timeline(),
                "dots",
                distributor.delay(index, 7) as i32,
            )
            .expect("the dots label is defined above");
    }
    group
}

#[component]
fn PresetButton(
    label: &'static str,
    preset: TransitionPreset,
    mut selected: Signal<TransitionPreset>,
    mut replay: Signal<u32>,
) -> Element {
    let active = selected() == preset;
    rsx! {
        button {
            margin_bottom: 8.0,
            percent_width: 1.0,
            background_color: if active { "#ff312e81" } else { "#ffffffff" },
            font_color: if active { "#ffffffff" } else { "#ff1e293b" },
            onclick: move |_| {
                selected.set(preset);
                replay += 1;
            },
            "{label}"
        }
    }
}

fn preset_label(preset: TransitionPreset) -> &'static str {
    match preset {
        TransitionPreset::Fade => "Fade",
        TransitionPreset::SlideUp => "Slide up",
        TransitionPreset::SlideDown => "Slide down",
        TransitionPreset::SlideLeft => "Slide left",
        TransitionPreset::SlideRight => "Slide right",
        TransitionPreset::ZoomIn => "Zoom in",
        TransitionPreset::ZoomOut => "Zoom out",
        TransitionPreset::RotateClockwise => "Rotate clockwise",
        TransitionPreset::RotateCounterClockwise => "Rotate counter-clockwise",
    }
}
