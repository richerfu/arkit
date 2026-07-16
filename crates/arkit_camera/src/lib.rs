//! Native CameraKit preview and photo capture for arkit.
//!
//! The crate is intentionally domain-isolated: applications opt into it with
//! the facade's `camera` feature, so CameraKit and the camera-specific native
//! surface/image dependency edges do not enter the default graph.

mod component;
mod controller;
mod error;
mod mode;
mod model;
mod native;
#[cfg(feature = "scan")]
mod scan;
mod surface;
mod view;
mod worker;

pub use component::{CameraPreview, CameraPreviewProps};
pub use controller::CameraController;
pub use error::{CameraError, CameraErrorKind, CameraResult};
pub use mode::{
    CameraMode, CameraPhotoModeConfiguration, CameraPhotoPreviewInteractions,
    CameraPhotoToolbarConfiguration,
};
#[cfg(feature = "scan")]
pub use mode::{
    CameraScanModeConfiguration, CameraScanPreviewInteractions, CameraScanToolbarConfiguration,
};
pub use model::{CameraProfileSelection, CameraStatus};
pub use ohos_camera_binding::{
    CameraCapabilities, CameraCaptureOptions, CameraControls, CameraExposureMode, CameraFlashMode,
    CameraFloatRange, CameraFocusMode, CameraFocusState, CameraFrame, CameraFrameRateRange,
    CameraImageRotation, CameraLocation, CameraPhotoQuality, CameraPoint, CameraPosition,
    CameraQualityPriority, CameraSessionInfo, CameraSize, CameraStabilizationMode, CameraTorchMode,
    CameraWhiteBalanceMode, CapturedPhoto,
};
#[cfg(feature = "scan")]
pub use scan::{CameraScanConfiguration, CameraScanFormat, CameraScanRegion, CameraScanResult};
pub use view::{CameraView, CameraViewProps};
