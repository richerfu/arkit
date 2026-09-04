use std::sync::Arc;
use std::time::Duration;

use arkit_prelude::*;

use crate::{VideoController, VideoSnapshot};

/// Labels used by the built-in video controls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoControlLabels {
    pub play: Arc<str>,
    pub pause: Arc<str>,
    pub rewind: Arc<str>,
    pub forward: Arc<str>,
    pub stop: Arc<str>,
    pub mute: Arc<str>,
    pub unmute: Arc<str>,
    pub loop_on: Arc<str>,
    pub loop_off: Arc<str>,
    pub fullscreen: Arc<str>,
    pub exit_fullscreen: Arc<str>,
}

impl Default for VideoControlLabels {
    fn default() -> Self {
        Self {
            play: "▶".into(),
            pause: "Ⅱ".into(),
            rewind: "后退".into(),
            forward: "前进".into(),
            stop: "停止".into(),
            mute: "静音".into(),
            unmute: "有声".into(),
            loop_on: "循环开".into(),
            loop_off: "循环关".into(),
            fullscreen: "全屏".into(),
            exit_fullscreen: "退出".into(),
        }
    }
}

/// Visual tokens for the built-in controls. Every field can be overridden.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VideoControlsStyle {
    pub overlay_color: u32,
    pub button_color: u32,
    pub text_color: u32,
    pub accent_color: u32,
    pub track_color: u32,
    pub thumb_color: u32,
    pub font_size: f32,
    pub button_font_size: f32,
    pub button_width: f32,
    pub control_height: f32,
    pub button_radius: f32,
    pub horizontal_padding: f32,
    pub vertical_padding: f32,
    pub gap: f32,
}

impl Default for VideoControlsStyle {
    fn default() -> Self {
        Self {
            overlay_color: 0xB0000000,
            button_color: 0x3DFFFFFF,
            text_color: 0xFFFFFFFF,
            accent_color: 0xFF3B82F6,
            track_color: 0x66FFFFFF,
            thumb_color: 0xFFFFFFFF,
            font_size: 12.0,
            button_font_size: 12.0,
            button_width: 42.0,
            control_height: 38.0,
            button_radius: 10.0,
            horizontal_padding: 8.0,
            vertical_padding: 6.0,
            gap: 4.0,
        }
    }
}

/// Feature and appearance configuration for the built-in control overlay.
///
/// Set [`crate::VideoPlayerProps::controls`] to `Some(Default::default())` for
/// the standard controls. Applications that need completely custom markup can
/// leave it disabled and drive the same [`VideoController`] from their own UI.
#[derive(Clone, Debug, PartialEq)]
pub struct VideoControls {
    pub show_play_pause: bool,
    pub show_rewind: bool,
    pub show_forward: bool,
    pub show_stop: bool,
    pub show_progress: bool,
    pub show_time: bool,
    pub show_mute: bool,
    pub show_loop: bool,
    pub show_playback_rate: bool,
    pub show_fullscreen: bool,
    pub seek_step: Duration,
    pub playback_rates: Vec<f32>,
    /// Hide the overlay after this delay while playing. `None` keeps it visible.
    pub auto_hide: Option<Duration>,
    pub labels: VideoControlLabels,
    pub style: VideoControlsStyle,
}

impl Default for VideoControls {
    fn default() -> Self {
        Self {
            show_play_pause: true,
            show_rewind: true,
            show_forward: true,
            show_stop: false,
            show_progress: true,
            show_time: true,
            show_mute: true,
            show_loop: false,
            show_playback_rate: true,
            show_fullscreen: true,
            seek_step: Duration::from_secs(10),
            playback_rates: vec![0.5, 1.0, 1.5, 2.0],
            auto_hide: Some(Duration::from_secs(3)),
            labels: VideoControlLabels::default(),
            style: VideoControlsStyle::default(),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub(crate) struct BuiltInVideoControlsProps {
    pub(crate) controller: VideoController,
    pub(crate) snapshot: VideoSnapshot,
    pub(crate) configuration: VideoControls,
    pub(crate) safe_bottom: f32,
    pub(crate) on_interaction: EventHandler<()>,
}

#[component]
pub(crate) fn BuiltInVideoControls(props: BuiltInVideoControlsProps) -> Element {
    let configuration = props.configuration;
    let style = configuration.style;
    let snapshot = props.snapshot;
    let mut scrub_position = use_signal(|| None::<f32>);
    let controller = props.controller;
    let on_interaction = props.on_interaction;
    let seek_seconds = configuration.seek_step.as_secs_f64();
    let duration_seconds = snapshot.progress.duration.as_secs_f32();
    let position_seconds = scrub_position().unwrap_or(snapshot.progress.position.as_secs_f32());
    let slider_max = duration_seconds.max(0.001);
    let time = format!(
        "{} / {}",
        format_duration(Duration::from_secs_f32(position_seconds)),
        format_duration(snapshot.progress.duration),
    );
    let play_label = if snapshot.status.is_playing() {
        configuration.labels.pause.clone()
    } else {
        configuration.labels.play.clone()
    };
    let mute_label = if snapshot.muted {
        configuration.labels.unmute.clone()
    } else {
        configuration.labels.mute.clone()
    };
    let loop_label = if snapshot.looping {
        configuration.labels.loop_off.clone()
    } else {
        configuration.labels.loop_on.clone()
    };
    let fullscreen_label = if snapshot.fullscreen {
        configuration.labels.exit_fullscreen.clone()
    } else {
        configuration.labels.fullscreen.clone()
    };
    let rates = configuration.playback_rates.clone();
    let seek_controller = controller.clone();
    let play_controller = controller.clone();
    let rewind_controller = controller.clone();
    let forward_controller = controller.clone();
    let stop_controller = controller.clone();
    let loop_controller = controller.clone();
    let rate_controller = controller.clone();
    let mute_controller = controller.clone();
    let fullscreen_controller = controller;

    rsx! {
        column {
            width: "100%",
            padding_top: style.vertical_padding,
            padding_right: style.horizontal_padding,
            padding_bottom: style.vertical_padding + props.safe_bottom,
            padding_left: style.horizontal_padding,
            background_color: style.overlay_color,
            hit_test_behavior: "default",
            onclick: move |event| {
                event.stop_propagation();
                on_interaction.call(());
            },
            if configuration.show_progress {
                slider {
                    width: "100%",
                    slider_value: position_seconds.min(slider_max),
                    slider_min: 0.0,
                    slider_max,
                    slider_step: 0.1,
                    selected_color: style.accent_color,
                    track_color: style.track_color,
                    block_color: style.thumb_color,
                    on_change: move |event| {
                        on_interaction.call(());
                        let seconds = f64::from(event.data().float_value).clamp(0.0, f64::from(slider_max));
                        match event.data().int_value {
                            0 | 1 => scrub_position.set(Some(seconds as f32)),
                            2 | 3 => {
                                scrub_position.set(None);
                                let _ = seek_controller.seek(Duration::from_secs_f64(seconds));
                            }
                            _ => {}
                        }
                    }
                }
            }
            row {
                margin_top: if configuration.show_progress { style.gap } else { 0.0 },
                width: "100%",
                height: style.control_height,
                align_items: "center",
                if configuration.show_play_pause {
                    ControlButton {
                        label: play_label,
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            let _ = play_controller.toggle();
                        },
                    }
                }
                if configuration.show_rewind {
                    ControlButton {
                        label: format!("−{}", compact_seconds(seek_seconds)),
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            let _ = rewind_controller.seek_by(-seek_seconds);
                        },
                    }
                }
                if configuration.show_forward {
                    ControlButton {
                        label: format!("+{}", compact_seconds(seek_seconds)),
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            let _ = forward_controller.seek_by(seek_seconds);
                        },
                    }
                }
                if configuration.show_stop {
                    ControlButton {
                        label: configuration.labels.stop.clone(),
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            let _ = stop_controller.stop();
                        },
                    }
                }
                if configuration.show_time {
                    row {
                        layout_weight: 1.0,
                        height: "100%",
                        align_items: "center",
                        justify_content: "center",
                        text {
                            max_lines: 1,
                            font_size: style.font_size,
                            font_color: style.text_color,
                            text_overflow: "ellipsis",
                            "{time}"
                        }
                    }
                } else {
                    row { layout_weight: 1.0 }
                }
                if configuration.show_loop {
                    ControlButton {
                        label: loop_label,
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            let _ = loop_controller.set_looping(!snapshot.looping);
                        },
                    }
                }
                if configuration.show_playback_rate {
                    ControlButton {
                        label: format!("{:.2}x", snapshot.playback_rate),
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            if let Some(rate) = next_rate(snapshot.playback_rate, &rates) {
                                let _ = rate_controller.set_playback_rate(rate);
                            }
                        },
                    }
                }
                if configuration.show_mute {
                    ControlButton {
                        label: mute_label,
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            let _ = mute_controller.set_muted(!snapshot.muted);
                        },
                    }
                }
                if configuration.show_fullscreen {
                    ControlButton {
                        label: fullscreen_label,
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            let _ = fullscreen_controller.toggle_fullscreen();
                        },
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ControlButtonProps {
    #[props(into)]
    label: Arc<str>,
    style: VideoControlsStyle,
    onclick: EventHandler<()>,
}

#[component]
fn ControlButton(props: ControlButtonProps) -> Element {
    rsx! {
        button {
            margin_left: props.style.gap,
            width: props.style.button_width,
            height: props.style.control_height,
            padding: 2.0,
            border_radius: props.style.button_radius,
            background_color: props.style.button_color,
            font_color: props.style.text_color,
            font_size: props.style.button_font_size,
            onclick: move |_| props.onclick.call(()),
            "{props.label}"
        }
    }
}

fn next_rate(current: f32, rates: &[f32]) -> Option<f32> {
    let first = rates
        .iter()
        .copied()
        .find(|rate| rate.is_finite() && (0.125..=4.0).contains(rate))?;
    rates
        .iter()
        .copied()
        .filter(|rate| rate.is_finite() && (0.125..=4.0).contains(rate))
        .find(|rate| *rate > current + f32::EPSILON)
        .or(Some(first))
}

fn compact_seconds(seconds: f64) -> String {
    if seconds.fract().abs() < f64::EPSILON {
        format!("{seconds:.0}")
    } else {
        format!("{seconds:.1}")
    }
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    if hours == 0 {
        format!("{minutes:02}:{seconds:02}")
    } else {
        format!("{hours}:{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_cycle_skips_invalid_values_and_wraps() {
        let rates = [f32::NAN, 0.5, 1.0, 5.0, 2.0];
        assert_eq!(next_rate(1.0, &rates), Some(2.0));
        assert_eq!(next_rate(2.0, &rates), Some(0.5));
    }

    #[test]
    fn time_labels_cover_short_and_long_media() {
        assert_eq!(format_duration(Duration::from_secs(65)), "01:05");
        assert_eq!(format_duration(Duration::from_secs(3_661)), "1:01:01");
    }
}
