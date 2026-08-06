use arkit::prelude::*;

const ORBIT: &[u8] = include_bytes!("../assets/orbit.json");
const REMOTE_ANIMATION: &str = "https://assets3.lottiefiles.com/packages/lf20_UJNc2t.json";

#[component]
pub fn LottiePage() -> Element {
    let controller = use_hook(LottieController::new);
    let remote_source = use_hook(|| LottieSource::url(REMOTE_ANIMATION));
    let mut status = use_signal(LottieStatus::default);
    let mut frame = use_signal(LottieFrame::default);
    let mut speed = use_signal(|| 1.0_f32);
    let mut repeat = use_signal(|| LottieRepeatMode::Loop);
    let mut fit = use_signal(|| LottieFit::Contain);
    let mut use_network = use_signal(|| false);
    let playing = status().is_playing();
    let progress_percent = frame().progress * 100.0;
    let source_label = if use_network() { "URL" } else { "embedded" };
    let toggle_controller = controller.clone();
    let stop_controller = controller.clone();
    let seek_controller = controller.clone();
    let source = if use_network() {
        remote_source.clone()
    } else {
        LottieSource::embedded("orbit-v1", ORBIT)
    };

    rsx! {
        column {
            width: "100%",
            height: "100%",
            padding: 20.0,
            background_color: "#FF07111F",
            text { font_size: 26.0, font_weight: 700, font_color: "#FFF8FAFC", "Native Lottie" }
            text {
                margin_top: 4.0,
                font_size: 12.0,
                font_color: "#FF94A3B8",
                "XComponent · async URL loading · native-window zero-copy"
            }
            column {
                margin_top: 16.0,
                width: "100%",
                layout_weight: 1.0,
                border_radius: 20.0,
                clip: true,
                background_color: "#FF0F172A",
                LottiePlayer {
                    source,
                    controller: Some(controller.clone()),
                    repeat: repeat(),
                    speed: speed(),
                    fit: fit(),
                    width: "100%",
                    height: "100%",
                    background_color: 0xFF0F_172A_u32,
                    on_status_change: move |next| status.set(next),
                    on_frame: move |next| frame.set(next),
                }
            }
            text {
                margin_top: 12.0,
                font_size: 12.0,
                font_color: "#FFCBD5E1",
                "{status():?} · frame {frame().frame:.1} · {progress_percent:.0}% · {speed():.2}x · {source_label}"
            }
            button {
                margin_top: 10.0,
                width: "100%",
                height: 40.0,
                onclick: move |_| use_network.toggle(),
                if use_network() { "切换到内嵌动画" } else { "加载网络 URL 动画" }
            }
            row {
                margin_top: 10.0,
                width: "100%",
                button {
                    width: "32%",
                    height: 40.0,
                    onclick: move |_| { let _ = toggle_controller.toggle(); },
                    if playing { "暂停" } else { "播放" }
                }
                button {
                    margin_left: 8.0,
                    width: "32%",
                    height: 40.0,
                    onclick: move |_| { let _ = stop_controller.stop(); },
                    "停止"
                }
                button {
                    margin_left: 8.0,
                    width: "32%",
                    height: 40.0,
                    onclick: move |_| { let _ = seek_controller.seek(0.5); },
                    "50%"
                }
            }
            row {
                margin_top: 8.0,
                width: "100%",
                button {
                    width: "32%",
                    height: 38.0,
                    onclick: move |_| speed.set(if speed() >= 2.0 { 0.5 } else { speed() + 0.5 }),
                    "速度"
                }
                button {
                    margin_left: 8.0,
                    width: "32%",
                    height: 38.0,
                    onclick: move |_| repeat.set(match repeat() {
                        LottieRepeatMode::Loop => LottieRepeatMode::Reverse,
                        LottieRepeatMode::Reverse => LottieRepeatMode::None,
                        _ => LottieRepeatMode::Loop,
                    }),
                    "{repeat():?}"
                }
                button {
                    margin_left: 8.0,
                    width: "32%",
                    height: 38.0,
                    onclick: move |_| fit.set(match fit() {
                        LottieFit::Contain => LottieFit::Cover,
                        LottieFit::Cover => LottieFit::Fill,
                        _ => LottieFit::Contain,
                    }),
                    "{fit():?}"
                }
            }
        }
    }
}
