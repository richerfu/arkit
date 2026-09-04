use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use arkit_hooks::{
    use_app_foreground, use_mounted_node, use_native_element_ref, use_safe_area, OverlayLayer,
    Portal,
};
use arkit_prelude::*;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::surface::SurfaceRegistration;
use crate::worker::{PlayerConfiguration, UiEvent, WorkerHandle, WorkerMessage};
use crate::{
    controls::BuiltInVideoControls, VideoBuffering, VideoController, VideoControls, VideoError,
    VideoMetadata, VideoProgress, VideoResizeMode, VideoSnapshot, VideoSource, VideoStatus,
    VideoSubtitleCue, VideoSubtitleSource, VideoTrack,
};

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
                let _ = events.send(UiEvent::Status(VideoStatus::Error(error.clone())));
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

    fn emit_error(&self, error: VideoError) {
        let _ = self
            .events
            .send(UiEvent::Status(VideoStatus::Error(error.clone())));
        let _ = self.events.send(UiEvent::Error(error));
    }

    fn shutdown(&self) {
        self.worker.borrow_mut().take();
    }
}

/// Properties for a native OpenHarmony video surface.
#[derive(Props, Clone, PartialEq)]
pub struct VideoPlayerProps {
    pub source: VideoSource,
    #[props(default)]
    pub controller: Option<VideoController>,
    /// Business-level activity gate. Application foreground is additionally
    /// applied unless `play_in_background` is enabled.
    #[props(default = true)]
    pub active: bool,
    #[props(default)]
    pub autoplay: bool,
    #[props(default)]
    pub play_in_background: bool,
    #[props(default)]
    pub looping: bool,
    #[props(default)]
    pub muted: bool,
    #[props(default = 1.0)]
    pub volume: f32,
    /// Arbitrary AVPlayer rate in `0.125..=4.0`.
    #[props(default = 1.0)]
    pub playback_rate: f32,
    #[props(default)]
    pub initial_position: Duration,
    #[props(default)]
    pub resize_mode: VideoResizeMode,
    /// Progress callback cadence, clamped to 50 ms..=10 s.
    #[props(default = Duration::from_millis(250))]
    pub progress_interval: Duration,
    #[props(default)]
    pub subtitles: Vec<VideoSubtitleSource>,
    #[props(default = "100%".to_string())]
    pub width: String,
    #[props(default)]
    pub height: Option<String>,
    #[props(default = 0xFF000000)]
    pub background_color: u32,
    /// Optional standard control overlay. Leave as `None` when supplying a
    /// completely custom controller UI.
    #[props(default)]
    pub controls: Option<VideoControls>,
    #[props(default)]
    pub on_load_start: Option<EventHandler<()>>,
    #[props(default)]
    pub on_load: Option<EventHandler<VideoMetadata>>,
    #[props(default)]
    pub on_status_change: Option<EventHandler<VideoStatus>>,
    #[props(default)]
    pub on_progress: Option<EventHandler<VideoProgress>>,
    #[props(default)]
    pub on_buffer: Option<EventHandler<VideoBuffering>>,
    #[props(default)]
    pub on_seek: Option<EventHandler<Duration>>,
    #[props(default)]
    pub on_playback_rate_change: Option<EventHandler<f32>>,
    #[props(default)]
    pub on_volume_change: Option<EventHandler<f32>>,
    #[props(default)]
    pub on_bitrate_change: Option<EventHandler<u32>>,
    #[props(default)]
    pub on_available_bitrates: Option<EventHandler<Vec<u32>>>,
    #[props(default)]
    pub on_ready_for_display: Option<EventHandler<()>>,
    #[props(default)]
    pub on_tracks_change: Option<EventHandler<Vec<VideoTrack>>>,
    #[props(default)]
    pub on_subtitle: Option<EventHandler<VideoSubtitleCue>>,
    #[props(default)]
    pub on_audio_interrupted: Option<EventHandler<()>>,
    #[props(default)]
    pub on_fullscreen_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_end: Option<EventHandler<()>>,
    #[props(default)]
    pub on_error: Option<EventHandler<VideoError>>,
}

/// Present a URL or file-descriptor video through an ArkUI XComponent.
#[component]
pub fn VideoPlayer(props: VideoPlayerProps) -> Element {
    let runtime = use_hook(|| Rc::new(ComponentRuntime::new()));
    let internal_controller = use_hook(VideoController::new);
    let active_controller = props
        .controller
        .clone()
        .unwrap_or_else(|| internal_controller.clone());
    let node_ref = use_native_element_ref();
    let app_foreground = use_app_foreground();
    let safe_area = use_safe_area();
    let effective_active = props.active && (props.play_in_background || app_foreground);
    let surface_registration = use_hook(|| Rc::new(RefCell::new(None::<SurfaceRegistration>)));
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<u64>)));
    let controller_binding = use_hook(|| Rc::new(RefCell::new(None::<(VideoController, u64)>)));
    let mut view_snapshot = use_signal(VideoSnapshot::default);
    let mut fullscreen = use_signal(|| false);
    let mut controls_visible = use_signal(|| true);
    let mut last_controls_interaction = use_signal(Instant::now);
    let auto_hide = use_hook(|| Rc::new(Cell::new(None::<Duration>)));
    auto_hide.set(
        props
            .controls
            .as_ref()
            .and_then(|controls| controls.auto_hide)
            .filter(|delay| !delay.is_zero()),
    );

    let load_start_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<()>>)));
    let load_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<VideoMetadata>>)));
    let status_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<VideoStatus>>)));
    let progress_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<VideoProgress>>)));
    let buffer_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<VideoBuffering>>)));
    let seek_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<Duration>>)));
    let rate_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<f32>>)));
    let volume_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<f32>>)));
    let bitrate_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<u32>>)));
    let bitrates_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<Vec<u32>>>)));
    let ready_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<()>>)));
    let tracks_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<Vec<VideoTrack>>>)));
    let subtitle_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<VideoSubtitleCue>>)));
    let interruption_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<()>>)));
    let fullscreen_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<bool>>)));
    let end_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<()>>)));
    let error_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<VideoError>>)));
    load_start_handler.set(props.on_load_start);
    load_handler.set(props.on_load);
    status_handler.set(props.on_status_change);
    progress_handler.set(props.on_progress);
    buffer_handler.set(props.on_buffer);
    seek_handler.set(props.on_seek);
    rate_handler.set(props.on_playback_rate_change);
    volume_handler.set(props.on_volume_change);
    bitrate_handler.set(props.on_bitrate_change);
    bitrates_handler.set(props.on_available_bitrates);
    ready_handler.set(props.on_ready_for_display);
    tracks_handler.set(props.on_tracks_change);
    subtitle_handler.set(props.on_subtitle);
    interruption_handler.set(props.on_audio_interrupted);
    fullscreen_handler.set(props.on_fullscreen_change);
    end_handler.set(props.on_end);
    error_handler.set(props.on_error);

    let controller_changed = {
        let binding = controller_binding.borrow();
        binding
            .as_ref()
            .is_none_or(|(current, _)| current != &active_controller)
    };
    if controller_changed {
        if let Some((controller, binding)) = controller_binding.borrow_mut().take() {
            controller.unbind(binding);
        }
        if let Some(sender) = runtime.sender() {
            let binding = active_controller.bind(sender);
            controller_binding
                .borrow_mut()
                .replace((active_controller.clone(), binding));
        }
    }

    let receiver_slot = use_hook(|| Rc::new(RefCell::new(runtime.take_receiver())));
    let event_controller = controller_binding.clone();
    let event_load_start = load_start_handler.clone();
    let event_load = load_handler.clone();
    let event_status = status_handler.clone();
    let event_progress = progress_handler.clone();
    let event_buffer = buffer_handler.clone();
    let event_seek = seek_handler.clone();
    let event_rate = rate_handler.clone();
    let event_volume = volume_handler.clone();
    let event_bitrate = bitrate_handler.clone();
    let event_bitrates = bitrates_handler.clone();
    let event_ready = ready_handler.clone();
    let event_tracks = tracks_handler.clone();
    let event_subtitle = subtitle_handler.clone();
    let event_interruption = interruption_handler.clone();
    let event_auto_hide = auto_hide.clone();
    let event_fullscreen = fullscreen_handler.clone();
    let event_end = end_handler.clone();
    let event_error = error_handler.clone();
    let _event_task = use_future(move || {
        let receiver = receiver_slot.borrow_mut().take();
        let event_controller = event_controller.clone();
        let event_load_start = event_load_start.clone();
        let event_load = event_load.clone();
        let event_status = event_status.clone();
        let event_progress = event_progress.clone();
        let event_buffer = event_buffer.clone();
        let event_seek = event_seek.clone();
        let event_rate = event_rate.clone();
        let event_volume = event_volume.clone();
        let event_bitrate = event_bitrate.clone();
        let event_bitrates = event_bitrates.clone();
        let event_ready = event_ready.clone();
        let event_tracks = event_tracks.clone();
        let event_subtitle = event_subtitle.clone();
        let event_interruption = event_interruption.clone();
        let event_auto_hide = event_auto_hide.clone();
        let event_fullscreen = event_fullscreen.clone();
        let event_end = event_end.clone();
        let event_error = event_error.clone();
        async move {
            let Some(mut receiver) = receiver else {
                return;
            };
            while let Some(event) = receiver.recv().await {
                match event {
                    UiEvent::Snapshot(snapshot) => {
                        let playing = snapshot.status.is_playing();
                        view_snapshot.set(snapshot.clone());
                        if !playing {
                            controls_visible.set(true);
                        }
                        if let Some((controller, binding)) = event_controller
                            .borrow()
                            .as_ref()
                            .map(|(controller, binding)| (controller.clone(), *binding))
                        {
                            controller.update_snapshot(binding, snapshot);
                        }
                    }
                    UiEvent::Status(status) => {
                        if let Some(handler) = event_status.get() {
                            handler.call(status);
                        }
                    }
                    UiEvent::LoadStart => {
                        if let Some(handler) = event_load_start.get() {
                            handler.call(());
                        }
                    }
                    UiEvent::Loaded(metadata) => {
                        if let Some(handler) = event_load.get() {
                            handler.call(metadata);
                        }
                    }
                    UiEvent::Progress(progress) => {
                        if let Some(handler) = event_progress.get() {
                            handler.call(progress);
                        }
                    }
                    UiEvent::Buffering(buffering) => {
                        if let Some(handler) = event_buffer.get() {
                            handler.call(buffering);
                        }
                    }
                    UiEvent::SeekCompleted(position) => {
                        if let Some(handler) = event_seek.get() {
                            handler.call(position);
                        }
                    }
                    UiEvent::PlaybackRateChanged(rate) => {
                        if let Some(handler) = event_rate.get() {
                            handler.call(rate);
                        }
                    }
                    UiEvent::VolumeChanged(volume) => {
                        if let Some(handler) = event_volume.get() {
                            handler.call(volume);
                        }
                    }
                    UiEvent::BitrateChanged(bitrate) => {
                        if let Some(handler) = event_bitrate.get() {
                            handler.call(bitrate);
                        }
                    }
                    UiEvent::AvailableBitrates(bitrates) => {
                        if let Some(handler) = event_bitrates.get() {
                            handler.call(bitrates);
                        }
                    }
                    UiEvent::ReadyForDisplay => {
                        if let Some(handler) = event_ready.get() {
                            handler.call(());
                        }
                    }
                    UiEvent::TracksChanged(tracks) => {
                        if let Some(handler) = event_tracks.get() {
                            handler.call(tracks);
                        }
                    }
                    UiEvent::Subtitle(cue) => {
                        if let Some(handler) = event_subtitle.get() {
                            handler.call(cue);
                        }
                    }
                    UiEvent::AudioInterrupted => {
                        if let Some(handler) = event_interruption.get() {
                            handler.call(());
                        }
                    }
                    UiEvent::ControlTick => {
                        if event_auto_hide.get().is_some_and(|delay| {
                            view_snapshot.peek().status.is_playing()
                                && Instant::now()
                                    .saturating_duration_since(*last_controls_interaction.peek())
                                    >= delay
                        }) {
                            controls_visible.set(false);
                        }
                    }
                    UiEvent::FullscreenChanged(next) => {
                        fullscreen.set(next);
                        controls_visible.set(true);
                        last_controls_interaction.set(Instant::now());
                        if let Some(handler) = event_fullscreen.get() {
                            handler.call(next);
                        }
                    }
                    UiEvent::Ended => {
                        if let Some(handler) = event_end.get() {
                            handler.call(());
                        }
                    }
                    UiEvent::Error(error) => {
                        if let Some(handler) = event_error.get() {
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
        autoplay: props.autoplay,
        looping: props.looping,
        muted: props.muted,
        volume: props.volume,
        playback_rate: props.playback_rate,
        initial_position: props.initial_position,
        resize_mode: props.resize_mode,
        progress_interval: props.progress_interval,
        subtitles: props.subtitles.clone(),
    };
    let configure_runtime = runtime.clone();
    use_effect(use_reactive((&configuration,), move |(configuration,)| {
        configure_runtime.send(WorkerMessage::Configure(configuration));
    }));

    let back_runtime = dioxus_core::try_consume_context::<arkit_runtime::RuntimeHandle>();
    let back_controller = active_controller.clone();
    let back_fullscreen = fullscreen;
    let scoped_back_handler = dioxus_hooks::use_callback(move |()| {
        if back_fullscreen() {
            let _ = back_controller.exit_fullscreen();
            true
        } else {
            false
        }
    });
    let back_handler: Rc<dyn Fn() -> bool> = Rc::new(move || scoped_back_handler.call(()));
    let _back_registration = use_hook(move || {
        back_runtime.map(|runtime| Rc::new(runtime.register_back_handler(back_handler)))
    });

    let effect_registration = surface_registration.clone();
    let effect_registered_node = registered_node.clone();
    let effect_runtime = runtime.clone();
    let native_liveness = dioxus_core::try_consume_context::<arkit_runtime::NativeLiveness>();
    use_mounted_node(node_ref.clone(), move |node| {
        let Some(node) = node else {
            // The same ref can briefly own an old and a new native node while
            // a Portal move is being committed. The generation-specific
            // native teardown below is authoritative; an unqualified
            // `None` here must not discard the newer registration.
            return;
        };
        let native_key = node.epoch();
        if effect_registered_node.get() == Some(native_key) {
            return;
        }
        effect_registration.borrow_mut().take();
        let Some(sender) = effect_runtime.sender() else {
            return;
        };
        match SurfaceRegistration::attach(&node, native_key, sender, native_liveness.clone()) {
            Ok(registration) => {
                effect_registration.borrow_mut().replace(registration);
                effect_registered_node.set(Some(native_key));
                let teardown_registration = effect_registration.clone();
                let teardown_registered_node = effect_registered_node.clone();
                // SAFETY: cleanup only removes native callbacks and releases
                // their worker senders before XComponent invalidation.
                let installed = unsafe {
                    node.install_native_teardown(move || {
                        let owns_registration = teardown_registration
                            .borrow()
                            .as_ref()
                            .is_some_and(|registration| registration.id() == native_key);
                        if owns_registration {
                            teardown_registration.borrow_mut().take();
                            teardown_registered_node.set(None);
                        }
                    })
                };
                if !installed {
                    effect_registration.borrow_mut().take();
                    effect_registered_node.set(None);
                }
            }
            Err(error) => effect_runtime.emit_error(error),
        }
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

    let is_fullscreen = fullscreen();
    let frame_width = if is_fullscreen {
        "100%".to_string()
    } else {
        props.width.clone()
    };
    let frame_height = if is_fullscreen {
        "100%".to_string()
    } else {
        props.height.clone().unwrap_or_else(|| "240".into())
    };
    let controls = props.controls.clone();
    let show_controls = controls.is_some() && controls_visible();
    let controls_enabled = controls.is_some();
    let control_configuration = controls;
    let control_controller = active_controller.clone();
    let snapshot = view_snapshot();
    let safe_bottom = if is_fullscreen { safe_area.bottom } else { 0.0 };
    let frame = rsx! {
        stack {
            width: frame_width,
            height: frame_height,
            alignment: "bottom-start",
            clip: true,
            hit_test_behavior: "default",
            background_color: props.background_color,
            onclick: move |_| {
                if controls_enabled {
                    controls_visible.set(true);
                    last_controls_interaction.set(Instant::now());
                }
            },
            xcomponent {
                native_ref: node_ref,
                width: "100%",
                height: "100%",
                background_color: props.background_color,
            }
            if show_controls {
                BuiltInVideoControls {
                    controller: control_controller,
                    snapshot,
                    configuration: control_configuration.expect("controls checked above"),
                    safe_bottom,
                    on_interaction: move |_| {
                        controls_visible.set(true);
                        last_controls_interaction.set(Instant::now());
                    },
                }
            }
        }
    };
    if is_fullscreen {
        rsx! {
            Portal {
                layer: OverlayLayer::Transient,
                {frame}
            }
        }
    } else {
        frame
    }
}
