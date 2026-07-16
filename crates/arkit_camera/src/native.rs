#[cfg(feature = "scan")]
use crate::CameraScanResult;
use crate::{
    CameraCapabilities, CameraControls, CameraError, CameraFocusState, CameraStatus, CapturedPhoto,
};

/// Events crossing from the camera worker into the arkit UI runtime.
#[derive(Debug)]
pub(crate) enum UiEvent {
    Status(CameraStatus),
    Capabilities(CameraCapabilities),
    Controls(CameraControls),
    FocusState(CameraFocusState),
    Photo(CapturedPhoto),
    #[cfg(feature = "scan")]
    Scan(CameraScanResult),
    Error(CameraError),
}
