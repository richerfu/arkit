use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

#[cfg(feature = "scan")]
use arkit_animation::{
    Animation, AnimationSelector, Composition, Easing, ExecutionPolicy, IterationCount, Length,
    Modifier, PropertyKeyframe, TargetName, TimeSpan, Timeline, TimelinePosition, OPACITY,
    TRANSLATE_Y,
};
use arkit_prelude::*;

use crate::{
    CameraCapabilities, CameraCaptureOptions, CameraController, CameraControls, CameraError,
    CameraFlashMode, CameraMode, CameraPhotoPreviewInteractions, CameraPhotoToolbarConfiguration,
    CameraPoint, CameraPosition, CameraPreview, CameraProfileSelection, CameraStatus,
    CapturedPhoto,
};
#[cfg(feature = "scan")]
use crate::{
    CameraScanPreviewInteractions, CameraScanResult, CameraScanToolbarConfiguration,
    CameraTorchMode,
};

#[derive(Props, Clone, PartialEq)]
pub struct CameraViewProps {
    #[props(default)]
    pub controller: Option<CameraController>,
    #[props(default)]
    pub mode: CameraMode,
    #[props(default)]
    pub position: CameraPosition,
    #[props(default = true)]
    pub active: bool,
    #[props(default)]
    pub width: Option<f32>,
    #[props(default)]
    pub height: Option<f32>,
    #[props(default = 1.0)]
    pub percent_width: f32,
    #[props(default)]
    pub percent_height: Option<f32>,
    #[props(default)]
    pub on_position_change: Option<EventHandler<CameraPosition>>,
    #[props(default)]
    pub on_profiles_change: Option<EventHandler<CameraProfileSelection>>,
    #[props(default)]
    pub on_status_change: Option<EventHandler<CameraStatus>>,
    #[props(default)]
    pub on_capabilities_change: Option<EventHandler<CameraCapabilities>>,
    #[props(default)]
    pub on_controls_change: Option<EventHandler<CameraControls>>,
    #[props(default)]
    pub on_photo: Option<EventHandler<CapturedPhoto>>,
    #[cfg(feature = "scan")]
    #[props(default)]
    pub on_scan: Option<EventHandler<CameraScanResult>>,
    #[props(default)]
    pub on_error: Option<EventHandler<CameraError>>,
}

/// Opinionated camera surface with distinct, configurable photo and scan modes.
///
/// Use [`CameraPreview`] directly when the application supplies its own chrome.
#[component]
pub fn CameraView(props: CameraViewProps) -> Element {
    let fallback_controller = use_hook(CameraController::new);
    let controller = props
        .controller
        .clone()
        .unwrap_or_else(|| fallback_controller.clone());
    let mut current_position = use_signal(|| props.position);
    let mut profiles = use_signal(|| props.mode.profiles());
    let mut capabilities = use_signal(|| None::<CameraCapabilities>);
    let mut controls = use_signal(|| None::<CameraControls>);
    let mut status = use_signal(CameraStatus::default);
    let last_tap = use_hook(|| Rc::new(Cell::new(None::<Instant>)));

    use_effect(use_reactive(&props.position, move |position| {
        current_position.set(position);
    }));
    use_effect(use_reactive(&props.mode, move |mode| {
        profiles.set(mode.profiles());
    }));

    let (interactions, photo_toolbar, capture_options) = match &props.mode {
        CameraMode::Photo(photo) => (
            InteractionConfiguration::from(photo.interactions),
            Some(photo.toolbar.clone()),
            Some(photo.capture),
        ),
        #[cfg(feature = "scan")]
        CameraMode::Scan(scan) => (
            InteractionConfiguration::from(scan.interactions),
            None,
            None,
        ),
    };
    #[cfg(feature = "scan")]
    let scan_toolbar = match &props.mode {
        CameraMode::Photo(_) => None,
        CameraMode::Scan(scan) => Some(scan.toolbar.clone()),
    };
    #[cfg(feature = "scan")]
    let scan_configuration = props.mode.scan_configuration();

    let position_change = props.on_position_change;
    let profile_change = props.on_profiles_change;
    let status_change = props.on_status_change;
    let capabilities_change = props.on_capabilities_change;
    let controls_change = props.on_controls_change;
    let photo_handler = props.on_photo;
    #[cfg(feature = "scan")]
    let scan_handler = props.on_scan;
    let error_handler = props.on_error;

    #[cfg(feature = "scan")]
    let preview = rsx! {
        CameraPreview {
            controller: Some(controller.clone()),
            position: current_position(),
            active: props.active,
            profiles: profiles(),
            scan: scan_configuration,
            percent_width: 1.0,
            percent_height: Some(1.0),
            on_status_change: move |next: CameraStatus| {
                status.set(next.clone());
                if let Some(handler) = status_change { handler.call(next); }
            },
            on_capabilities_change: move |next: CameraCapabilities| {
                capabilities.set(Some(next.clone()));
                if let Some(handler) = capabilities_change { handler.call(next); }
            },
            on_controls_change: move |next: CameraControls| {
                controls.set(Some(next.clone()));
                if let Some(handler) = controls_change { handler.call(next); }
            },
            on_photo: move |photo: CapturedPhoto| {
                if let Some(handler) = photo_handler { handler.call(photo); }
            },
            on_scan: move |result: CameraScanResult| {
                if let Some(handler) = scan_handler { handler.call(result); }
            },
            on_error: move |error: CameraError| {
                if let Some(handler) = error_handler { handler.call(error); }
            },
        }
    };
    #[cfg(not(feature = "scan"))]
    let preview = rsx! {
        CameraPreview {
            controller: Some(controller.clone()),
            position: current_position(),
            active: props.active,
            profiles: profiles(),
            percent_width: 1.0,
            percent_height: Some(1.0),
            on_status_change: move |next: CameraStatus| {
                status.set(next.clone());
                if let Some(handler) = status_change { handler.call(next); }
            },
            on_capabilities_change: move |next: CameraCapabilities| {
                capabilities.set(Some(next.clone()));
                if let Some(handler) = capabilities_change { handler.call(next); }
            },
            on_controls_change: move |next: CameraControls| {
                controls.set(Some(next.clone()));
                if let Some(handler) = controls_change { handler.call(next); }
            },
            on_photo: move |photo: CapturedPhoto| {
                if let Some(handler) = photo_handler { handler.call(photo); }
            },
            on_error: move |error: CameraError| {
                if let Some(handler) = error_handler { handler.call(error); }
            },
        }
    };

    let touch_controller = controller.clone();
    let touch_last_tap = last_tap.clone();
    let touch_position_change = position_change;
    let touch_error = error_handler;

    let toolbar = if let Some(configuration) = photo_toolbar.filter(|toolbar| toolbar.visible) {
        rsx! {
            PhotoToolbar {
                controller: controller.clone(),
                configuration,
                interactions,
                controls: controls(),
                capabilities: capabilities(),
                profiles: profiles(),
                status: status(),
                capture: capture_options.unwrap_or_default(),
                on_profiles_change: move |next: CameraProfileSelection| {
                    profiles.set(next);
                    if let Some(handler) = profile_change { handler.call(next); }
                },
                on_position_change: move |_| {
                    let next = current_position().opposite();
                    current_position.set(next);
                    if let Some(handler) = position_change { handler.call(next); }
                },
                on_error: move |error: CameraError| {
                    if let Some(handler) = error_handler { handler.call(error); }
                },
            }
        }
    } else {
        #[cfg(feature = "scan")]
        {
            if let Some(configuration) = scan_toolbar.filter(|toolbar| toolbar.visible) {
                rsx! {
                    ScanToolbar {
                        controller: controller.clone(),
                        configuration,
                        interactions,
                        controls: controls(),
                        status: status(),
                        on_position_change: move |_| {
                            let next = current_position().opposite();
                            current_position.set(next);
                            if let Some(handler) = position_change { handler.call(next); }
                        },
                        on_error: move |error: CameraError| {
                            if let Some(handler) = error_handler { handler.call(error); }
                        },
                    }
                }
            } else {
                rsx! {}
            }
        }
        #[cfg(not(feature = "scan"))]
        {
            rsx! {}
        }
    };

    rsx! {
        stack {
            width: props.width,
            height: props.height,
            percent_width: props.percent_width,
            percent_height: props.percent_height,
            background_color: 0xFF000000u32,

            {preview}

            column {
                percent_width: 1.0,
                percent_height: 1.0,
                justify_content: "space-between",
                hit_test_behavior: 0,
                ontouch: move |event: dioxus_core::Event<dioxus_elements::event::PointerData>| {
                    let Some(pointer) = event.data().pointer else {
                        return;
                    };
                    if pointer.action != dioxus_elements::event::PointerAction::Up {
                        return;
                    }
                    let now = Instant::now();
                    let is_double_tap = touch_last_tap
                        .get()
                        .is_some_and(|last| now.duration_since(last) <= interactions.double_tap_timeout);
                    touch_last_tap.set(Some(now));
                    if is_double_tap && interactions.double_tap_to_switch_camera {
                        touch_last_tap.set(None);
                        let next = current_position().opposite();
                        current_position.set(next);
                        if let Some(handler) = touch_position_change {
                            handler.call(next);
                        }
                        return;
                    }
                    if !interactions.tap_to_focus {
                        return;
                    }
                    let point = CameraPoint::new(
                        f64::from((pointer.x / pointer.target_width.max(1.0)).clamp(0.0, 1.0)),
                        f64::from((pointer.y / pointer.target_height.max(1.0)).clamp(0.0, 1.0)),
                    );
                    if let Err(error) = touch_controller.set_focus_point(point) {
                        if let Some(handler) = touch_error {
                            handler.call(error);
                        }
                    }
                    if interactions.meter_exposure_on_tap {
                        if let Err(error) = touch_controller.set_metering_point(point) {
                            if let Some(handler) = touch_error {
                                handler.call(error);
                            }
                        }
                    }
                },

                {toolbar}
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct InteractionConfiguration {
    tap_to_focus: bool,
    meter_exposure_on_tap: bool,
    double_tap_to_switch_camera: bool,
    double_tap_timeout: std::time::Duration,
    zoom_step: f32,
}

impl From<CameraPhotoPreviewInteractions> for InteractionConfiguration {
    fn from(value: CameraPhotoPreviewInteractions) -> Self {
        Self {
            tap_to_focus: value.tap_to_focus,
            meter_exposure_on_tap: value.meter_exposure_on_tap,
            double_tap_to_switch_camera: value.double_tap_to_switch_camera,
            double_tap_timeout: value.double_tap_timeout,
            zoom_step: value.zoom_step,
        }
    }
}

#[cfg(feature = "scan")]
impl From<CameraScanPreviewInteractions> for InteractionConfiguration {
    fn from(value: CameraScanPreviewInteractions) -> Self {
        Self {
            tap_to_focus: value.tap_to_focus,
            meter_exposure_on_tap: value.meter_exposure_on_tap,
            double_tap_to_switch_camera: value.double_tap_to_switch_camera,
            double_tap_timeout: value.double_tap_timeout,
            zoom_step: value.zoom_step,
        }
    }
}

fn camera_icon_button(
    icon_name: &'static str,
    badge: Option<String>,
    size: f32,
    background_color: u32,
    foreground_color: u32,
    enabled: bool,
    mut on_click: impl FnMut() + 'static,
) -> Element {
    let icon_size = if badge.is_some() {
        size * 0.43
    } else {
        size * 0.5
    };
    let icon = arkit_icon::icon(icon_name, icon_size, foreground_color);
    rsx! {
        button {
            width: size,
            height: size,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            border_width: 0.0,
            border_radius: size / 2.0,
            background_color,
            alignment: 4,
            focusable: false,
            focus_on_touch: false,
            enabled,
            onclick: move |_| on_click(),
            column {
                align_items: "center",
                justify_content: "center",
                {icon}
                if let Some(badge) = badge {
                    text {
                        margin_top: 1.0,
                        font_size: 8.0,
                        font_weight: 600,
                        font_color: foreground_color,
                        "{badge}"
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CameraPillStyle {
    width: f32,
    height: f32,
    background_color: u32,
    foreground_color: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CameraZoomSliderSpec {
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    width: f32,
    selected_color: u32,
    track_color: u32,
    thumb_color: u32,
    thumb_border_color: u32,
    enabled: bool,
}

#[cfg(feature = "scan")]
thread_local! {
    static NEXT_CAMERA_SCAN_LINE_TARGET: Cell<u64> = const { Cell::new(0) };
}

fn camera_zoom_slider(
    spec: CameraZoomSliderSpec,
    mut on_change: impl FnMut(f32) + 'static,
) -> Element {
    const TOUCH_HEIGHT: f32 = 36.0;
    const TRACK_HEIGHT: f32 = 3.0;
    const THUMB_SIZE: f32 = 16.0;
    let width = spec.width.max(96.0);
    let range = (spec.max - spec.min).max(f32::EPSILON);
    let progress = ((spec.value - spec.min) / range).clamp(0.0, 1.0);
    let usable_width = (width - THUMB_SIZE).max(0.0);
    let track_x = THUMB_SIZE / 2.0;
    let track_y = (TOUCH_HEIGHT - TRACK_HEIGHT) / 2.0;
    let selected_width = progress * usable_width;
    let thumb_x = progress * usable_width;
    let thumb_y = (TOUCH_HEIGHT - THUMB_SIZE) / 2.0;
    rsx! {
        stack {
            width,
            height: TOUCH_HEIGHT,
            alignment: 0_i32,
            enabled: spec.enabled,
            ontouch: move |event: dioxus_core::Event<dioxus_elements::event::PointerData>| {
                event.stop_propagation();
                if !spec.enabled {
                    return;
                }
                let Some(pointer) = event.data().pointer else {
                    return;
                };
                if !matches!(
                    pointer.action,
                    dioxus_elements::event::PointerAction::Down
                        | dioxus_elements::event::PointerAction::Move
                        | dioxus_elements::event::PointerAction::Up
                ) {
                    return;
                }
                let progress = (pointer.x / pointer.target_width.max(1.0)).clamp(0.0, 1.0);
                let raw = spec.min + progress * (spec.max - spec.min);
                let step = spec.step.max(0.01);
                let value = (spec.min + ((raw - spec.min) / step).round() * step)
                    .clamp(spec.min, spec.max);
                on_change(value);
            },
            row {
                position: format!("{track_x},{track_y}"),
                width: usable_width,
                height: TRACK_HEIGHT,
                border_radius: TRACK_HEIGHT / 2.0,
                background_color: spec.track_color,
                hit_test_behavior: 2,
            }
            if selected_width > f32::EPSILON {
                row {
                    position: format!("{track_x},{track_y}"),
                    width: selected_width,
                    height: TRACK_HEIGHT,
                    border_radius: TRACK_HEIGHT / 2.0,
                    background_color: spec.selected_color,
                    hit_test_behavior: 2,
                }
            }
            row {
                position: format!("{thumb_x},{thumb_y}"),
                width: THUMB_SIZE,
                height: THUMB_SIZE,
                border_width: 2.0,
                border_color: spec.thumb_border_color,
                border_radius: THUMB_SIZE / 2.0,
                background_color: spec.thumb_color,
                hit_test_behavior: 2,
            }
        }
    }
}

#[cfg(feature = "scan")]
#[component]
fn CameraScanLine(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    travel: f32,
    color: u32,
    duration: std::time::Duration,
    active: bool,
) -> Element {
    let target_name = use_hook(next_camera_scan_line_target_name);
    let target = arkit_animation::use_animation_target(target_name.clone());
    let controls =
        arkit_animation::use_animation(camera_scan_line_timeline(&target_name, travel, duration));
    let animation = controls.clone();

    use_effect(use_reactive(
        (&active, &travel, &duration),
        move |(active, travel, duration)| {
            if !target.is_ready() || !animation.is_ready() {
                return;
            }
            animation.set_timeline(camera_scan_line_timeline(&target_name, travel, duration));
            if active {
                animation.restart();
            } else {
                animation.pause();
            }
        },
    ));

    let line_height = height.max(1.0);
    let glow_height = line_height + 4.0;
    let glow_y = (glow_height - line_height) / 2.0;
    let glow_color = (color & 0x00FF_FFFF) | 0x3300_0000;
    rsx! {
        stack {
            position: format!("{x},{y}"),
            width: width.max(1.0),
            height: glow_height,
            alignment: 0_i32,
            opacity: 0.0_f32,
            hit_test_behavior: 2_i32,
            row {
                width: width.max(1.0),
                height: glow_height,
                border_radius: glow_height / 2.0,
                background_color: glow_color,
                hit_test_behavior: 2_i32,
            }
            row {
                position: format!("0,{glow_y}"),
                width: width.max(1.0),
                height: line_height,
                border_radius: line_height / 2.0,
                background_color: color,
                hit_test_behavior: 2_i32,
            }
        }
    }
}

#[cfg(feature = "scan")]
fn next_camera_scan_line_target_name() -> String {
    NEXT_CAMERA_SCAN_LINE_TARGET.with(|next| {
        let id = next.get();
        next.set(
            id.checked_add(1)
                .expect("camera scan line target id space exhausted"),
        );
        format!("arkit-camera-scan-line-{id}")
    })
}

#[cfg(feature = "scan")]
fn camera_scan_line_timeline(
    target_name: &str,
    travel: f32,
    duration: std::time::Duration,
) -> Timeline {
    let duration_ms = u64::try_from(duration.as_millis())
        .unwrap_or(u64::MAX)
        .max(1);
    let duration = TimeSpan::from_millis(duration_ms);
    let scan = Animation::new(AnimationSelector::Target(TargetName::owned(target_name)))
        .tween(
            &TRANSLATE_Y,
            Length::vp(0.0),
            Length::vp(travel.max(0.0)),
            duration,
        )
        .configure_last(
            Easing::Linear,
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        )
        .keyframes(
            &OPACITY,
            [
                PropertyKeyframe::new(0.0, 0.0),
                PropertyKeyframe::new(0.06, 1.0),
                PropertyKeyframe::new(0.94, 1.0),
                PropertyKeyframe::new(1.0, 0.0),
            ],
            duration,
        )
        .expect("constant camera scan line keyframes are valid");

    Timeline::new()
        .add(scan, TimelinePosition::START)
        .iterations(IterationCount::Infinite)
        .execution_policy(ExecutionPolicy::NativePreferred)
}

fn camera_pill_button(
    icon_name: Option<&'static str>,
    label: String,
    style: CameraPillStyle,
    enabled: bool,
    mut on_click: impl FnMut() + 'static,
) -> Element {
    let icon = icon_name.map(|name| arkit_icon::icon(name, 15.0, style.foreground_color));
    rsx! {
        button {
            width: style.width,
            height: style.height,
            padding_top: 0.0,
            padding_right: 8.0,
            padding_bottom: 0.0,
            padding_left: 8.0,
            border_width: 0.0,
            border_radius: style.height / 2.0,
            background_color: style.background_color,
            alignment: 4,
            focusable: false,
            focus_on_touch: false,
            enabled,
            onclick: move |_| on_click(),
            row {
                align_items: "center",
                justify_content: "center",
                if let Some(icon) = icon {
                    {icon}
                }
                text {
                    margin_left: if icon_name.is_some() { 5.0 } else { 0.0 },
                    font_size: 11.0,
                    font_weight: 600,
                    font_color: style.foreground_color,
                    max_lines: 1,
                    "{label}"
                }
            }
        }
    }
}

fn preview_profile_label(size: Option<crate::CameraSize>) -> String {
    size.map_or_else(
        || "AUTO".to_string(),
        |size| {
            let short_edge = size.width.min(size.height);
            let long_edge = size.width.max(size.height);
            if long_edge >= 3_840 && short_edge >= 2_160 {
                "4K".to_string()
            } else {
                format!("{short_edge}P")
            }
        },
    )
}

fn photo_profile_label(size: Option<crate::CameraSize>) -> String {
    size.map_or_else(
        || "AUTO".to_string(),
        |size| {
            let megapixels = f64::from(size.width) * f64::from(size.height) / 1_000_000.0;
            if megapixels >= 10.0 {
                format!("{megapixels:.0} MP")
            } else {
                format!("{megapixels:.1} MP")
            }
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProfilePickerKind {
    Preview,
    Photo,
}

fn profile_option_label(size: crate::CameraSize, kind: ProfilePickerKind) -> String {
    let divisor = greatest_common_divisor(size.width, size.height).max(1);
    let ratio_width = size.width / divisor;
    let ratio_height = size.height / divisor;
    match kind {
        ProfilePickerKind::Preview => format!(
            "{}×{} · {}:{}",
            size.width, size.height, ratio_width, ratio_height
        ),
        ProfilePickerKind::Photo => {
            let megapixels = f64::from(size.width) * f64::from(size.height) / 1_000_000.0;
            format!(
                "{}×{} · {:.1}MP · {}:{}",
                size.width, size.height, megapixels, ratio_width, ratio_height
            )
        }
    }
}

const fn greatest_common_divisor(mut left: u32, mut right: u32) -> u32 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

#[component]
fn PhotoToolbar(
    controller: CameraController,
    configuration: CameraPhotoToolbarConfiguration,
    interactions: InteractionConfiguration,
    controls: Option<CameraControls>,
    capabilities: Option<CameraCapabilities>,
    profiles: CameraProfileSelection,
    status: CameraStatus,
    capture: CameraCaptureOptions,
    on_profiles_change: EventHandler<CameraProfileSelection>,
    on_position_change: EventHandler<()>,
    on_error: EventHandler<CameraError>,
) -> Element {
    let mut profile_picker = use_signal(|| None::<ProfilePickerKind>);
    let (flash_icon, flash_badge) = controls
        .as_ref()
        .and_then(|controls| controls.flash_mode)
        .map_or(("zap", None), |mode| match mode {
            CameraFlashMode::Off => ("zap-off", Some("关".to_string())),
            CameraFlashMode::On => ("zap", Some("开".to_string())),
            CameraFlashMode::Auto => ("zap", Some("A".to_string())),
            CameraFlashMode::AlwaysOn => ("flashlight", Some("常亮".to_string())),
        });
    let session = status.session();
    let effective_profiles = CameraProfileSelection {
        preview_size: profiles
            .preview_size
            .or_else(|| session.map(|session| session.preview_size)),
        photo_size: profiles
            .photo_size
            .or_else(|| session.and_then(|session| session.photo_size)),
    };
    let preview_resolution_label = preview_profile_label(effective_profiles.preview_size);
    let photo_resolution_label = photo_profile_label(effective_profiles.photo_size);
    let shutter_enabled = matches!(&status, CameraStatus::Running(info) if info.supports_photo());
    let zoom = controls.as_ref().map_or(1.0, |controls| controls.zoom);
    let zoom_min = controls
        .as_ref()
        .map_or(1.0, |controls| controls.zoom_range.min);
    let zoom_max = controls
        .as_ref()
        .map_or(1.0, |controls| controls.zoom_range.max);
    let zoom_label = format!("{zoom:.1}×");
    let zoom_enabled = controls.is_some() && status.is_running() && zoom_max > zoom_min;
    let control_size = configuration.control_size.max(32.0);
    let shutter_size = configuration.shutter_size.max(56.0);
    let top_bar_height =
        configuration.top_bar_height.max(control_size + 12.0) + configuration.top_inset.max(0.0);
    let bottom_bar_height = configuration.bottom_bar_height.max(shutter_size + 78.0)
        + configuration.bottom_inset.max(0.0);
    let flash_controls = controls.clone();
    let preview_capabilities = capabilities.clone();
    let photo_capabilities = capabilities;
    let flash_controller = controller.clone();
    let zoom_controller = controller.clone();
    let capture_controller = controller;
    let current_picker = profile_picker();
    let picker_options = match current_picker {
        Some(ProfilePickerKind::Preview) => preview_capabilities
            .as_ref()
            .map(|capabilities| capabilities.preview_sizes.to_vec())
            .unwrap_or_default(),
        Some(ProfilePickerKind::Photo) => photo_capabilities
            .as_ref()
            .map(|capabilities| capabilities.photo_sizes.to_vec())
            .unwrap_or_default(),
        None => Vec::new(),
    };
    let picker_title = match current_picker {
        Some(ProfilePickerKind::Preview) => "选择预览分辨率",
        Some(ProfilePickerKind::Photo) => "选择照片分辨率",
        None => "",
    };
    let picker_height = 48.0 + 44.0 * picker_options.len().min(6) as f32;
    let mut preview_picker = profile_picker;
    let mut photo_picker = profile_picker;

    let flash_button = camera_icon_button(
        flash_icon,
        flash_badge,
        control_size,
        configuration.control_background_color,
        configuration.foreground_color,
        controls
            .as_ref()
            .is_some_and(|controls| !controls.supported_flash_modes.is_empty()),
        move || {
            let Some(controls) = flash_controls.as_ref() else {
                return;
            };
            let modes = &controls.supported_flash_modes;
            if modes.is_empty() {
                return;
            }
            let index = controls
                .flash_mode
                .and_then(|mode| modes.iter().position(|candidate| *candidate == mode))
                .map_or(0, |index| (index + 1) % modes.len());
            if let Err(error) = flash_controller.set_flash_mode(modes[index]) {
                on_error.call(error);
            }
        },
    );
    let preview_button = camera_pill_button(
        Some("ratio"),
        preview_resolution_label,
        CameraPillStyle {
            width: 76.0,
            height: 34.0,
            background_color: configuration.control_background_color,
            foreground_color: configuration.foreground_color,
        },
        preview_capabilities
            .as_ref()
            .is_some_and(|capabilities| !capabilities.preview_sizes.is_empty()),
        move || {
            let next = (preview_picker() != Some(ProfilePickerKind::Preview))
                .then_some(ProfilePickerKind::Preview);
            preview_picker.set(next);
        },
    );
    let photo_button = camera_pill_button(
        Some("image"),
        photo_resolution_label,
        CameraPillStyle {
            width: 66.0,
            height: 34.0,
            background_color: configuration.control_background_color,
            foreground_color: configuration.foreground_color,
        },
        photo_capabilities
            .as_ref()
            .is_some_and(|capabilities| !capabilities.photo_sizes.is_empty()),
        move || {
            let next = (photo_picker() != Some(ProfilePickerKind::Photo))
                .then_some(ProfilePickerKind::Photo);
            photo_picker.set(next);
        },
    );
    let zoom_slider = camera_zoom_slider(
        CameraZoomSliderSpec {
            value: zoom,
            min: zoom_min,
            max: zoom_max,
            step: interactions.zoom_step,
            width: configuration.zoom_slider_width,
            selected_color: configuration.accent_color,
            track_color: configuration.zoom_track_color,
            thumb_color: configuration.zoom_thumb_color,
            thumb_border_color: configuration.foreground_color,
            enabled: zoom_enabled,
        },
        move |value| {
            if let Err(error) = zoom_controller.set_zoom(value, false) {
                on_error.call(error);
            }
        },
    );
    let switch_button = camera_icon_button(
        "switch-camera",
        None,
        control_size,
        configuration.control_background_color,
        configuration.foreground_color,
        true,
        move || on_position_change.call(()),
    );

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            layout_weight: 1.0,
            row {
                percent_width: 1.0,
                height: top_bar_height,
                padding_top: 6.0 + configuration.top_inset.max(0.0),
                padding_right: 12.0,
                padding_bottom: 6.0,
                padding_left: 12.0,
                align_items: "center",
                background_color: configuration.panel_color,
                ontouch: move |event: dioxus_core::Event<dioxus_elements::event::PointerData>| {
                    event.stop_propagation();
                },
                if configuration.show_flash {
                    {flash_button}
                }
                row {
                    layout_weight: 1.0,
                    hit_test_behavior: 2,
                }
                if configuration.show_preview_resolution {
                    {preview_button}
                }
                if configuration.show_photo_resolution {
                    row { width: 8.0, hit_test_behavior: 2 }
                    {photo_button}
                }
            }

            if let Some(picker_kind) = current_picker {
                row {
                    percent_width: 1.0,
                    height: picker_height,
                    padding_right: 12.0,
                    background_color: 0x33000000u32,
                    ontouch: move |event: dioxus_core::Event<dioxus_elements::event::PointerData>| {
                        event.stop_propagation();
                    },
                    row { layout_weight: 1.0, hit_test_behavior: 2 }
                    column {
                        width: 184.0,
                        height: picker_height,
                        padding_top: 6.0,
                        padding_right: 6.0,
                        padding_bottom: 6.0,
                        padding_left: 6.0,
                        border_radius: 16.0,
                        background_color: 0xF22C2C2Eu32,
                        row {
                            percent_width: 1.0,
                            height: 36.0,
                            padding_left: 10.0,
                            align_items: "center",
                            text {
                                font_size: 12.0,
                                font_weight: 600,
                                font_color: configuration.foreground_color,
                                "{picker_title}"
                            }
                            row { layout_weight: 1.0, hit_test_behavior: 2 }
                            button {
                                width: 32.0,
                                height: 32.0,
                                padding_top: 0.0,
                                padding_right: 0.0,
                                padding_bottom: 0.0,
                                padding_left: 0.0,
                                border_width: 0.0,
                                border_radius: 16.0,
                                background_color: 0x00000000u32,
                                onclick: move |_| profile_picker.set(None),
                                {arkit_icon::icon("x", 17.0, configuration.foreground_color)}
                            }
                        }
                        scroll {
                            percent_width: 1.0,
                            height: picker_height - 48.0,
                            scroll_enabled: picker_options.len() > 6,
                            column {
                                percent_width: 1.0,
                                for size in picker_options.iter().copied() {
                                    button {
                                        percent_width: 1.0,
                                        height: 42.0,
                                        margin_top: 2.0,
                                        border_width: 0.0,
                                        border_radius: 10.0,
                                        background_color: if match picker_kind {
                                            ProfilePickerKind::Preview => effective_profiles.preview_size == Some(size),
                                            ProfilePickerKind::Photo => effective_profiles.photo_size == Some(size),
                                        } {
                                            0x55FFFFFFu32
                                        } else {
                                            0x00000000u32
                                        },
                                        font_size: 9.0,
                                        font_color: configuration.foreground_color,
                                        onclick: move |_| {
                                            let next = match picker_kind {
                                                ProfilePickerKind::Preview => CameraProfileSelection {
                                                    preview_size: Some(size),
                                                    photo_size: effective_profiles.photo_size,
                                                },
                                                ProfilePickerKind::Photo => CameraProfileSelection {
                                                    preview_size: effective_profiles.preview_size,
                                                    photo_size: Some(size),
                                                },
                                            };
                                            on_profiles_change.call(next);
                                            profile_picker.set(None);
                                        },
                                        "{profile_option_label(size, picker_kind)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // API 24 does not consistently honor `space-between` on a
            // full-height Column. Let ArkUI measure the remaining preview
            // height and assign it to a real flex child instead.
            row {
                percent_width: 1.0,
                layout_weight: 1.0,
                hit_test_behavior: 2,
            }

            column {
                percent_width: 1.0,
                height: bottom_bar_height,
                padding_top: 10.0,
                padding_right: 18.0,
                padding_bottom: 10.0 + configuration.bottom_inset.max(0.0),
                padding_left: 18.0,
                align_items: "center",
                background_color: configuration.panel_color,
                ontouch: move |event: dioxus_core::Event<dioxus_elements::event::PointerData>| {
                    event.stop_propagation();
                },
                if configuration.show_zoom {
                    column {
                        height: 48.0,
                        align_items: "center",
                        justify_content: "center",
                        text {
                            font_size: 10.0,
                            font_weight: 600,
                            font_color: configuration.foreground_color,
                            "{zoom_label}"
                        }
                        {zoom_slider}
                    }
                }
                if configuration.show_mode_label {
                    text {
                        margin_top: 2.0,
                        margin_bottom: 6.0,
                        padding_top: 2.0,
                        padding_right: 8.0,
                        padding_bottom: 2.0,
                        padding_left: 8.0,
                        border_radius: 8.0,
                        background_color: configuration.accent_color,
                        font_size: 10.0,
                        font_weight: 600,
                        font_color: configuration.foreground_color,
                        "{configuration.mode_label}"
                    }
                }
                row {
                    percent_width: 1.0,
                    height: shutter_size,
                    align_items: "center",
                    row {
                        layout_weight: 1.0,
                        height: control_size,
                        hit_test_behavior: 2,
                    }
                    if configuration.show_shutter {
                        button {
                            width: shutter_size,
                            height: shutter_size,
                            padding_top: 0.0,
                            padding_right: 0.0,
                            padding_bottom: 0.0,
                            padding_left: 0.0,
                            border_radius: shutter_size / 2.0,
                            border_width: 5.0,
                            border_color: configuration.shutter_color,
                            background_color: 0x0000_0000u32,
                            alignment: 4,
                            focusable: false,
                            focus_on_touch: false,
                            enabled: shutter_enabled,
                            onclick: move |_| {
                                if let Err(error) = capture_controller.capture_with_options(capture) {
                                    on_error.call(error);
                                }
                            },
                            row {
                                width: shutter_size - 14.0,
                                height: shutter_size - 14.0,
                                border_radius: (shutter_size - 14.0) / 2.0,
                                background_color: configuration.shutter_color,
                                hit_test_behavior: 2,
                            }
                        }
                    } else {
                        row {
                            width: shutter_size,
                            height: shutter_size,
                            hit_test_behavior: 2,
                        }
                    }
                    row {
                        layout_weight: 1.0,
                        height: control_size,
                        align_items: "center",
                        row {
                            layout_weight: 1.0,
                            hit_test_behavior: 2,
                        }
                        if configuration.show_camera_switch {
                            {switch_button}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "scan")]
#[component]
fn ScanToolbar(
    controller: CameraController,
    configuration: CameraScanToolbarConfiguration,
    interactions: InteractionConfiguration,
    controls: Option<CameraControls>,
    status: CameraStatus,
    on_position_change: EventHandler<()>,
    on_error: EventHandler<CameraError>,
) -> Element {
    let torch_on = controls
        .as_ref()
        .is_some_and(|controls| controls.torch_mode == CameraTorchMode::On);
    let (torch_icon, torch_badge) = if torch_on {
        ("flashlight", "开")
    } else {
        ("flashlight-off", "关")
    };
    let zoom = controls.as_ref().map_or(1.0, |controls| controls.zoom);
    let zoom_min = controls
        .as_ref()
        .map_or(1.0, |controls| controls.zoom_range.min);
    let zoom_max = controls
        .as_ref()
        .map_or(1.0, |controls| controls.zoom_range.max);
    let zoom_label = format!("{zoom:.1}×");
    let zoom_enabled = controls.is_some() && status.is_running() && zoom_max > zoom_min;
    let torch_controller = controller.clone();
    let zoom_controller = controller.clone();
    let control_size = configuration.control_size.max(32.0);
    let top_bar_height =
        configuration.top_bar_height.max(control_size + 12.0) + configuration.top_inset.max(0.0);
    let bottom_bar_height =
        configuration.bottom_bar_height.max(64.0) + configuration.bottom_inset.max(0.0);
    let reticle_size = configuration.reticle_size.max(96.0);
    let corner_length = configuration
        .reticle_corner_length
        .clamp(16.0, reticle_size / 2.0);
    let reticle_stroke = configuration.reticle_stroke_width.max(1.0);
    let reticle_radius = configuration.reticle_corner_radius.max(0.0);
    let line_radius = (reticle_stroke / 2.0).min(reticle_radius.max(1.0));
    let reticle_end = reticle_size - reticle_stroke;
    let reticle_corner_start = reticle_size - corner_length;
    let reticle_outline_color = (configuration.accent_color & 0x00FF_FFFF) | 0x3300_0000;
    let reticle_scan_line_inset = configuration
        .reticle_scan_line_inset
        .clamp(0.0, reticle_size / 2.0 - 1.0);
    let reticle_scan_line_width = reticle_size - reticle_scan_line_inset * 2.0;
    let reticle_scan_line_height = configuration.reticle_scan_line_height.max(1.0);
    let reticle_scan_line_y = reticle_scan_line_inset;
    let reticle_scan_line_travel =
        (reticle_size - reticle_scan_line_inset * 2.0 - reticle_scan_line_height).max(0.0);
    let torch_action_label = if torch_on {
        configuration.torch_on_label.clone()
    } else {
        configuration.torch_off_label.clone()
    };
    let torch_button = camera_icon_button(
        torch_icon,
        Some(torch_badge.to_string()),
        control_size,
        if torch_on {
            configuration.accent_color
        } else {
            configuration.control_background_color
        },
        if torch_on {
            0xFF00_0000
        } else {
            configuration.foreground_color
        },
        controls
            .as_ref()
            .is_some_and(|controls| controls.torch_supported),
        move || {
            let next = if torch_on {
                CameraTorchMode::Off
            } else {
                CameraTorchMode::On
            };
            if let Err(error) = torch_controller.set_torch_mode(next) {
                on_error.call(error);
            }
        },
    );
    let zoom_slider = camera_zoom_slider(
        CameraZoomSliderSpec {
            value: zoom,
            min: zoom_min,
            max: zoom_max,
            step: interactions.zoom_step,
            width: configuration.zoom_slider_width,
            selected_color: configuration.accent_color,
            track_color: configuration.zoom_track_color,
            thumb_color: configuration.zoom_thumb_color,
            thumb_border_color: configuration.foreground_color,
            enabled: zoom_enabled,
        },
        move |value| {
            if let Err(error) = zoom_controller.set_zoom(value, false) {
                on_error.call(error);
            }
        },
    );
    let switch_button = camera_icon_button(
        "switch-camera",
        None,
        control_size,
        configuration.control_background_color,
        configuration.foreground_color,
        true,
        move || on_position_change.call(()),
    );

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            layout_weight: 1.0,
            row {
                percent_width: 1.0,
                height: top_bar_height,
                padding_top: 6.0 + configuration.top_inset.max(0.0),
                padding_right: 12.0,
                padding_bottom: 6.0,
                padding_left: 12.0,
                align_items: "center",
                background_color: configuration.panel_color,
                ontouch: move |event: dioxus_core::Event<dioxus_elements::event::PointerData>| {
                    event.stop_propagation();
                },
                row {
                    layout_weight: 1.0,
                    hit_test_behavior: 2,
                }
                if configuration.show_camera_switch {
                    {switch_button}
                }
            }

        row {
            percent_width: 1.0,
            layout_weight: 1.0,
            hit_test_behavior: 2,
        }

        column {
            percent_width: 1.0,
            align_items: "center",
            if configuration.show_reticle {
                stack {
                    width: reticle_size,
                    height: reticle_size,
                    alignment: 0_i32,
                    row {
                        width: reticle_size,
                        height: reticle_size,
                        border_width: 1.0,
                        border_color: reticle_outline_color,
                        border_radius: reticle_radius,
                        hit_test_behavior: 2,
                    }
                    if configuration.show_reticle_scan_line {
                        CameraScanLine {
                            x: reticle_scan_line_inset,
                            y: reticle_scan_line_y,
                            width: reticle_scan_line_width,
                            height: reticle_scan_line_height,
                            travel: reticle_scan_line_travel,
                            color: configuration.reticle_scan_line_color,
                            duration: configuration.reticle_scan_duration,
                            active: status.is_running(),
                        }
                    }

                    // Top-left corner.
                    row {
                        position: "0,0",
                        width: corner_length,
                        height: reticle_stroke,
                        border_radius: line_radius,
                        background_color: configuration.accent_color,
                        hit_test_behavior: 2,
                    }
                    row {
                        position: "0,0",
                        width: reticle_stroke,
                        height: corner_length,
                        border_radius: line_radius,
                        background_color: configuration.accent_color,
                        hit_test_behavior: 2,
                    }

                    // Top-right corner.
                    row {
                        position: format!("{reticle_corner_start},0"),
                        width: corner_length,
                        height: reticle_stroke,
                        border_radius: line_radius,
                        background_color: configuration.accent_color,
                        hit_test_behavior: 2,
                    }
                    row {
                        position: format!("{reticle_end},0"),
                        width: reticle_stroke,
                        height: corner_length,
                        border_radius: line_radius,
                        background_color: configuration.accent_color,
                        hit_test_behavior: 2,
                    }

                    // Bottom-left corner.
                    row {
                        position: format!("0,{reticle_end}"),
                        width: corner_length,
                        height: reticle_stroke,
                        border_radius: line_radius,
                        background_color: configuration.accent_color,
                        hit_test_behavior: 2,
                    }
                    row {
                        position: format!("0,{reticle_corner_start}"),
                        width: reticle_stroke,
                        height: corner_length,
                        border_radius: line_radius,
                        background_color: configuration.accent_color,
                        hit_test_behavior: 2,
                    }

                    // Bottom-right corner.
                    row {
                        position: format!("{reticle_corner_start},{reticle_end}"),
                        width: corner_length,
                        height: reticle_stroke,
                        border_radius: line_radius,
                        background_color: configuration.accent_color,
                        hit_test_behavior: 2,
                    }
                    row {
                        position: format!("{reticle_end},{reticle_corner_start}"),
                        width: reticle_stroke,
                        height: corner_length,
                        border_radius: line_radius,
                        background_color: configuration.accent_color,
                        hit_test_behavior: 2,
                    }
                }
            }
            if configuration.show_hint {
                text {
                    margin_top: 18.0,
                    padding: 10.0,
                    background_color: configuration.panel_color,
                    border_radius: 16.0,
                    font_color: configuration.foreground_color,
                    font_size: 14.0,
                    "{configuration.hint}"
                }
            }
            if configuration.show_zoom {
                column {
                    margin_top: 14.0,
                    height: 48.0,
                    align_items: "center",
                    justify_content: "center",
                    text {
                        font_size: 10.0,
                        font_weight: 600,
                        font_color: configuration.foreground_color,
                        "{zoom_label}"
                    }
                    {zoom_slider}
                }
            }
            if configuration.show_torch {
                column {
                    margin_top: 12.0,
                    align_items: "center",
                    {torch_button}
                    text {
                        margin_top: 5.0,
                        font_size: 11.0,
                        font_color: configuration.foreground_color,
                        "{torch_action_label}"
                    }
                }
            }
        }

            row {
                percent_width: 1.0,
                layout_weight: 1.0,
                hit_test_behavior: 2,
            }

            if configuration.show_footer {
                row {
                    percent_width: 1.0,
                    height: bottom_bar_height,
                    padding_bottom: configuration.bottom_inset.max(0.0),
                    align_items: "center",
                    justify_content: "center",
                    background_color: configuration.panel_color,
                    ontouch: move |event: dioxus_core::Event<dioxus_elements::event::PointerData>| {
                        event.stop_propagation();
                    },
                    {arkit_icon::icon("scan-qr-code", 18.0, configuration.accent_color)}
                    text {
                        margin_left: 8.0,
                        font_color: configuration.foreground_color,
                        font_size: 13.0,
                        "{configuration.footer}"
                    }
                }
            } else {
                row {
                    percent_width: 1.0,
                    height: configuration.bottom_inset.max(0.0),
                    hit_test_behavior: 2,
                }
            }
        }
    }
}
