use std::time::Duration;

use arkit::prelude::*;

const SAMPLE_VIDEO: &str = "https://media.w3.org/2010/05/sintel/trailer.mp4";
const SAMPLE_HLS: &str =
    "https://devstreaming-cdn.apple.com/videos/streaming/examples/bipbop_4x3/bipbop_4x3_variant.m3u8";
const SAMPLE_SUBTITLES: &str = "https://media.w3.org/wai/accessibility-intro/W3C_INTRO_SFHI.en.vtt";

fn network_source(key: &str, url: &str) -> VideoSource {
    VideoSource::network(
        VideoNetworkSource::new(url)
            .with_key(key)
            .with_header("User-Agent", "arkit-video-example/1.0"),
    )
}

#[component]
pub fn VideoPage() -> Element {
    let controller = use_hook(VideoController::new);
    let mut source = use_signal(|| network_source("sintel-mp4", SAMPLE_VIDEO));
    let subtitles = use_hook(|| vec![VideoSubtitleSource::url(SAMPLE_SUBTITLES)]);
    let mut status = use_signal(VideoStatus::default);
    let mut progress = use_signal(VideoProgress::default);
    let mut event = use_signal(|| "等待加载".to_string());
    let mut resize_mode = use_signal(VideoResizeMode::default);
    let mut track_count = use_signal(|| 0_usize);
    let mut available_bitrates = use_signal(Vec::<u32>::new);
    let mut subtitle_track = use_signal(|| None::<i32>);
    let mut subtitle_selected = use_signal(|| true);
    let mut hls = use_signal(|| false);
    let controls = use_hook(|| VideoControls {
        show_rewind: false,
        show_forward: false,
        show_stop: true,
        show_loop: true,
        auto_hide: Some(Duration::from_secs(4)),
        style: VideoControlsStyle {
            accent_color: 0xFF22D3EE,
            button_color: 0x5538BDF8,
            ..VideoControlsStyle::default()
        },
        ..VideoControls::default()
    });

    let bitrate_controller = controller.clone();
    let subtitle_controller = controller.clone();
    let elapsed = progress().position.as_secs_f64();
    let duration = progress().duration.as_secs_f64();
    let percent = progress().fraction() * 100.0;
    let resize_label = match resize_mode() {
        VideoResizeMode::Contain => "适应",
        VideoResizeMode::Cover => "裁切",
        VideoResizeMode::Stretch => "拉伸",
        _ => "原始",
    };

    rsx! {
        column {
            width: "100%",
            height: "100%",
            padding: 16.0,
            background_color: "#FF07111F",
            text {
                font_size: 24.0,
                font_weight: 700,
                font_color: "#FFF8FAFC",
                "Native Video"
            }
            text {
                margin_top: 4.0,
                font_size: 12.0,
                font_color: "#FF94A3B8",
                "AVPlayer · XComponent · URL/FD · tracks/subtitles"
            }
            column {
                margin_top: 14.0,
                width: "100%",
                height: 218.0,
                border_radius: 16.0,
                clip: true,
                background_color: "#FF000000",
                VideoPlayer {
                    source: source(),
                    controller: Some(controller.clone()),
                    autoplay: true,
                    resize_mode: resize_mode(),
                    progress_interval: Duration::from_millis(200),
                    subtitles: subtitles.clone(),
                    controls: Some(controls.clone()),
                    width: "100%",
                    height: "100%",
                    on_load_start: move |_| event.set("开始加载".into()),
                    on_load: move |metadata: VideoMetadata| event.set(format!(
                        "已加载 {}x{} · {} 轨",
                        metadata.size.width,
                        metadata.size.height,
                        metadata.tracks.len()
                    )),
                    on_status_change: move |next: VideoStatus| status.set(next),
                    on_progress: move |next: VideoProgress| progress.set(next),
                    on_buffer: move |next: VideoBuffering| event.set(format!("缓冲: {next:?}")),
                    on_seek: move |position: Duration| event.set(format!("跳转到 {:.1}s", position.as_secs_f64())),
                    on_playback_rate_change: move |next: f32| event.set(format!("倍速已生效: {next:.2}x")),
                    on_volume_change: move |next: f32| event.set(format!("音量已生效: {next:.2}")),
                    on_bitrate_change: move |next: u32| event.set(format!("码率已切换: {next} bps")),
                    on_available_bitrates: move |next: Vec<u32>| {
                        let count = next.len();
                        available_bitrates.set(next);
                        event.set(format!("可选码率: {count}"));
                    },
                    on_ready_for_display: move |_| event.set("首帧可显示".into()),
                    on_tracks_change: move |tracks: Vec<VideoTrack>| {
                        track_count.set(tracks.len());
                        subtitle_track.set(tracks.iter().find_map(|track| {
                            (track.track_type == VideoTrackType::Subtitle)
                                .then(|| i32::try_from(track.index).ok())
                                .flatten()
                        }));
                        event.set(format!("媒体轨更新: {}", tracks.len()));
                    },
                    on_subtitle: move |cue: VideoSubtitleCue| event.set(format!("字幕: {}", cue.text)),
                    on_audio_interrupted: move |_| event.set("音频被系统中断".into()),
                    on_fullscreen_change: move |fullscreen: bool| event.set(if fullscreen {
                        "已进入全屏；系统返回键可退出".into()
                    } else {
                        "已退出全屏".into()
                    }),
                    on_end: move |_| event.set("播放结束".into()),
                    on_error: move |error: VideoError| event.set(format!("错误: {error}")),
                }
            }
            text {
                margin_top: 10.0,
                font_size: 12.0,
                font_color: "#FFE2E8F0",
                "{status():?} · {elapsed:.1}/{duration:.1}s · {percent:.0}% · {track_count()} tracks"
            }
            text {
                margin_top: 3.0,
                font_size: 11.0,
                font_color: "#FF94A3B8",
                max_lines: 1,
                text_overflow: "ellipsis",
                "{event()}"
            }
            row {
                margin_top: 10.0,
                width: "100%",
                button {
                    width: "24%",
                    height: 38.0,
                    onclick: move |_| {
                        let next = !hls();
                        hls.set(next);
                        source.set(if next {
                            network_source("apple-bipbop-hls", SAMPLE_HLS)
                        } else {
                            network_source("sintel-mp4", SAMPLE_VIDEO)
                        });
                        subtitle_selected.set(true);
                        event.set(if next { "切换到 HLS".into() } else { "切换到 MP4".into() });
                    },
                    if hls() { "切到 MP4" } else { "切到 HLS" }
                }
                button {
                    margin_left: 5.0,
                    width: "24%",
                    height: 38.0,
                    onclick: move |_| {
                        if let Some(bitrate) = available_bitrates().into_iter().max() {
                            let _ = bitrate_controller.select_bitrate(bitrate);
                        } else {
                            event.set("当前媒体没有可选码率".into());
                        }
                    },
                    "码率 ({available_bitrates().len()})"
                }
                button {
                    margin_left: 5.0,
                    width: "24%",
                    height: 38.0,
                    onclick: move |_| {
                        if let Some(index) = subtitle_track() {
                            let next = !subtitle_selected();
                            let result = if next {
                                subtitle_controller.select_track(index)
                            } else {
                                subtitle_controller.deselect_track(index)
                            };
                            if result.is_ok() {
                                subtitle_selected.set(next);
                            }
                        } else {
                            event.set("尚未发现字幕轨".into());
                        }
                    },
                    if subtitle_selected() { "关闭字幕" } else { "开启字幕" }
                }
                button {
                    margin_left: 5.0,
                    width: "24%",
                    height: 38.0,
                    onclick: move |_| resize_mode.set(match resize_mode() {
                        VideoResizeMode::Contain => VideoResizeMode::Cover,
                        VideoResizeMode::Cover => VideoResizeMode::Stretch,
                        _ => VideoResizeMode::Contain,
                    }),
                    "{resize_label}"
                }
            }
        }
    }
}
