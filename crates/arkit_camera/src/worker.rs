use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
#[cfg(feature = "scan")]
use std::sync::{mpsc::SyncSender, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[cfg(feature = "scan")]
use ohos_camera_binding::CameraFrameOutputConfiguration;
use ohos_camera_binding::{
    CameraCaptureOptions, CameraConfiguration, CameraEvent, CameraExposureMode, CameraFlashMode,
    CameraFocusMode, CameraFrameRateRange, CameraImageRotation, CameraPoint, CameraQualityPriority,
    CameraSession, CameraStabilizationMode, CameraSurface, CameraTorchMode, CameraWhiteBalanceMode,
    CameraXComponentEvent,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::native::UiEvent;
use crate::{CameraError, CameraPosition, CameraProfileSelection, CameraStatus};
#[cfg(feature = "scan")]
use crate::{CameraFrame, CameraScanConfiguration, CameraScanResult};

const CAMERA_PERMISSION: &str = "ohos.permission.CAMERA";
const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(8);

#[derive(Debug, Clone, PartialEq)]
struct SessionKey {
    surface: CameraSurface,
    position: CameraPosition,
    profiles: CameraProfileSelection,
    #[cfg(feature = "scan")]
    scan: Option<CameraScanConfiguration>,
}

#[derive(Debug)]
pub(crate) enum WorkerCommand {
    Configure {
        active: bool,
        position: CameraPosition,
        profiles: CameraProfileSelection,
        #[cfg(feature = "scan")]
        scan: Option<CameraScanConfiguration>,
    },
    Capture(Option<CameraCaptureOptions>),
    Control(CameraControlCommand),
    Shutdown,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum CameraControlCommand {
    Flash(CameraFlashMode),
    Torch(CameraTorchMode),
    Zoom { ratio: f32, smooth: bool },
    ExposureMode(CameraExposureMode),
    ExposureBias(f32),
    MeteringPoint(CameraPoint),
    FocusMode(CameraFocusMode),
    FocusPoint(CameraPoint),
    Stabilization(CameraStabilizationMode),
    WhiteBalanceMode(CameraWhiteBalanceMode),
    WhiteBalanceTemperature(i32),
    Macro(bool),
    AutoDeviceSwitch(bool),
    QualityPriority(CameraQualityPriority),
    FrameRate(CameraFrameRateRange),
    PreviewRotation(CameraImageRotation),
    ColorSpace(u32),
}

pub(crate) struct WorkerHandle {
    sender: Sender<WorkerCommand>,
    surface_sender: Sender<CameraXComponentEvent>,
    thread: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    pub(crate) fn spawn(events: UnboundedSender<UiEvent>) -> crate::CameraResult<Self> {
        let (sender, receiver) = mpsc::channel();
        let (surface_sender, surface_receiver) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("arkit-camera".into())
            .spawn(move || run_worker(receiver, surface_receiver, events))
            .map_err(|error| {
                CameraError::worker_stopped("spawn camera worker").with_message(error.to_string())
            })?;
        Ok(Self {
            sender,
            surface_sender,
            thread: Some(thread),
        })
    }

    pub(crate) fn sender(&self) -> Sender<WorkerCommand> {
        self.sender.clone()
    }

    pub(crate) fn surface_sender(&self) -> Sender<CameraXComponentEvent> {
        self.surface_sender.clone()
    }

    pub(crate) fn send(&self, command: WorkerCommand) -> crate::CameraResult<()> {
        self.sender
            .send(command)
            .map_err(|_| CameraError::worker_stopped("send camera command"))
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(WorkerCommand::Shutdown);
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                ohos_hilog_binding::error("arkit_camera: worker thread panicked");
            }
        }
    }
}

struct WorkerState {
    desired_active: bool,
    desired_position: CameraPosition,
    desired_profiles: CameraProfileSelection,
    #[cfg(feature = "scan")]
    desired_scan: Option<CameraScanConfiguration>,
    surface: Option<CameraSurface>,
    active_key: Option<SessionKey>,
    session: Option<CameraSession>,
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            desired_active: false,
            desired_position: CameraPosition::Back,
            desired_profiles: CameraProfileSelection::default(),
            #[cfg(feature = "scan")]
            desired_scan: None,
            surface: None,
            active_key: None,
            session: None,
        }
    }
}

fn run_worker(
    receiver: Receiver<WorkerCommand>,
    surface_receiver: Receiver<CameraXComponentEvent>,
    events: UnboundedSender<UiEvent>,
) {
    let (camera_sender, camera_receiver) = mpsc::channel();
    #[cfg(feature = "scan")]
    let mut scan_decoder = ScanDecoder::spawn(events.clone());
    let mut state = WorkerState::default();
    let _ = events.send(UiEvent::Status(CameraStatus::WaitingForSurface));

    loop {
        while let Ok(event) = surface_receiver.try_recv() {
            handle_surface_event(event, &mut state, &camera_sender, &events);
        }
        while let Ok(event) = camera_receiver.try_recv() {
            match event {
                #[cfg(feature = "scan")]
                CameraEvent::Frame(frame) => scan_decoder.submit(frame),
                #[cfg(not(feature = "scan"))]
                CameraEvent::Frame(_) => {}
                event => handle_camera_event(event, &state, &events),
            }
        }
        match receiver.recv_timeout(WORKER_POLL_INTERVAL) {
            Ok(WorkerCommand::Configure {
                active,
                position,
                profiles,
                #[cfg(feature = "scan")]
                scan,
            }) => {
                state.desired_active = active;
                state.desired_position = position;
                state.desired_profiles = profiles;
                #[cfg(feature = "scan")]
                {
                    state.desired_scan = scan.clone();
                    scan_decoder.configure(scan);
                }
                reconcile(&mut state, &camera_sender, &events);
            }
            Ok(WorkerCommand::Capture(options)) => capture(&state, options, &events),
            Ok(WorkerCommand::Control(command)) => control(&mut state, command, &events),
            Ok(WorkerCommand::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
    state.session.take();
}

fn handle_surface_event(
    event: CameraXComponentEvent,
    state: &mut WorkerState,
    camera_sender: &Sender<CameraEvent>,
    events: &UnboundedSender<UiEvent>,
) {
    match event {
        CameraXComponentEvent::Surface(surface) => {
            state.surface = Some(surface);
            reconcile(state, camera_sender, events);
        }
        CameraXComponentEvent::SurfaceLost => {
            state.surface = None;
            state.active_key = None;
            state.session.take();
            let _ = events.send(UiEvent::Status(CameraStatus::WaitingForSurface));
        }
    }
}

fn handle_camera_event(event: CameraEvent, state: &WorkerState, events: &UnboundedSender<UiEvent>) {
    match event {
        CameraEvent::Photo(photo) => {
            let _ = events.send(UiEvent::Photo(photo));
            if let Some(info) = state.session.as_ref().map(CameraSession::info) {
                let _ = events.send(UiEvent::Status(CameraStatus::Running(info)));
            }
        }
        CameraEvent::Frame(_) => {}
        CameraEvent::FocusState(focus) => {
            let _ = events.send(UiEvent::FocusState(focus));
        }
        CameraEvent::Error(error) => emit_error(events, error.into()),
    }
}

fn capture(
    state: &WorkerState,
    options: Option<CameraCaptureOptions>,
    events: &UnboundedSender<UiEvent>,
) {
    let Some(current) = state.session.as_ref() else {
        emit_error(
            events,
            CameraError::invalid_state(
                "CameraController::capture",
                "camera session is not running",
            ),
        );
        return;
    };
    let info = current.info();
    let _ = events.send(UiEvent::Status(CameraStatus::Capturing(info)));
    let result = options.map_or_else(
        || current.capture(),
        |options| current.capture_with_options(options),
    );
    if let Err(error) = result {
        emit_error(events, error.into());
        let _ = events.send(UiEvent::Status(CameraStatus::Running(info)));
    }
}

fn control(
    state: &mut WorkerState,
    command: CameraControlCommand,
    events: &UnboundedSender<UiEvent>,
) {
    let Some(session) = state.session.as_mut() else {
        emit_error(
            events,
            CameraError::invalid_state(
                "CameraController::control",
                "camera session is not running",
            ),
        );
        return;
    };
    let result = match command {
        CameraControlCommand::Flash(value) => session.set_flash_mode(value),
        CameraControlCommand::Torch(value) => session.set_torch_mode(value),
        CameraControlCommand::Zoom { ratio, smooth } => session.set_zoom(ratio, smooth),
        CameraControlCommand::ExposureMode(value) => session.set_exposure_mode(value),
        CameraControlCommand::ExposureBias(value) => session.set_exposure_bias(value),
        CameraControlCommand::MeteringPoint(value) => session.set_metering_point(value),
        CameraControlCommand::FocusMode(value) => session.set_focus_mode(value),
        CameraControlCommand::FocusPoint(value) => session.set_focus_point(value),
        CameraControlCommand::Stabilization(value) => session.set_stabilization_mode(value),
        CameraControlCommand::WhiteBalanceMode(value) => session.set_white_balance_mode(value),
        CameraControlCommand::WhiteBalanceTemperature(value) => {
            session.set_white_balance_temperature(value)
        }
        CameraControlCommand::Macro(value) => session.set_macro_enabled(value),
        CameraControlCommand::AutoDeviceSwitch(value) => {
            session.set_auto_device_switch_enabled(value)
        }
        CameraControlCommand::QualityPriority(value) => session.set_quality_priority(value),
        CameraControlCommand::FrameRate(value) => session.set_frame_rate(value),
        CameraControlCommand::PreviewRotation(value) => session.set_preview_rotation(value),
        CameraControlCommand::ColorSpace(value) => session.set_color_space(value),
    };
    match result {
        Ok(()) => {
            let _ = events.send(UiEvent::Controls(session.controls()));
        }
        Err(error) => emit_error(events, error.into()),
    }
}

fn reconcile(
    state: &mut WorkerState,
    camera_sender: &Sender<CameraEvent>,
    events: &UnboundedSender<UiEvent>,
) {
    if !state.desired_active {
        state.session.take();
        state.active_key = None;
        let _ = events.send(UiEvent::Status(CameraStatus::Stopped));
        return;
    }
    let Some(surface) = state.surface else {
        state.session.take();
        state.active_key = None;
        let _ = events.send(UiEvent::Status(CameraStatus::WaitingForSurface));
        return;
    };
    let next_key = SessionKey {
        surface,
        position: state.desired_position,
        profiles: state.desired_profiles,
        #[cfg(feature = "scan")]
        scan: state.desired_scan.clone(),
    };
    if state.active_key.as_ref() == Some(&next_key) && state.session.is_some() {
        return;
    }
    state.session.take();
    state.active_key = None;
    let _ = events.send(UiEvent::Status(CameraStatus::Starting(
        state.desired_position,
    )));
    if !ohos_ability_access_control_binding::check_self_permission(CAMERA_PERMISSION) {
        let error = CameraError::permission_denied();
        let _ = events.send(UiEvent::Status(CameraStatus::PermissionDenied));
        let _ = events.send(UiEvent::Error(error));
        return;
    }
    let configuration = CameraConfiguration {
        position: state.desired_position,
        surface,
        preview_size: state.desired_profiles.preview_size,
        enable_photo_output: {
            #[cfg(feature = "scan")]
            {
                state.desired_scan.is_none()
            }
            #[cfg(not(feature = "scan"))]
            {
                true
            }
        },
        photo_size: state.desired_profiles.photo_size,
        frame_output: {
            #[cfg(feature = "scan")]
            {
                state
                    .desired_scan
                    .as_ref()
                    .map(|scan| CameraFrameOutputConfiguration {
                        size: scan.frame_size,
                        capacity: 8,
                        max_frames_per_second: scan.max_frames_per_second,
                    })
            }
            #[cfg(not(feature = "scan"))]
            {
                None
            }
        },
    };
    match CameraSession::open(configuration, camera_sender.clone()) {
        Ok(next) => {
            let info = next.info();
            let _ = events.send(UiEvent::Capabilities(next.capabilities()));
            let _ = events.send(UiEvent::Controls(next.controls()));
            state.session = Some(next);
            state.active_key = Some(next_key);
            let _ = events.send(UiEvent::Status(CameraStatus::Running(info)));
        }
        Err(error) => {
            let error: CameraError = error.into();
            let status = match error.kind() {
                crate::CameraErrorKind::PermissionDenied => CameraStatus::PermissionDenied,
                crate::CameraErrorKind::NoCamera => CameraStatus::Unavailable,
                _ => CameraStatus::Error(error.clone()),
            };
            let _ = events.send(UiEvent::Status(status));
            let _ = events.send(UiEvent::Error(error));
        }
    }
}

#[cfg(feature = "scan")]
struct ScanDecoder {
    sender: Option<SyncSender<CameraFrame>>,
    configuration: Arc<Mutex<Option<CameraScanConfiguration>>>,
    thread: Option<JoinHandle<()>>,
}

#[cfg(feature = "scan")]
impl ScanDecoder {
    fn spawn(events: UnboundedSender<UiEvent>) -> Self {
        let (sender, receiver) = mpsc::sync_channel(1);
        let configuration = Arc::new(Mutex::new(None));
        let thread_configuration = configuration.clone();
        let thread = thread::Builder::new()
            .name("arkit-camera-scan".into())
            .spawn(move || decode_frames(receiver, thread_configuration, events))
            .ok();
        Self {
            sender: thread.as_ref().map(|_| sender),
            configuration,
            thread,
        }
    }

    fn configure(&mut self, configuration: Option<CameraScanConfiguration>) {
        *self
            .configuration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = configuration;
    }

    fn submit(&self, frame: CameraFrame) {
        if let Some(sender) = self.sender.as_ref() {
            let _ = sender.try_send(frame);
        }
    }
}

#[cfg(feature = "scan")]
impl Drop for ScanDecoder {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(feature = "scan")]
fn decode_frames(
    receiver: Receiver<CameraFrame>,
    configuration: Arc<Mutex<Option<CameraScanConfiguration>>>,
    events: UnboundedSender<UiEvent>,
) {
    let mut active_configuration = None;
    let mut single_scan_completed = false;
    let mut last_result: Option<(CameraScanResult, std::time::Instant)> = None;
    while let Ok(frame) = receiver.recv() {
        let next_configuration = configuration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if next_configuration != active_configuration {
            active_configuration = next_configuration.clone();
            single_scan_completed = false;
            last_result = None;
        }
        let Some(scan) = next_configuration else {
            continue;
        };
        if single_scan_completed {
            continue;
        }
        let Some(result) = crate::scan::decode_frame(frame, &scan) else {
            continue;
        };
        let now = std::time::Instant::now();
        let duplicate = last_result.as_ref().is_some_and(|(previous, at)| {
            previous.text == result.text
                && previous.format == result.format
                && now.duration_since(*at) < scan.duplicate_timeout
        });
        if duplicate {
            continue;
        }
        last_result = Some((result.clone(), now));
        let _ = events.send(UiEvent::Scan(result));
        single_scan_completed = !scan.continuous;
    }
}

fn emit_error(events: &UnboundedSender<UiEvent>, error: CameraError) {
    let _ = events.send(UiEvent::Status(CameraStatus::Error(error.clone())));
    let _ = events.send(UiEvent::Error(error));
}
