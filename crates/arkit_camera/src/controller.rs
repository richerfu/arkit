use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use crate::worker::{CameraControlCommand, WorkerCommand};
use crate::{
    CameraCapabilities, CameraCaptureOptions, CameraControls, CameraError, CameraExposureMode,
    CameraFlashMode, CameraFocusMode, CameraFrameRateRange, CameraImageRotation, CameraPoint,
    CameraQualityPriority, CameraResult, CameraStabilizationMode, CameraStatus, CameraTorchMode,
    CameraWhiteBalanceMode,
};

struct ControllerBinding {
    id: u64,
    sender: Sender<WorkerCommand>,
}

#[derive(Default)]
struct ControllerState {
    next_binding: u64,
    binding: Option<ControllerBinding>,
    status: CameraStatus,
    capabilities: Option<CameraCapabilities>,
    controls: Option<CameraControls>,
}

/// Imperative handle for a mounted [`crate::CameraPreview`].
#[derive(Clone, Default)]
pub struct CameraController {
    inner: Rc<RefCell<ControllerState>>,
}

impl CameraController {
    pub fn new() -> Self {
        Self::default()
    }

    /// Capture one JPEG with the active photo profile.
    pub fn capture(&self) -> CameraResult<()> {
        let state = self.inner.borrow();
        let CameraStatus::Running(info) = state.status else {
            return Err(CameraError::invalid_state(
                "CameraController::capture",
                format!("capture requires Running state, got {:?}", state.status),
            ));
        };
        if !info.supports_photo() {
            return Err(CameraError::unsupported(
                "CameraController::capture",
                "the active camera does not expose JPEG photo capture",
            ));
        }
        let sender = state
            .binding
            .as_ref()
            .map(|binding| binding.sender.clone())
            .ok_or_else(|| {
                CameraError::invalid_state("CameraController::capture", "controller is not mounted")
            })?;
        drop(state);
        sender
            .send(WorkerCommand::Capture(None))
            .map_err(|_| CameraError::worker_stopped("CameraController::capture"))
    }

    /// Capture one JPEG with explicit quality, rotation, mirror, and location options.
    pub fn capture_with_options(&self, options: CameraCaptureOptions) -> CameraResult<()> {
        self.send_running(
            "CameraController::capture_with_options",
            WorkerCommand::Capture(Some(options)),
        )
    }

    pub fn set_flash_mode(&self, value: CameraFlashMode) -> CameraResult<()> {
        self.control(CameraControlCommand::Flash(value))
    }

    pub fn set_torch_mode(&self, value: CameraTorchMode) -> CameraResult<()> {
        self.control(CameraControlCommand::Torch(value))
    }

    pub fn set_zoom(&self, ratio: f32, smooth: bool) -> CameraResult<()> {
        self.control(CameraControlCommand::Zoom { ratio, smooth })
    }

    pub fn set_exposure_mode(&self, value: CameraExposureMode) -> CameraResult<()> {
        self.control(CameraControlCommand::ExposureMode(value))
    }

    pub fn set_exposure_bias(&self, value: f32) -> CameraResult<()> {
        self.control(CameraControlCommand::ExposureBias(value))
    }

    pub fn set_metering_point(&self, value: CameraPoint) -> CameraResult<()> {
        self.control(CameraControlCommand::MeteringPoint(value))
    }

    pub fn set_focus_mode(&self, value: CameraFocusMode) -> CameraResult<()> {
        self.control(CameraControlCommand::FocusMode(value))
    }

    pub fn set_focus_point(&self, value: CameraPoint) -> CameraResult<()> {
        self.control(CameraControlCommand::FocusPoint(value))
    }

    pub fn set_stabilization_mode(&self, value: CameraStabilizationMode) -> CameraResult<()> {
        self.control(CameraControlCommand::Stabilization(value))
    }

    pub fn set_white_balance_mode(&self, value: CameraWhiteBalanceMode) -> CameraResult<()> {
        self.control(CameraControlCommand::WhiteBalanceMode(value))
    }

    pub fn set_white_balance_temperature(&self, value: i32) -> CameraResult<()> {
        self.control(CameraControlCommand::WhiteBalanceTemperature(value))
    }

    pub fn set_macro_enabled(&self, value: bool) -> CameraResult<()> {
        self.control(CameraControlCommand::Macro(value))
    }

    pub fn set_auto_device_switch_enabled(&self, value: bool) -> CameraResult<()> {
        self.control(CameraControlCommand::AutoDeviceSwitch(value))
    }

    pub fn set_quality_priority(&self, value: CameraQualityPriority) -> CameraResult<()> {
        self.control(CameraControlCommand::QualityPriority(value))
    }

    pub fn set_frame_rate(&self, value: CameraFrameRateRange) -> CameraResult<()> {
        self.control(CameraControlCommand::FrameRate(value))
    }

    pub fn set_preview_rotation(&self, value: CameraImageRotation) -> CameraResult<()> {
        self.control(CameraControlCommand::PreviewRotation(value))
    }

    pub fn set_color_space(&self, value: u32) -> CameraResult<()> {
        self.control(CameraControlCommand::ColorSpace(value))
    }

    pub fn status(&self) -> CameraStatus {
        self.inner.borrow().status.clone()
    }

    pub fn capabilities(&self) -> Option<CameraCapabilities> {
        self.inner.borrow().capabilities.clone()
    }

    pub fn controls(&self) -> Option<CameraControls> {
        self.inner.borrow().controls.clone()
    }

    pub fn is_mounted(&self) -> bool {
        self.inner.borrow().binding.is_some()
    }

    pub(crate) fn bind(&self, sender: Sender<WorkerCommand>) -> u64 {
        let mut state = self.inner.borrow_mut();
        state.next_binding = state
            .next_binding
            .checked_add(1)
            .expect("arkit_camera: controller binding id exhausted");
        let id = state.next_binding;
        state.binding = Some(ControllerBinding { id, sender });
        state.status = CameraStatus::WaitingForSurface;
        state.capabilities = None;
        state.controls = None;
        id
    }

    pub(crate) fn update_status(&self, binding: u64, status: CameraStatus) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|current| current.id == binding)
        {
            state.status = status;
        }
    }

    pub(crate) fn update_capabilities(&self, binding: u64, capabilities: CameraCapabilities) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|current| current.id == binding)
        {
            state.capabilities = Some(capabilities);
        }
    }

    pub(crate) fn update_controls(&self, binding: u64, controls: CameraControls) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|current| current.id == binding)
        {
            state.controls = Some(controls);
        }
    }

    pub(crate) fn unbind(&self, binding: u64) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|current| current.id == binding)
        {
            state.binding = None;
            state.status = CameraStatus::Idle;
            state.capabilities = None;
            state.controls = None;
        }
    }

    fn control(&self, command: CameraControlCommand) -> CameraResult<()> {
        self.send_running("CameraController::control", WorkerCommand::Control(command))
    }

    fn send_running(&self, operation: &'static str, command: WorkerCommand) -> CameraResult<()> {
        let state = self.inner.borrow();
        if !state.status.is_running() {
            return Err(CameraError::invalid_state(
                operation,
                format!(
                    "operation requires a running camera, got {:?}",
                    state.status
                ),
            ));
        }
        let sender = state
            .binding
            .as_ref()
            .map(|binding| binding.sender.clone())
            .ok_or_else(|| CameraError::invalid_state(operation, "controller is not mounted"))?;
        drop(state);
        sender
            .send(command)
            .map_err(|_| CameraError::worker_stopped(operation))
    }
}

impl std::fmt::Debug for CameraController {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CameraController")
            .field("mounted", &self.is_mounted())
            .field("status", &self.status())
            .finish()
    }
}

impl PartialEq for CameraController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}
