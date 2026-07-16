use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "scan")]
use crate::CameraScanConfiguration;
use crate::{CameraCaptureOptions, CameraProfileSelection};

#[derive(Debug, Clone, PartialEq)]
pub enum CameraMode {
    Photo(CameraPhotoModeConfiguration),
    #[cfg(feature = "scan")]
    Scan(CameraScanModeConfiguration),
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::Photo(CameraPhotoModeConfiguration::default())
    }
}

impl CameraMode {
    pub fn profiles(&self) -> CameraProfileSelection {
        match self {
            Self::Photo(photo) => photo.profiles,
            #[cfg(feature = "scan")]
            Self::Scan(scan) => CameraProfileSelection {
                preview_size: scan.preview_size,
                photo_size: None,
            },
        }
    }

    #[cfg(feature = "scan")]
    pub fn scan_configuration(&self) -> Option<CameraScanConfiguration> {
        match self {
            Self::Photo(_) => None,
            Self::Scan(scan) => Some(scan.scanner.clone()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CameraPhotoModeConfiguration {
    pub profiles: CameraProfileSelection,
    pub capture: CameraCaptureOptions,
    pub toolbar: CameraPhotoToolbarConfiguration,
    pub interactions: CameraPhotoPreviewInteractions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraPhotoToolbarConfiguration {
    pub visible: bool,
    pub show_flash: bool,
    pub show_zoom: bool,
    pub show_preview_resolution: bool,
    pub show_photo_resolution: bool,
    pub show_camera_switch: bool,
    pub show_shutter: bool,
    pub show_mode_label: bool,
    pub mode_label: Arc<str>,
    pub foreground_color: u32,
    pub panel_color: u32,
    pub control_background_color: u32,
    pub accent_color: u32,
    pub shutter_color: u32,
    pub zoom_track_color: u32,
    pub zoom_thumb_color: u32,
    pub control_size: f32,
    pub shutter_size: f32,
    pub zoom_slider_width: f32,
    pub top_bar_height: f32,
    pub bottom_bar_height: f32,
    pub top_inset: f32,
    pub bottom_inset: f32,
}

impl Default for CameraPhotoToolbarConfiguration {
    fn default() -> Self {
        Self {
            visible: true,
            show_flash: true,
            show_zoom: true,
            show_preview_resolution: true,
            show_photo_resolution: true,
            show_camera_switch: true,
            show_shutter: true,
            show_mode_label: true,
            mode_label: Arc::from("照片"),
            foreground_color: 0xFFFF_FFFF,
            panel_color: 0xFF00_0000,
            control_background_color: 0xFF3A_3A3C,
            accent_color: 0xFFFF_2D55,
            shutter_color: 0xFFFF_FFFF,
            zoom_track_color: 0x99FF_FFFF,
            zoom_thumb_color: 0xFFFF_2D55,
            control_size: 46.0,
            shutter_size: 76.0,
            zoom_slider_width: 220.0,
            top_bar_height: 58.0,
            bottom_bar_height: 168.0,
            top_inset: 48.0,
            bottom_inset: 24.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraPhotoPreviewInteractions {
    pub tap_to_focus: bool,
    pub meter_exposure_on_tap: bool,
    pub double_tap_to_switch_camera: bool,
    pub double_tap_timeout: Duration,
    pub smooth_zoom: bool,
    pub zoom_step: f32,
}

impl Default for CameraPhotoPreviewInteractions {
    fn default() -> Self {
        Self {
            tap_to_focus: true,
            meter_exposure_on_tap: true,
            double_tap_to_switch_camera: true,
            double_tap_timeout: Duration::from_millis(350),
            smooth_zoom: true,
            zoom_step: 0.1,
        }
    }
}

#[cfg(feature = "scan")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CameraScanModeConfiguration {
    pub preview_size: Option<crate::CameraSize>,
    pub scanner: CameraScanConfiguration,
    pub toolbar: CameraScanToolbarConfiguration,
    pub interactions: CameraScanPreviewInteractions,
}

#[cfg(feature = "scan")]
#[derive(Debug, Clone, PartialEq)]
pub struct CameraScanToolbarConfiguration {
    pub visible: bool,
    pub show_torch: bool,
    pub show_zoom: bool,
    pub show_camera_switch: bool,
    pub show_reticle: bool,
    pub show_reticle_scan_line: bool,
    pub show_hint: bool,
    pub show_footer: bool,
    pub hint: Arc<str>,
    pub footer: Arc<str>,
    pub torch_on_label: Arc<str>,
    pub torch_off_label: Arc<str>,
    pub foreground_color: u32,
    pub panel_color: u32,
    pub control_background_color: u32,
    pub accent_color: u32,
    pub zoom_track_color: u32,
    pub zoom_thumb_color: u32,
    pub control_size: f32,
    pub reticle_size: f32,
    pub reticle_stroke_width: f32,
    pub reticle_corner_radius: f32,
    pub reticle_corner_length: f32,
    pub reticle_scan_line_color: u32,
    pub reticle_scan_line_height: f32,
    pub reticle_scan_line_inset: f32,
    pub reticle_scan_duration: Duration,
    pub zoom_slider_width: f32,
    pub top_bar_height: f32,
    pub bottom_bar_height: f32,
    pub top_inset: f32,
    pub bottom_inset: f32,
}

#[cfg(feature = "scan")]
impl Default for CameraScanToolbarConfiguration {
    fn default() -> Self {
        Self {
            visible: true,
            show_torch: true,
            show_zoom: true,
            show_camera_switch: true,
            show_reticle: true,
            show_reticle_scan_line: true,
            show_hint: true,
            show_footer: true,
            hint: Arc::from("将二维码或条码放入框内"),
            footer: Arc::from("自动识别 · 无需按快门"),
            torch_on_label: Arc::from("轻触关闭"),
            torch_off_label: Arc::from("轻触照亮"),
            foreground_color: 0xFFFF_FFFF,
            panel_color: 0xB300_0000,
            control_background_color: 0xB33A_3A3C,
            accent_color: 0xFF22_C55E,
            zoom_track_color: 0x99FF_FFFF,
            zoom_thumb_color: 0xFF22_C55E,
            control_size: 46.0,
            reticle_size: 260.0,
            reticle_stroke_width: 3.0,
            reticle_corner_radius: 8.0,
            reticle_corner_length: 36.0,
            reticle_scan_line_color: 0xCC22_C55E,
            reticle_scan_line_height: 2.0,
            reticle_scan_line_inset: 18.0,
            reticle_scan_duration: Duration::from_millis(1_800),
            zoom_slider_width: 220.0,
            top_bar_height: 58.0,
            bottom_bar_height: 86.0,
            top_inset: 48.0,
            bottom_inset: 24.0,
        }
    }
}

#[cfg(feature = "scan")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraScanPreviewInteractions {
    pub tap_to_focus: bool,
    pub meter_exposure_on_tap: bool,
    pub double_tap_to_switch_camera: bool,
    pub double_tap_timeout: Duration,
    pub smooth_zoom: bool,
    pub zoom_step: f32,
}

#[cfg(feature = "scan")]
impl Default for CameraScanPreviewInteractions {
    fn default() -> Self {
        Self {
            tap_to_focus: true,
            meter_exposure_on_tap: false,
            double_tap_to_switch_camera: false,
            double_tap_timeout: Duration::from_millis(350),
            smooth_zoom: true,
            zoom_step: 0.1,
        }
    }
}
