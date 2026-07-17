use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arkit_hooks::{use_app_foreground, use_ark_node};
use arkit_prelude::*;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

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
    #[props(default)]
    pub width: Option<f32>,
    #[props(default)]
    pub height: Option<f32>,
    #[props(default = 1.0)]
    pub percent_width: f32,
    #[props(default)]
    pub percent_height: Option<f32>,
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
    let node_ref = use_ark_node();
    let app_foreground = use_app_foreground();
    // CameraPreview is a Surface XComponent. Its created/destroyed callbacks
    // are the authoritative component lifecycle; ArkUI's visible-area event
    // can transiently report 0% for an on-screen Surface node.
    let effective_active = props.active && app_foreground;
    let surface_registration = use_hook(|| Rc::new(RefCell::new(None::<SurfaceRegistration>)));
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<usize>)));
    let controller_binding = use_hook(|| Rc::new(RefCell::new(None::<(CameraController, u64)>)));
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

    let controller_changed = {
        let binding = controller_binding.borrow();
        match (binding.as_ref(), props.controller.as_ref()) {
            (Some((current, _)), Some(next)) => current != next,
            (None, None) => false,
            _ => true,
        }
    };
    if controller_changed {
        if let Some((controller, binding)) = controller_binding.borrow_mut().take() {
            controller.unbind(binding);
        }
        if let (Some(controller), Some(sender)) =
            (props.controller.clone(), runtime.command_sender())
        {
            let binding = controller.bind(sender);
            controller_binding
                .borrow_mut()
                .replace((controller, binding));
        }
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
                            .as_ref()
                            .map(|(controller, binding)| (controller.clone(), *binding));
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
                            .as_ref()
                            .map(|(controller, binding)| (controller.clone(), *binding));
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
                            .as_ref()
                            .map(|(controller, binding)| (controller.clone(), *binding));
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
    use_effect(move || {
        let Some(node) = node_ref.get() else {
            return;
        };
        let native_key = node.borrow().raw_handle() as usize;
        if effect_registered_node.get() == Some(native_key) {
            return;
        }
        effect_registration.borrow_mut().take();
        let Some(sender) = effect_runtime.surface_sender() else {
            return;
        };
        let attachment = {
            let node = node.borrow();
            SurfaceRegistration::attach(&node, sender)
        };
        match attachment {
            Ok(registration) => {
                effect_registration.borrow_mut().replace(registration);
                effect_registered_node.set(Some(native_key));
            }
            Err(error) => effect_runtime.emit_error(error),
        };
    });

    let drop_registration = surface_registration.clone();
    let drop_binding = controller_binding.clone();
    let drop_runtime = runtime.clone();
    use_drop(move || {
        drop_registration.borrow_mut().take();
        if let Some((controller, binding)) = drop_binding.borrow_mut().take() {
            controller.unbind(binding);
        }
        drop_runtime.shutdown();
    });

    let default_height = if props.height.is_none() && props.percent_height.is_none() {
        Some(360.0)
    } else {
        props.height
    };
    rsx! {
        xcomponent {
            percent_width: props.percent_width,
            width: props.width,
            height: default_height,
            percent_height: props.percent_height,
            background_color: 0xFF000000u32,
        }
    }
}
