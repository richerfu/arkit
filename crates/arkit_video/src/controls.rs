use std::sync::Arc;
use std::time::Duration;

use arkit_prelude::*;
use arkit_shadcn::components::{Button, ButtonSize, ButtonVariant, Slider, SliderStyle, Spinner};
use arkit_shadcn::theme::{Theme, ThemeMode, ThemePreset, ThemeProvider};

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
            play: "播放".into(),
            pause: "暂停".into(),
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

/// Lucide icon names used by the built-in video controls.
///
/// Names resolve through `arkit_icon`, so applications can replace any glyph
/// with another embedded Lucide icon without replacing the control layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoControlIcons {
    pub play: Arc<str>,
    pub pause: Arc<str>,
    pub rewind: Arc<str>,
    pub forward: Arc<str>,
    pub stop: Arc<str>,
    pub muted: Arc<str>,
    pub audible: Arc<str>,
    pub looping: Arc<str>,
    pub fullscreen: Arc<str>,
    pub exit_fullscreen: Arc<str>,
}

impl Default for VideoControlIcons {
    fn default() -> Self {
        Self {
            play: "play".into(),
            pause: "pause".into(),
            rewind: "rotate-ccw".into(),
            forward: "rotate-cw".into(),
            stop: "square".into(),
            muted: "volume-x".into(),
            audible: "volume-2".into(),
            looping: "repeat-2".into(),
            fullscreen: "maximize-2".into(),
            exit_fullscreen: "minimize-2".into(),
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
    pub icon_size: f32,
    pub progress_touch_height: f32,
    pub progress_track_height: f32,
    pub progress_thumb_size: f32,
    pub loading_size: f32,
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
            // A transparent zinc surface keeps the controls visibly layered
            // over the moving image instead of resembling a separate bar.
            overlay_color: 0x6609090B,
            button_color: 0x00000000,
            text_color: 0xFFFAFAFA,
            accent_color: 0xFFFAFAFA,
            track_color: 0x803F3F46,
            thumb_color: 0xFFFAFAFA,
            font_size: 11.0,
            button_font_size: 12.0,
            icon_size: 17.0,
            progress_touch_height: 44.0,
            progress_track_height: 3.0,
            progress_thumb_size: 12.0,
            loading_size: 18.0,
            button_width: 32.0,
            control_height: 32.0,
            button_radius: 6.0,
            horizontal_padding: 8.0,
            vertical_padding: 4.0,
            gap: 2.0,
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
    /// Prefer compact Lucide glyphs. Disable to render localized text labels.
    pub prefer_icons: bool,
    /// Hide the overlay after this delay while playing. `None` keeps it visible.
    pub auto_hide: Option<Duration>,
    pub labels: VideoControlLabels,
    pub icons: VideoControlIcons,
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
            prefer_icons: true,
            auto_hide: Some(Duration::from_secs(3)),
            labels: VideoControlLabels::default(),
            icons: VideoControlIcons::default(),
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
    pub(crate) seek_revision: u64,
    pub(crate) on_interaction: EventHandler<()>,
    pub(crate) on_seeking_change: EventHandler<bool>,
}

#[component]
pub(crate) fn BuiltInVideoControls(props: BuiltInVideoControlsProps) -> Element {
    let configuration = props.configuration;
    let style = configuration.style;
    let control_theme = controls_theme(style);
    let snapshot = props.snapshot;
    let mut scrub_position = use_signal(|| None::<f32>);
    let mut pending_seek = use_signal(|| None::<f32>);
    let mut acknowledged_seek_revision = use_signal(|| props.seek_revision);
    let completed_seek_revision = props.seek_revision;
    let seek_finished = props.on_seeking_change;
    let seek_started = props.on_seeking_change;
    use_effect(use_reactive(
        (&completed_seek_revision,),
        move |(revision,)| {
            if revision != *acknowledged_seek_revision.peek() {
                acknowledged_seek_revision.set(revision);
                pending_seek.set(None);
                seek_finished.call(false);
            }
        },
    ));
    let controller = props.controller;
    let on_interaction = props.on_interaction;
    let seek_seconds = configuration.seek_step.as_secs_f64();
    let rewind_position = relative_seek_target(
        snapshot.progress.position,
        snapshot.progress.duration,
        -seek_seconds,
    );
    let forward_position = relative_seek_target(
        snapshot.progress.position,
        snapshot.progress.duration,
        seek_seconds,
    );
    let duration_seconds = snapshot.progress.duration.as_secs_f32();
    let pending_position = pending_seek();
    let position_seconds = scrub_position()
        .or(pending_position)
        .unwrap_or(snapshot.progress.position.as_secs_f32());
    let slider_max = duration_seconds.max(0.001);
    let progress_style = SliderStyle {
        touch_target: style.progress_touch_height,
        track_thickness: style.progress_track_height,
        thumb_size: style.progress_thumb_size,
        thumb_color: Some(style.thumb_color),
        thumb_border_color: Some(style.thumb_color),
        thumb_border_width: 0.0,
        track_color: Some(style.track_color),
        selected_color: Some(style.accent_color),
        ..SliderStyle::default()
    };
    let show_loading = pending_position.is_some()
        || matches!(
            snapshot.status,
            crate::VideoStatus::WaitingForSurface
                | crate::VideoStatus::Loading
                | crate::VideoStatus::Buffering
        );
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
    let play_icon = if snapshot.status.is_playing() {
        configuration.icons.pause.clone()
    } else {
        configuration.icons.play.clone()
    };
    let mute_label = if snapshot.muted {
        configuration.labels.unmute.clone()
    } else {
        configuration.labels.mute.clone()
    };
    let mute_icon = if snapshot.muted {
        configuration.icons.muted.clone()
    } else {
        configuration.icons.audible.clone()
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
    let fullscreen_icon = if snapshot.fullscreen {
        configuration.icons.exit_fullscreen.clone()
    } else {
        configuration.icons.fullscreen.clone()
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
        ThemeProvider {
            theme: control_theme,
            stack {
                width: "100%",
                height: "100%",
                alignment: "bottom-start",
                hit_test_behavior: "default",
                onclick: move |event| {
                    event.stop_propagation();
                    on_interaction.call(());
                },
                if show_loading {
                    column {
                        width: "100%",
                        height: "100%",
                        align_items: "center",
                        justify_content: "center",
                        hit_test_behavior: "transparent",
                        row {
                            width: 36.0,
                            height: 36.0,
                            align_items: "center",
                            justify_content: "center",
                            border_radius: 18.0,
                            background_color: style.overlay_color,
                            Spinner {
                                size: style.loading_size,
                                color: Some(style.text_color),
                            }
                        }
                    }
                }
                column {
                    width: "100%",
                    padding_top: 0.0,
                    padding_right: style.horizontal_padding,
                    padding_bottom: style.vertical_padding + props.safe_bottom,
                    padding_left: style.horizontal_padding,
                    background_color: style.overlay_color,
                    if configuration.show_progress {
                        Slider {
                            value: position_seconds.min(slider_max),
                            min: Some(0.0),
                            max: Some(slider_max),
                            step: Some(0.1),
                            height: Some(style.progress_touch_height),
                            style: progress_style,
                            disabled: pending_position.is_some(),
                            on_change: move |seconds: f32| {
                                on_interaction.call(());
                                scrub_position.set(Some(seconds.clamp(0.0, slider_max)));
                            },
                            on_change_end: move |seconds: f32| {
                                on_interaction.call(());
                                let seconds = seconds.clamp(0.0, slider_max);
                                scrub_position.set(None);
                                pending_seek.set(Some(seconds));
                                seek_started.call(true);
                                if seek_controller.seek(Duration::from_secs_f32(seconds)).is_err() {
                                    pending_seek.set(None);
                                    seek_started.call(false);
                                }
                            },
                        }
                    }
                    row {
                        margin_top: if configuration.show_progress { -6.0 } else { 0.0 },
                        width: "100%",
                        height: style.control_height,
                        align_items: "center",
                if configuration.show_play_pause {
                    ControlButton {
                        label: play_label,
                        icon: configuration.prefer_icons.then_some(play_icon),
                        selected: snapshot.status.is_playing(),
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
                        icon: configuration.prefer_icons.then(|| configuration.icons.rewind.clone()),
                        selected: false,
                        disabled: pending_position.is_some(),
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            pending_seek.set(Some(rewind_position.as_secs_f32()));
                            seek_started.call(true);
                            if rewind_controller.seek(rewind_position).is_err() {
                                pending_seek.set(None);
                                seek_started.call(false);
                            }
                        },
                    }
                }
                if configuration.show_forward {
                    ControlButton {
                        label: format!("+{}", compact_seconds(seek_seconds)),
                        icon: configuration.prefer_icons.then(|| configuration.icons.forward.clone()),
                        selected: false,
                        disabled: pending_position.is_some(),
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            pending_seek.set(Some(forward_position.as_secs_f32()));
                            seek_started.call(true);
                            if forward_controller.seek(forward_position).is_err() {
                                pending_seek.set(None);
                                seek_started.call(false);
                            }
                        },
                    }
                }
                if configuration.show_stop {
                    ControlButton {
                        label: configuration.labels.stop.clone(),
                        icon: configuration.prefer_icons.then(|| configuration.icons.stop.clone()),
                        selected: false,
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
                        icon: configuration.prefer_icons.then(|| configuration.icons.looping.clone()),
                        selected: snapshot.looping,
                        style,
                        onclick: move |_| {
                            on_interaction.call(());
                            let _ = loop_controller.set_looping(!snapshot.looping);
                        },
                    }
                }
                if configuration.show_playback_rate {
                    ControlButton {
                        label: format_rate(snapshot.playback_rate),
                        icon: None,
                        selected: (snapshot.playback_rate - 1.0).abs() > f32::EPSILON,
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
                        icon: configuration.prefer_icons.then_some(mute_icon),
                        selected: snapshot.muted,
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
                        icon: configuration.prefer_icons.then_some(fullscreen_icon),
                        selected: snapshot.fullscreen,
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
    }
}

#[derive(Props, Clone, PartialEq)]
struct ControlButtonProps {
    #[props(into)]
    label: Arc<str>,
    icon: Option<Arc<str>>,
    selected: bool,
    #[props(default)]
    disabled: bool,
    style: VideoControlsStyle,
    onclick: EventHandler<()>,
}

#[component]
fn ControlButton(props: ControlButtonProps) -> Element {
    let foreground = if props.selected {
        props.style.accent_color
    } else {
        props.style.text_color
    };
    rsx! {
        row {
            margin_left: props.style.gap,
            Button {
                variant: ButtonVariant::Secondary,
                size: ButtonSize::Icon,
                width: Some(format!("{}", props.style.button_width)),
                height: Some(props.style.control_height),
                border_radius: Some(props.style.button_radius),
                shadow: Some(false),
                disabled: Some(props.disabled),
                onclick: move |_| props.onclick.call(()),
                if let Some(icon) = props.icon {
                    {arkit_shadcn::icon::icon_placeholder(
                        icon.as_ref(),
                        props.style.icon_size,
                        foreground,
                    )}
                } else {
                    text {
                        font_size: props.style.button_font_size,
                        font_weight: 500,
                        font_color: foreground,
                        max_lines: 1,
                        "{props.label}"
                    }
                }
            }
        }
    }
}

fn controls_theme(style: VideoControlsStyle) -> Theme {
    let mut theme = Theme::preset(ThemePreset::Zinc, ThemeMode::Dark);
    theme.colors.background = style.overlay_color;
    theme.colors.foreground = style.text_color;
    theme.colors.secondary = style.button_color;
    theme.colors.secondary_foreground = style.text_color;
    theme.colors.primary = style.accent_color;
    theme.colors.primary_foreground = style.text_color;
    theme.colors.primary_track = style.track_color;
    theme.colors.border = style.track_color;
    theme.radii.md = style.button_radius;
    theme
}

fn format_rate(rate: f32) -> String {
    let value = format!("{rate:.2}");
    let value = value.trim_end_matches('0').trim_end_matches('.');
    format!("{value}×")
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

fn relative_seek_target(position: Duration, duration: Duration, delta_seconds: f64) -> Duration {
    let target = (position.as_secs_f64() + delta_seconds).max(0.0);
    Duration::from_secs_f64(if duration.is_zero() {
        target
    } else {
        target.min(duration.as_secs_f64())
    })
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    if hours == 0 {
        format!("{minutes}:{seconds:02}")
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
        assert_eq!(format_duration(Duration::from_secs(65)), "1:05");
        assert_eq!(format_duration(Duration::from_secs(3_661)), "1:01:01");
    }

    #[test]
    fn playback_rate_labels_are_compact() {
        assert_eq!(format_rate(0.5), "0.5×");
        assert_eq!(format_rate(1.0), "1×");
        assert_eq!(format_rate(1.25), "1.25×");
        assert_eq!(format_rate(2.0), "2×");
    }

    #[test]
    fn relative_seek_targets_are_clamped_to_media_bounds() {
        let duration = Duration::from_secs(60);
        assert_eq!(
            relative_seek_target(Duration::from_secs(5), duration, -10.0),
            Duration::ZERO
        );
        assert_eq!(
            relative_seek_target(Duration::from_secs(55), duration, 10.0),
            duration
        );
    }
}
