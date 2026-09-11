use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arkit_hooks::{use_app_foreground, use_mounted_node, use_native_element_ref};
use arkit_prelude::*;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::controller::CameraControllerLease;
use crate::native::UiEvent;
use crate::surface::SurfaceRegistration;
use crate::worker::{WorkerCommand, WorkerHandle};
use crate::{
    CameraCapabilities, CameraController, CameraControls, CameraError, CameraFocusState,
    CameraPosition, CameraProfileSelection, CameraStatus, CapturedPhoto,
};
#[cfg(feature = "scan")]
use crate::{CameraScanConfiguration, CameraScanResult};

struct ComponentRuntime {
    worker: RefCell<Option<WorkerHandle>>,
    receiver: RefCell<Option<UnboundedReceiver<UiEvent>>>,
    events: UnboundedSender<UiEvent>,
}

#[derive(Default)]
struct ControllerBindingSlot {
    attempted: Option<CameraController>,
    lease: Option<CameraControllerLease>,
}

impl ControllerBindingSlot {
    fn reconcile(
        &mut self,
        controller: Option<CameraController>,
        sender: Option<std::sync::mpsc::Sender<WorkerCommand>>,
    ) -> Option<CameraError> {
        if self.attempted.as_ref() == controller.as_ref() {
            return None;
        }

        self.lease.take();
        self.attempted = controller.clone();
        let (Some(controller), Some(sender)) = (controller, sender) else {
            return None;
        };
        match controller.bind(sender) {
            Ok(lease) => {
                self.lease = Some(lease);
                None
            }
            Err(error) => Some(error),
        }
    }

    fn active(&self) -> Option<&CameraControllerLease> {
        self.lease.as_ref()
    }

    fn clear(&mut self) {
        self.lease.take();
        self.attempted = None;
    }
}

impl ComponentRuntime {
    fn new() -> Self {
        let (events, receiver) = mpsc::unbounded_channel();
        let worker = match WorkerHandle::spawn(events.clone()) {
            Ok(worker) => Some(worker),
            Err(error) => {
                let _ = events.send(UiEvent::Status(CameraStatus::Error(error.clone())));
                let _ = events.send(UiEvent::Error(error));
                None
            }
        };
        Self {
            worker: RefCell::new(worker),
            receiver: RefCell::new(Some(receiver)),
            events,
        }
    }

    fn command_sender(&self) -> Option<std::sync::mpsc::Sender<WorkerCommand>> {
        self.worker.borrow().as_ref().map(WorkerHandle::sender)
    }

    fn surface_sender(
        &self,
    ) -> Option<std::sync::mpsc::Sender<ohos_camera_binding::CameraXComponentEvent>> {
        self.worker
            .borrow()
            .as_ref()
            .map(WorkerHandle::surface_sender)
    }

    fn send(&self, command: WorkerCommand) {
        if let Some(worker) = self.worker.borrow().as_ref() {
            if let Err(error) = worker.send(command) {
                self.emit_error(error);
            }
        }
    }

    fn take_receiver(&self) -> Option<UnboundedReceiver<UiEvent>> {
        self.receiver.borrow_mut().take()
    }

    fn emit_error(&self, error: CameraError) {
        let _ = self
            .events
            .send(UiEvent::Status(CameraStatus::Error(error.clone())));
        let _ = self.events.send(UiEvent::Error(error));
    }

    fn shutdown(&self) {
        self.worker.borrow_mut().take();
    }
}

/// Properties for the native CameraKit preview surface.
#[derive(Props, Clone, PartialEq)]
pub struct CameraPreviewProps {
    #[props(default)]
    pub controller: Option<CameraController>,
    #[props(default)]
    pub position: CameraPosition,
    #[props(default = true)]
    pub active: bool,
    #[props(default)]
    pub profiles: CameraProfileSelection,
    #[cfg(feature = "scan")]
    #[props(default)]
    pub scan: Option<CameraScanConfiguration>,
    /// CSS width (`"100%"`, `"320"`). Defaults to `"100%"`.
    #[props(default = "100%".to_string())]
    pub width: String,
    /// CSS height (`"360"`, `"100%"`). Defaults to `"360"` when unset.
    #[props(default)]
    pub height: Option<String>,
    #[props(default)]
    pub on_status_change: Option<EventHandler<CameraStatus>>,
    #[props(default)]
    pub on_capabilities_change: Option<EventHandler<CameraCapabilities>>,
    #[props(default)]
    pub on_controls_change: Option<EventHandler<CameraControls>>,
    #[props(default)]
    pub on_focus_state_change: Option<EventHandler<CameraFocusState>>,
    #[props(default)]
    pub on_photo: Option<EventHandler<CapturedPhoto>>,
    #[cfg(feature = "scan")]
    #[props(default)]
    pub on_scan: Option<EventHandler<CameraScanResult>>,
    #[props(default)]
    pub on_error: Option<EventHandler<CameraError>>,
}

/// Mount an ArkUI XComponent surface backed by a native CameraKit session.
#[component]
pub fn CameraPreview(props: CameraPreviewProps) -> Element {
    let runtime = use_hook(|| Rc::new(ComponentRuntime::new()));
    let node_ref = use_native_element_ref();
    let app_foreground = use_app_foreground();
    // CameraPreview is a Surface XComponent. Its created/destroyed callbacks
    // are the authoritative component lifecycle; ArkUI's visible-area event
    // can transiently report 0% for an on-screen Surface node.
    let effective_active = props.active && app_foreground;
    let surface_registration = use_hook(|| Rc::new(RefCell::new(None::<SurfaceRegistration>)));
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<u64>)));
    let controller_binding = use_hook(|| Rc::new(RefCell::new(ControllerBindingSlot::default())));
    let status_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<CameraStatus>>)));
    let capabilities_handler =
        use_hook(|| Rc::new(Cell::new(None::<EventHandler<CameraCapabilities>>)));
    let controls_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<CameraControls>>)));
    let focus_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<CameraFocusState>>)));
    let photo_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<CapturedPhoto>>)));
    #[cfg(feature = "scan")]
    let scan_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<CameraScanResult>>)));
    let error_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<CameraError>>)));
    status_handler.set(props.on_status_change);
    capabilities_handler.set(props.on_capabilities_change);
    controls_handler.set(props.on_controls_change);
    focus_handler.set(props.on_focus_state_change);
    photo_handler.set(props.on_photo);
    #[cfg(feature = "scan")]
    scan_handler.set(props.on_scan);
    error_handler.set(props.on_error);

    if let Some(error) = controller_binding
        .borrow_mut()
        .reconcile(props.controller.clone(), runtime.command_sender())
    {
        runtime.emit_error(error);
    }

    let receiver_slot = use_hook(|| Rc::new(RefCell::new(runtime.take_receiver())));
    let events_controller = controller_binding.clone();
    let events_status = status_handler.clone();
    let events_capabilities = capabilities_handler.clone();
    let events_controls = controls_handler.clone();
    let events_focus = focus_handler.clone();
    let events_photo = photo_handler.clone();
    #[cfg(feature = "scan")]
    let events_scan = scan_handler.clone();
    let events_error = error_handler.clone();
    // ComponentRuntime and its worker are created once by `use_hook` and are
    // never replaced before `use_drop`. Worker events therefore belong to this
    // component/native surface, not to a controller lease: after a controller
    // prop swap, queued events intentionally update the current controller.
    let _event_task = use_future(move || {
        let receiver = receiver_slot.borrow_mut().take();
        let events_controller = events_controller.clone();
        let events_status = events_status.clone();
        let events_capabilities = events_capabilities.clone();
        let events_controls = events_controls.clone();
        let events_focus = events_focus.clone();
        let events_photo = events_photo.clone();
        #[cfg(feature = "scan")]
        let events_scan = events_scan.clone();
        let events_error = events_error.clone();
        async move {
            let Some(mut receiver) = receiver else {
                return;
            };
            while let Some(event) = receiver.recv().await {
                match event {
                    UiEvent::Status(status) => {
                        let controller = events_controller
                            .borrow()
                            .active()
                            .map(|lease| (lease.controller().clone(), lease.binding()));
                        if let Some((controller, binding)) = controller {
                            controller.update_status(binding, status.clone());
                        }
                        if let Some(handler) = events_status.get() {
                            handler.call(status);
                        }
                    }
                    UiEvent::Capabilities(capabilities) => {
                        let controller = events_controller
                            .borrow()
                            .active()
                            .map(|lease| (lease.controller().clone(), lease.binding()));
                        if let Some((controller, binding)) = controller {
                            controller.update_capabilities(binding, capabilities.clone());
                        }
                        if let Some(handler) = events_capabilities.get() {
                            handler.call(capabilities);
                        }
                    }
                    UiEvent::Controls(controls) => {
                        let controller = events_controller
                            .borrow()
                            .active()
                            .map(|lease| (lease.controller().clone(), lease.binding()));
                        if let Some((controller, binding)) = controller {
                            controller.update_controls(binding, controls.clone());
                        }
                        if let Some(handler) = events_controls.get() {
                            handler.call(controls);
                        }
                    }
                    UiEvent::FocusState(focus) => {
                        if let Some(handler) = events_focus.get() {
                            handler.call(focus);
                        }
                    }
                    UiEvent::Photo(photo) => {
                        if let Some(handler) = events_photo.get() {
                            handler.call(photo);
                        }
                    }
                    #[cfg(feature = "scan")]
                    UiEvent::Scan(result) => {
                        if let Some(handler) = events_scan.get() {
                            handler.call(result);
                        }
                    }
                    UiEvent::Error(error) => {
                        if let Some(handler) = events_error.get() {
                            handler.call(error);
                        }
                    }
                }
            }
        }
    });

    #[cfg(not(feature = "scan"))]
    let configure_runtime = runtime.clone();
    #[cfg(not(feature = "scan"))]
    use_effect(use_reactive(
        (&effective_active, &props.position, &props.profiles),
        move |(active, position, profiles)| {
            configure_runtime.send(WorkerCommand::Configure {
                active,
                position,
                profiles,
            });
        },
    ));
    #[cfg(feature = "scan")]
    let configure_runtime = runtime.clone();
    #[cfg(feature = "scan")]
    use_effect(use_reactive(
        (
            &effective_active,
            &props.position,
            &props.profiles,
            &props.scan,
        ),
        move |(active, position, profiles, scan)| {
            configure_runtime.send(WorkerCommand::Configure {
                active,
                position,
                profiles,
                scan,
            });
        },
    ));

    let effect_registration = surface_registration.clone();
    let effect_registered_node = registered_node.clone();
    let effect_runtime = runtime.clone();
    use_mounted_node(node_ref.clone(), move |node| {
        let Some(node) = node else {
            effect_registration.borrow_mut().take();
            effect_registered_node.set(None);
            return;
        };
        let native_key = node.epoch();
        if effect_registered_node.get() == Some(native_key) {
            return;
        }
        effect_registration.borrow_mut().take();
        let Some(sender) = effect_runtime.surface_sender() else {
            return;
        };
        let attachment = SurfaceRegistration::attach(&node, sender);
        match attachment {
            Ok(registration) => {
                effect_registration.borrow_mut().replace(registration);
                effect_registered_node.set(Some(native_key));
                let teardown_registration = effect_registration.clone();
                let teardown_registered_node = effect_registered_node.clone();
                // SAFETY: this closure only unregisters the XComponent
                // attachment while its native node is still valid.
                let installed = unsafe {
                    node.install_native_teardown(move || {
                        teardown_registration.borrow_mut().take();
                        teardown_registered_node.set(None);
                    })
                };
                if !installed {
                    effect_registration.borrow_mut().take();
                    effect_registered_node.set(None);
                }
            }
            Err(error) => effect_runtime.emit_error(error),
        };
    });

    let drop_registration = surface_registration.clone();
    let drop_binding = controller_binding.clone();
    let drop_runtime = runtime.clone();
    use_drop(move || {
        drop_registration.borrow_mut().take();
        drop_binding.borrow_mut().clear();
        drop_runtime.shutdown();
    });

    let height = props.height.clone().unwrap_or_else(|| "360".into());
    rsx! {
        xcomponent {
            native_ref: node_ref,
            width: props.width.clone(),
            height: height,
            background_color: "#FF000000",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CameraErrorKind;

    #[test]
    fn duplicate_binding_attempt_is_reported_once_and_controller_change_recovers() {
        let occupied = CameraController::new();
        let (occupied_sender, _) = std::sync::mpsc::channel();
        let occupied_lease = occupied.bind(occupied_sender).unwrap();
        let (component_sender, _) = std::sync::mpsc::channel();
        let mut slot = ControllerBindingSlot::default();

        let first = slot.reconcile(Some(occupied.clone()), Some(component_sender.clone()));
        let rerender = slot.reconcile(Some(occupied.clone()), Some(component_sender.clone()));
        assert_eq!(first.unwrap().kind(), CameraErrorKind::AlreadyBound);
        assert!(
            rerender.is_none(),
            "the same failed attempt must not repeat"
        );
        assert!(slot.active().is_none());

        let replacement = CameraController::new();
        assert!(slot
            .reconcile(Some(replacement.clone()), Some(component_sender))
            .is_none());
        assert!(slot.active().is_some());
        assert!(replacement.is_mounted());
        assert!(occupied.is_mounted());

        drop(occupied_lease);
    }
}
