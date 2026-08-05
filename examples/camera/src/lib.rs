use std::sync::Arc;

use arkit::entry;
use arkit::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoMode {
    Photo,
    Scan,
}

#[entry]
fn app() -> Element {
    let controller = use_hook(CameraController::new);
    let mut demo_mode = use_signal(|| DemoMode::Photo);
    let mut position = use_signal(|| CameraPosition::Back);
    let mut active = use_signal(|| true);
    let mut status = use_signal(CameraStatus::default);
    let mut result_text = use_signal(|| String::from("相机准备中"));
    let mut error_text = use_signal(String::new);
    let mut capture_count = use_signal(|| 0_u32);
    let mut scan_count = use_signal(|| 0_u32);

    let mode = match demo_mode() {
        DemoMode::Photo => CameraMode::Photo(CameraPhotoModeConfiguration {
            toolbar: CameraPhotoToolbarConfiguration {
                show_flash: true,
                show_zoom: true,
                show_preview_resolution: true,
                show_photo_resolution: true,
                show_camera_switch: true,
                show_shutter: true,
                show_mode_label: true,
                mode_label: Arc::from("照片"),
                panel_color: 0xFF000000,
                control_background_color: 0xFF3A3A3C,
                accent_color: 0xFFFF2D55,
                shutter_color: 0xFFFFFFFF,
                zoom_track_color: 0x99FFFFFF,
                zoom_thumb_color: 0xFFFF2D55,
                control_size: 46.0,
                shutter_size: 76.0,
                zoom_slider_width: 220.0,
                top_bar_height: 58.0,
                bottom_bar_height: 168.0,
                top_inset: 0.0,
                ..CameraPhotoToolbarConfiguration::default()
            },
            interactions: CameraPhotoPreviewInteractions {
                tap_to_focus: true,
                meter_exposure_on_tap: true,
                double_tap_to_switch_camera: true,
                zoom_step: 0.1,
                ..CameraPhotoPreviewInteractions::default()
            },
            ..CameraPhotoModeConfiguration::default()
        }),
        DemoMode::Scan => CameraMode::Scan(CameraScanModeConfiguration {
            scanner: CameraScanConfiguration {
                formats: Arc::from([
                    CameraScanFormat::QrCode,
                    CameraScanFormat::DataMatrix,
                    CameraScanFormat::Code128,
                    CameraScanFormat::Ean13,
                ]),
                continuous: true,
                max_frames_per_second: 10,
                ..CameraScanConfiguration::default()
            },
            toolbar: CameraScanToolbarConfiguration {
                show_torch: true,
                show_zoom: true,
                show_camera_switch: true,
                show_reticle: true,
                show_reticle_scan_line: true,
                show_hint: true,
                show_footer: true,
                hint: Arc::from("扫码模式 · 对准二维码、条码或 Data Matrix"),
                footer: Arc::from("自动识别 · 无需按快门"),
                torch_on_label: Arc::from("轻触关闭"),
                torch_off_label: Arc::from("轻触照亮"),
                panel_color: 0xB3000000,
                control_background_color: 0xB33A3A3C,
                accent_color: 0xFF22C55E,
                zoom_track_color: 0x99FFFFFF,
                zoom_thumb_color: 0xFF22C55E,
                control_size: 46.0,
                reticle_size: 248.0,
                reticle_stroke_width: 3.0,
                reticle_corner_radius: 8.0,
                reticle_corner_length: 34.0,
                reticle_scan_line_color: 0xCC22C55E,
                reticle_scan_line_height: 2.0,
                reticle_scan_line_inset: 18.0,
                reticle_scan_duration: std::time::Duration::from_millis(1_800),
                zoom_slider_width: 220.0,
                top_bar_height: 58.0,
                bottom_bar_height: 86.0,
                top_inset: 0.0,
                ..CameraScanToolbarConfiguration::default()
            },
            interactions: CameraScanPreviewInteractions {
                tap_to_focus: true,
                meter_exposure_on_tap: false,
                double_tap_to_switch_camera: false,
                zoom_step: 0.1,
                ..CameraScanPreviewInteractions::default()
            },
            ..CameraScanModeConfiguration::default()
        }),
    };
    let status_label = status_text(&status());
    let pause_label = if active() { "暂停" } else { "继续" };

    rsx! {
        column {
            width: "100%",
            height: "100%",
            background_color: "#FF020617",
            padding_top: 48.0,

            column {
                width: "100%",
                height: if error_text().is_empty() { 88.0 } else { 108.0 },
                padding_top: 6.0,
                padding_right: 10.0,
                padding_bottom: 6.0,
                padding_left: 10.0,
                background_color: "#FF000000",

                row {
                    width: "100%",
                    height: 40.0,
                    align_items: "center",
                    button {
                        width: 68.0,
                        height: 34.0,
                        border_radius: 17.0,
                        background_color: if demo_mode() == DemoMode::Photo {
                            0xFFFF2D55u32
                        } else {
                            0xFF2C2C2Eu32
                        },
                        onclick: move |_| {
                            demo_mode.set(DemoMode::Photo);
                            result_text.set("拍照模式 · 单击对焦，双击切换镜头".into());
                        },
                        "相机"
                    }
                    button {
                        margin_left: 6.0,
                        width: 68.0,
                        height: 34.0,
                        border_radius: 17.0,
                        background_color: if demo_mode() == DemoMode::Scan {
                            0xFF16A34Au32
                        } else {
                            0xFF2C2C2Eu32
                        },
                        onclick: move |_| {
                            demo_mode.set(DemoMode::Scan);
                            result_text.set("扫码模式 · 自动识别，无需快门".into());
                        },
                        "扫码"
                    }
                    row { layout_weight: 1.0, hit_test_behavior: "transparent" }
                    button {
                        width: 68.0,
                        height: 34.0,
                        padding: 0.0,
                        border_radius: 17.0,
                        background_color: "#FF2C2C2E",
                        font_size: 13.0,
                        onclick: move |_| active.toggle(),
                        "{pause_label}"
                    }
                }

                row {
                    width: "100%",
                    height: 32.0,
                    align_items: "center",
                    text {
                        font_size: 11.0,
                        font_color: "#FFD1D1D6",
                        max_lines: 1,
                        "{status_label}"
                    }
                    row { width: 8.0, hit_test_behavior: "transparent" }
                    row {
                        layout_weight: 1.0,
                        text {
                            font_size: 11.0,
                            font_color: "#FF8E8E93",
                            max_lines: 1,
                            "{result_text}"
                        }
                    }
                }
                if !error_text().is_empty() {
                    text {
                        height: 20.0,
                        font_size: 11.0,
                        font_color: "#FFFF453A",
                        max_lines: 1,
                        "{error_text}"
                    }
                }
            }

            column {
                width: "100%",
                layout_weight: 1.0,
                CameraView {
                    controller: Some(controller.clone()),
                    mode,
                    position: position(),
                    active: active(),
                    width: "100%",
                    height: "100%",
                    on_position_change: move |next| position.set(next),
                    on_status_change: move |next| status.set(next),
                    on_photo: move |photo: CapturedPhoto| {
                        capture_count += 1;
                        result_text.set(format!(
                            "第 {} 张 · {}×{} · {:.1} MB · JPEG",
                            capture_count(),
                            photo.size.width,
                            photo.size.height,
                            photo.bytes().len() as f64 / 1_048_576.0,
                        ));
                        error_text.set(String::new());
                    },
                    on_scan: move |result: CameraScanResult| {
                        scan_count += 1;
                        result_text.set(format!(
                            "第 {} 次 · {} · {}",
                            scan_count(),
                            result.format,
                            result.text,
                        ));
                        error_text.set(String::new());
                    },
                    on_error: move |error: CameraError| error_text.set(error.to_string()),
                }
            }
        }
    }
}

fn status_text(status: &CameraStatus) -> String {
    match status {
        CameraStatus::Idle => "空闲".into(),
        CameraStatus::WaitingForSurface => "等待预览 Surface".into(),
        CameraStatus::Starting(_) => "正在启动 CameraKit".into(),
        CameraStatus::Running(info) => format!(
            "预览 {}×{} · 拍照 {} · 分析 {}",
            info.preview_size.width,
            info.preview_size.height,
            info.photo_size.map_or_else(
                || "关闭".into(),
                |size| format!("{}×{}", size.width, size.height)
            ),
            info.frame_size.map_or_else(
                || "关闭".into(),
                |size| format!("{}×{}", size.width, size.height)
            ),
        ),
        CameraStatus::Capturing(_) => "正在拍照".into(),
        CameraStatus::Stopped => "已暂停".into(),
        CameraStatus::PermissionDenied => "相机权限未授权".into(),
        CameraStatus::Unavailable => "当前设备没有可用相机".into(),
        CameraStatus::Error(error) => format!("错误：{error}"),
    }
}
