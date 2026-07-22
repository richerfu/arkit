use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use arkit_hooks::{use_app_foreground, use_ark_node};
use arkit_prelude::*;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::network::LottieSourceLoader;
use crate::surface::SurfaceRegistration;
use crate::worker::{PlayerConfiguration, UiEvent, WorkerHandle, WorkerMessage};
use crate::{
    LottieAlignment, LottieComposition, LottieController, LottieError, LottieFit, LottieFrame,
    LottieRepeatMode, LottieSource, LottieStatus,
};

struct ComponentRuntime {
    worker: RefCell<Option<WorkerHandle>>,
    receiver: RefCell<Option<UnboundedReceiver<UiEvent>>>,
    events: UnboundedSender<UiEvent>,
}

struct DownloadTask {
    ui_task: arkit_prelude::dioxus_core::Task,
    network_task: tokio::task::AbortHandle,
}

impl Drop for DownloadTask {
    fn drop(&mut self) {
        self.network_task.abort();
        self.ui_task.cancel();
    }
}

impl ComponentRuntime {
    fn new() -> Self {
        let (events, receiver) = mpsc::unbounded_channel();
        let worker = match WorkerHandle::spawn(events.clone()) {
            Ok(worker) => Some(worker),
            Err(error) => {
                let _ = events.send(UiEvent::Status(LottieStatus::Error(error.clone())));
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

    fn sender(&self) -> Option<std::sync::mpsc::Sender<WorkerMessage>> {
        self.worker.borrow().as_ref().map(WorkerHandle::sender)
    }

    fn send(&self, message: WorkerMessage) {
        if let Some(worker) = self.worker.borrow().as_ref() {
            if let Err(error) = worker.send(message) {
                self.emit_error(error);
            }
        }
    }

    fn take_receiver(&self) -> Option<UnboundedReceiver<UiEvent>> {
        self.receiver.borrow_mut().take()
    }

    fn emit_error(&self, error: LottieError) {
        let _ = self
            .events
            .send(UiEvent::Status(LottieStatus::Error(error.clone())));
        let _ = self.events.send(UiEvent::Error(error));
    }

    fn shutdown(&self) {
        self.worker.borrow_mut().take();
    }
}

/// Properties for a native, worker-driven Lottie surface.
#[derive(Props, Clone, PartialEq)]
pub struct LottiePlayerProps {
    pub source: LottieSource,
    #[props(default)]
    pub controller: Option<LottieController>,
    /// Business-level activity gate. Application foreground and component
    /// visibility are applied automatically in addition to this value.
    #[props(default = true)]
    pub active: bool,
    #[props(default = true)]
    pub playing: bool,
    #[props(default)]
    pub repeat: LottieRepeatMode,
    /// Signed, finite, non-zero playback multiplier, clamped to `-16.0..=16.0`.
    #[props(default = 1.0)]
    pub speed: f32,
    #[props(default)]
    pub fit: LottieFit,
    #[props(default)]
    pub alignment: LottieAlignment,
    /// ThorVG effect quality in `0..=100`. Default `50` balances blur/shadow
    /// quality and frame cost.
    #[props(default = 50)]
    pub quality: u8,
    /// Upper bound for native frame production, clamped to `1..=120`.
    #[props(default = 60)]
    pub max_frames_per_second: u16,
    #[props(default)]
    /// CSS width (`"100%"`, `"240"`). Defaults to `"100%"`.
    #[props(default = "100%".to_string())]
    pub width: String,
    /// CSS height (`"240"`, `"100%"`). Defaults to `"240"` when unset.
    #[props(default)]
    pub height: Option<String>,
    #[props(default = 0x00000000)]
    pub background_color: u32,
    #[props(default)]
    pub on_status_change: Option<EventHandler<LottieStatus>>,
    #[props(default)]
    pub on_composition: Option<EventHandler<LottieComposition>>,
    /// Coalesced to at most ten events per second; rendering itself never
    /// crosses the Dioxus UI loop.
    #[props(default)]
    pub on_frame: Option<EventHandler<LottieFrame>>,
    #[props(default)]
    pub on_complete: Option<EventHandler<()>>,
    #[props(default)]
    pub on_error: Option<EventHandler<LottieError>>,
}

/// Render Lottie JSON directly into an ArkUI XComponent native window.
#[component]
pub fn LottiePlayer(props: LottiePlayerProps) -> Element {
    let runtime = use_hook(|| Rc::new(ComponentRuntime::new()));
    let node_ref = use_ark_node();
    let app_foreground = use_app_foreground();
    // XComponent presentation is owned by its native surface lifecycle. ArkUI
    // reports a transient 0% visible area for a mounted Surface node even while
    // its native window is on screen, so the generic node visibility hook must
    // not suspend this worker. SurfaceAvailable/SurfaceLost provide the exact
    // component-level gate; the application lifecycle remains independent.
    let effective_active = props.active && app_foreground;
    let surface_registration = use_hook(|| Rc::new(RefCell::new(None::<SurfaceRegistration>)));
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<usize>)));
    let controller_binding = use_hook(|| Rc::new(RefCell::new(None::<(LottieController, u64)>)));
    let source_loader = use_hook(|| {
        Rc::new(RefCell::new(
            None::<crate::LottieResult<LottieSourceLoader>>,
        ))
    });
    let download_task = use_hook(|| Rc::new(RefCell::new(None::<DownloadTask>)));

    let status_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<LottieStatus>>)));
    let composition_handler =
        use_hook(|| Rc::new(Cell::new(None::<EventHandler<LottieComposition>>)));
    let frame_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<LottieFrame>>)));
    let complete_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<()>>)));
    let error_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<LottieError>>)));
    status_handler.set(props.on_status_change);
    composition_handler.set(props.on_composition);
    frame_handler.set(props.on_frame);
    complete_handler.set(props.on_complete);
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
        if let (Some(controller), Some(sender)) = (props.controller.clone(), runtime.sender()) {
            let binding = controller.bind(sender);
            controller_binding
                .borrow_mut()
                .replace((controller, binding));
        }
    }

    let receiver_slot = use_hook(|| Rc::new(RefCell::new(runtime.take_receiver())));
    let events_controller = controller_binding.clone();
    let events_status = status_handler.clone();
    let events_composition = composition_handler.clone();
    let events_frame = frame_handler.clone();
    let events_complete = complete_handler.clone();
    let events_error = error_handler.clone();
    let _event_task = use_future(move || {
        let receiver = receiver_slot.borrow_mut().take();
        let events_controller = events_controller.clone();
        let events_status = events_status.clone();
        let events_composition = events_composition.clone();
        let events_frame = events_frame.clone();
        let events_complete = events_complete.clone();
        let events_error = events_error.clone();
        async move {
            let Some(mut receiver) = receiver else {
                return;
            };
            while let Some(event) = receiver.recv().await {
                match event {
                    UiEvent::Status(status) => {
                        if let Some((controller, binding)) = events_controller
                            .borrow()
                            .as_ref()
                            .map(|(controller, binding)| (controller.clone(), *binding))
                        {
                            controller.update_status(binding, status.clone());
                        }
                        if let Some(handler) = events_status.get() {
                            handler.call(status);
                        }
                    }
                    UiEvent::Composition(composition) => {
                        if let Some((controller, binding)) = events_controller
                            .borrow()
                            .as_ref()
                            .map(|(controller, binding)| (controller.clone(), *binding))
                        {
                            controller.update_composition(binding, composition);
                        }
                        if let Some(handler) = events_composition.get() {
                            handler.call(composition);
                        }
                    }
                    UiEvent::Frame(frame) => {
                        if let Some((controller, binding)) = events_controller
                            .borrow()
                            .as_ref()
                            .map(|(controller, binding)| (controller.clone(), *binding))
                        {
                            controller.update_frame(binding, frame);
                        }
                        if let Some(handler) = events_frame.get() {
                            handler.call(frame);
                        }
                    }
                    UiEvent::Completed => {
                        if let Some(handler) = events_complete.get() {
                            handler.call(());
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

    let configuration = PlayerConfiguration {
        source: props.source.clone(),
        active: effective_active,
        playing: props.playing,
        repeat: props.repeat,
        speed: props.speed,
        fit: props.fit,
        alignment: props.alignment,
        quality: props.quality,
        max_frames_per_second: props.max_frames_per_second,
    };
    let configure_runtime = runtime.clone();
    use_effect(use_reactive((&configuration,), move |(configuration,)| {
        configure_runtime.send(WorkerMessage::Configure(configuration));
    }));

    let download_runtime = runtime.clone();
    let download_loader = source_loader.clone();
    let active_download = download_task.clone();
    let async_runtime = arkit_runtime::tokio_handle();
    use_effect(use_reactive((&props.source,), move |(source,)| {
        active_download.borrow_mut().take();
        let Some(network_source) = source.network_source().cloned() else {
            return;
        };
        let source_key: Arc<str> = Arc::from(source.key());
        let loader = download_loader
            .borrow_mut()
            .get_or_insert_with(LottieSourceLoader::new)
            .clone();
        let loader = match loader {
            Ok(loader) => loader,
            Err(error) => {
                download_runtime.send(WorkerMessage::SourceLoaded {
                    key: source_key,
                    result: Err(error),
                });
                return;
            }
        };
        let network_future = async_runtime.spawn(async move { loader.load(network_source).await });
        let network_task = network_future.abort_handle();
        let event_runtime = download_runtime.clone();
        let ui_task = arkit_prelude::dioxus_core::spawn(async move {
            let result = network_future.await.unwrap_or_else(|error| {
                Err(LottieError::network(
                    "LottieSourceLoader::task",
                    if error.is_cancelled() {
                        "the Lottie download task was cancelled"
                    } else {
                        "the Lottie download task stopped unexpectedly"
                    },
                ))
            });
            event_runtime.send(WorkerMessage::SourceLoaded {
                key: source_key,
                result,
            });
        });
        active_download.borrow_mut().replace(DownloadTask {
            ui_task,
            network_task,
        });
    }));

    let effect_registration = surface_registration.clone();
    let effect_registered_node = registered_node.clone();
    let effect_runtime = runtime.clone();
    let frame_rate = props.max_frames_per_second;
    use_effect(move || {
        let Some(node) = node_ref.get() else {
            return;
        };
        let native_key = node.borrow().raw_handle() as usize;
        if effect_registered_node.get() == Some(native_key) {
            if let Some(registration) = effect_registration.borrow().as_ref() {
                if let Err(error) = registration.set_frame_rate(frame_rate) {
                    effect_runtime.emit_error(error);
                }
            }
            return;
        }
        effect_registration.borrow_mut().take();
        let Some(sender) = effect_runtime.sender() else {
            return;
        };
        let tick_pending = effect_runtime
            .worker
            .borrow()
            .as_ref()
            .map(WorkerHandle::tick_pending);
        let Some(tick_pending) = tick_pending else {
            return;
        };
        let attachment = {
            let node = node.borrow();
            SurfaceRegistration::attach(&node, sender, tick_pending, frame_rate)
        };
        match attachment {
            Ok(registration) => {
                effect_registration.borrow_mut().replace(registration);
                effect_registered_node.set(Some(native_key));
            }
            Err(error) => effect_runtime.emit_error(error),
        }
    });

    let rate_registration = surface_registration.clone();
    let rate_runtime = runtime.clone();
    use_effect(use_reactive(
        (&props.max_frames_per_second,),
        move |(frames_per_second,)| {
            if let Some(registration) = rate_registration.borrow().as_ref() {
                if let Err(error) = registration.set_frame_rate(frames_per_second) {
                    rate_runtime.emit_error(error);
                }
            }
        },
    ));

    let drop_registration = surface_registration.clone();
    let drop_binding = controller_binding.clone();
    let drop_runtime = runtime.clone();
    let drop_download = download_task.clone();
    use_drop(move || {
        drop_download.borrow_mut().take();
        drop_registration.borrow_mut().take();
        if let Some((controller, binding)) = drop_binding.borrow_mut().take() {
            controller.unbind(binding);
        }
        drop_runtime.shutdown();
    });

    let height = props.height.clone().unwrap_or_else(|| "240".into());
    rsx! {
        xcomponent {
            width: props.width.clone(),
            height: height,
            background_color: props.background_color,
        }
    }
}
