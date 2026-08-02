//! Declarative terminal **surface** — render-state paint only.
//!
//! Host I/O (PTY / SSH / local shell) is **not** wired here. The embedder:
//!
//! 1. Feeds host output with [`TerminalController::feed_vt`]
//! 2. Receives host-bound input via [`TerminalProps::on_input`]
//! 3. Receives terminal→host replies via [`TerminalProps::on_write_pty`]
//!
//! ```text
//!   on_input / encode_*  ──►  your host (SSH, PTY, …)
//!   host output          ──►  feed_vt
//!   on_write_pty         ──►  your host
//!   capture + paint      ──►  this component
//! ```

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use arkit_hooks::{use_mounted_node, use_native_element_ref};
use arkit_prelude::*;
use dioxus_elements::event::PointerAction;

use crate::config::{Rgb, TerminalConfig, TerminalEffects};
use crate::engine::TerminalEngine;
use crate::error::TerminalError;
use crate::frame::{CursorVisualStyle, TerminalFrame};
use crate::ime::TerminalImeSession;
use crate::input::{KeyChord, KeyMods, MouseAction, MouseButton, MouseInput};
use crate::native_surface::SurfaceRegistration;
use crate::surface::TerminalSurfaceMetrics;
use crate::worker::{RenderPacket, WorkerHandle};

const BLINK_MS: u64 = 530;

struct ComponentRuntime {
    worker: RefCell<Option<WorkerHandle>>,
}

impl ComponentRuntime {
    fn new() -> Self {
        let worker = match WorkerHandle::spawn() {
            Ok(worker) => Some(worker),
            Err(error) => {
                ohos_hilog_binding::error(format!(
                    "arkit_terminal: failed to spawn render worker: {error}"
                ));
                None
            }
        };
        Self {
            worker: RefCell::new(worker),
        }
    }

    fn sender(&self) -> Option<std::sync::mpsc::Sender<crate::worker::WorkerMessage>> {
        self.worker.borrow().as_ref().map(WorkerHandle::sender)
    }

    fn publish(&self, frame: TerminalFrame, settings: PaintSettings) {
        if let Some(worker) = self.worker.borrow().as_ref() {
            worker.publish(RenderPacket {
                frame,
                metrics: settings.metrics,
                cursor_phase: settings.cursor_phase,
                cursor_blink: settings.cursor_blink,
                background_color: settings.background_color,
            });
        }
    }

    fn shutdown(&self) {
        self.worker.borrow_mut().take();
    }
}

#[derive(Clone, Copy)]
struct PaintSettings {
    metrics: TerminalSurfaceMetrics,
    cursor_phase: bool,
    cursor_blink: bool,
    background_color: u32,
}

/// Props for [`Terminal`].
#[derive(Props, Clone, PartialEq)]
pub struct TerminalProps {
    /// Full Ghostty-aligned configuration (colors, cursor, metrics, …).
    #[props(default)]
    pub config: Option<TerminalConfig>,
    /// Initial VT written once on mount (host→terminal path).
    #[props(default)]
    pub initial: Option<String>,
    #[props(default = 80)]
    pub cols: u16,
    #[props(default = 24)]
    pub rows: u16,
    #[props(default = "100%".to_string())]
    pub width: String,
    #[props(default = "360".to_string())]
    pub height: String,
    #[props(default = 0xFF0B_1220_u32)]
    pub background_color: u32,
    #[props(default = 0xFFE2_E8F0_u32)]
    pub foreground_color: u32,
    /// Drive caret blink phase on the native render worker.
    #[props(default = true)]
    pub cursor_blink: bool,
    /// Capture soft-keyboard / pointer and emit [`on_input`] host bytes.
    /// Set `false` for a pure paint surface (embedder supplies its own input).
    #[props(default = true)]
    pub capture_input: bool,
    #[props(default)]
    pub controller: Option<TerminalController>,
    #[props(default)]
    pub on_error: Option<EventHandler<TerminalError>>,
    /// Host-bound bytes from IME / encoded keys / mouse / focus.
    /// **Embedder must write these to the PTY/SSH** — they are not fed into VT.
    #[props(default)]
    pub on_input: Option<EventHandler<Vec<u8>>>,
    /// Terminal → host replies (DA, DSR, …). Write to the same host.
    #[props(default)]
    pub on_write_pty: Option<EventHandler<Vec<u8>>>,
    #[props(default)]
    pub on_bell: Option<EventHandler<()>>,
    #[props(default)]
    pub on_title: Option<EventHandler<String>>,
    #[props(default)]
    pub on_pwd: Option<EventHandler<String>>,
    #[props(default)]
    pub on_frame: Option<EventHandler<TerminalFrame>>,
}

#[derive(Clone, Default)]
struct TerminalCallbackSlots {
    on_error: Rc<Cell<Option<EventHandler<TerminalError>>>>,
    on_input: Rc<Cell<Option<EventHandler<Vec<u8>>>>>,
    on_write_pty: Rc<Cell<Option<EventHandler<Vec<u8>>>>>,
    on_bell: Rc<Cell<Option<EventHandler<()>>>>,
    on_title: Rc<Cell<Option<EventHandler<String>>>>,
    on_pwd: Rc<Cell<Option<EventHandler<String>>>>,
    on_frame: Rc<Cell<Option<EventHandler<TerminalFrame>>>>,
}

impl TerminalCallbackSlots {
    fn update(&self, props: &TerminalProps) {
        self.on_error.set(props.on_error);
        self.on_input.set(props.on_input);
        self.on_write_pty.set(props.on_write_pty);
        self.on_bell.set(props.on_bell);
        self.on_title.set(props.on_title);
        self.on_pwd.set(props.on_pwd);
        self.on_frame.set(props.on_frame);
    }
}

/// Cloneable handle for imperative control of a mounted terminal.
#[derive(Clone, Default)]
pub struct TerminalController {
    inner: Rc<RefCell<Option<TerminalEngine>>>,
    frame: Rc<RefCell<TerminalFrame>>,
    generation: Rc<Cell<u64>>,
    on_change: Rc<RefCell<Option<Box<TerminalChangeCallback>>>>,
    pending_updates: Rc<RefCell<VecDeque<TerminalUpdate>>>,
    publishing: Rc<Cell<bool>>,
    callback_epoch: Rc<Cell<u64>>,
}

type TerminalChangeCallback = dyn FnMut(TerminalFrame, TerminalEffects, Option<TerminalError>);

struct TerminalUpdate {
    frame: TerminalFrame,
    effects: TerminalEffects,
    error: Option<TerminalError>,
}

impl PartialEq for TerminalController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl TerminalController {
    pub fn new() -> Self {
        Self::default()
    }

    /// Host → terminal (PTY/SSH output). Preferred name for the Ghostty model.
    pub fn feed_vt(&self, data: &[u8]) {
        self.write_bytes(data);
    }

    pub fn write_str(&self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    pub fn write_bytes(&self, data: &[u8]) {
        self.update_engine(|engine| {
            engine.write_bytes(data);
            Ok(())
        });
    }

    /// Encode a named key → host-bound bytes (does not paint / does not feed VT).
    pub fn encode_key(&self, name: &str) -> Vec<u8> {
        let guard = self.inner.borrow();
        guard
            .as_ref()
            .and_then(|e| e.encode_key(name).ok())
            .unwrap_or_else(|| fallback_key_bytes(name))
    }

    pub fn encode_key_chord(&self, chord: KeyChord) -> Vec<u8> {
        let guard = self.inner.borrow();
        guard
            .as_ref()
            .and_then(|e| e.encode_key_chord(chord.clone()).ok())
            .unwrap_or_else(|| {
                chord
                    .utf8
                    .map(|s| s.into_bytes())
                    .unwrap_or_else(|| fallback_key_bytes(&chord.name))
            })
    }

    /// UTF-8 text as host-bound bytes (typed characters).
    pub fn encode_text(&self, text: &str) -> Vec<u8> {
        text.as_bytes().to_vec()
    }

    pub fn encode_mouse(&self, event: MouseInput) -> Vec<u8> {
        let guard = self.inner.borrow();
        guard
            .as_ref()
            .and_then(|e| e.encode_mouse(event).ok())
            .unwrap_or_default()
    }

    pub fn encode_focus(&self, gained: bool) -> Vec<u8> {
        let guard = self.inner.borrow();
        guard
            .as_ref()
            .and_then(|e| e.encode_focus(gained).ok())
            .unwrap_or_default()
    }

    pub fn reconfigure(&self, config: TerminalConfig) {
        self.update_engine(|engine| engine.reconfigure(config));
    }

    /// Pin viewport to live bottom (after host output / user “jump to end”).
    pub fn scroll_to_bottom(&self) {
        self.update_engine(|engine| {
            engine.scroll_to_bottom();
            Ok(())
        });
    }

    /// Jump to the top of scrollback history.
    pub fn scroll_to_top(&self) {
        self.update_engine(|engine| {
            engine.scroll_to_top();
            Ok(())
        });
    }

    /// Scroll history by signed rows (negative = up into scrollback).
    ///
    /// This is Ghostty `ghostty_terminal_scroll_viewport` DELTA — not UI layout
    /// scrolling of a tall bitmap. The paint surface always shows one viewport.
    pub fn scroll_by(&self, delta_rows: i64) {
        if delta_rows == 0 {
            return;
        }
        self.update_engine(|engine| {
            engine.scroll_by(delta_rows);
            Ok(())
        });
    }

    /// Absolute history row as the first visible line.
    pub fn scroll_to_row(&self, row: u64) {
        self.update_engine(|engine| {
            engine.scroll_to_row(row);
            Ok(())
        });
    }

    pub fn snapshot(&self) -> String {
        self.frame.borrow().plain()
    }

    pub fn frame(&self) -> TerminalFrame {
        self.frame.borrow().clone()
    }

    pub fn generation(&self) -> u64 {
        self.generation.get()
    }

    fn capture_update(
        &self,
        engine: &mut TerminalEngine,
        mut error: Option<TerminalError>,
    ) -> TerminalUpdate {
        let effects = engine.take_effects();
        let frame = match engine.capture() {
            Ok(frame) => {
                *self.frame.borrow_mut() = frame.clone();
                self.generation.set(self.generation.get().wrapping_add(1));
                frame
            }
            Err(capture_error) => {
                if error.is_none() {
                    error = Some(capture_error);
                }
                self.frame.borrow().clone()
            }
        };
        TerminalUpdate {
            frame,
            effects,
            error,
        }
    }

    fn publish(&self, update: TerminalUpdate) {
        self.pending_updates.borrow_mut().push_back(update);
        if self.publishing.replace(true) {
            return;
        }
        let _publishing = PublishingGuard(&self.publishing);

        loop {
            let Some(update) = self.pending_updates.borrow_mut().pop_front() else {
                break;
            };
            let epoch = self.callback_epoch.get();
            let Some(mut callback) = self.on_change.borrow_mut().take() else {
                continue;
            };
            callback(update.frame, update.effects, update.error);

            // A callback may synchronously drive the controller again. Those
            // updates are queued above. It may also replace/remove itself; do
            // not resurrect the old callback in that case.
            if self.callback_epoch.get() == epoch && self.on_change.borrow().is_none() {
                *self.on_change.borrow_mut() = Some(callback);
            }
        }
    }

    /// Mutate/capture under the engine borrow, then release it before invoking
    /// user callbacks. `on_frame` may legitimately call this controller again.
    fn update_engine(&self, update: impl FnOnce(&mut TerminalEngine) -> Result<(), TerminalError>) {
        let captured = {
            let mut guard = self.inner.borrow_mut();
            let Some(engine) = guard.as_mut() else {
                return;
            };
            let error = update(engine).err();
            self.capture_update(engine, error)
        };
        self.publish(captured);
    }

    fn detach(&self) {
        self.callback_epoch
            .set(self.callback_epoch.get().wrapping_add(1));
        self.on_change.borrow_mut().take();
        self.pending_updates.borrow_mut().clear();
        self.inner.borrow_mut().take();
    }

    fn attach(&self, mut engine: TerminalEngine) {
        let update = self.capture_update(&mut engine, None);
        *self.inner.borrow_mut() = Some(engine);
        self.publish(update);
    }

    fn set_on_change(
        &self,
        cb: Box<dyn FnMut(TerminalFrame, TerminalEffects, Option<TerminalError>)>,
    ) {
        self.callback_epoch
            .set(self.callback_epoch.get().wrapping_add(1));
        *self.on_change.borrow_mut() = Some(cb);
    }
}

struct PublishingGuard<'a>(&'a Cell<bool>);

impl Drop for PublishingGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

fn fallback_key_bytes(name: &str) -> Vec<u8> {
    match name {
        "enter" | "return" => b"\r".to_vec(),
        "backspace" => b"\x7f".to_vec(),
        "tab" => b"\t".to_vec(),
        "escape" | "esc" => b"\x1b".to_vec(),
        "arrow_up" | "up" => b"\x1b[A".to_vec(),
        "arrow_down" | "down" => b"\x1b[B".to_vec(),
        "arrow_right" | "right" => b"\x1b[C".to_vec(),
        "arrow_left" | "left" => b"\x1b[D".to_vec(),
        "home" => b"\x1b[H".to_vec(),
        "end" => b"\x1b[F".to_vec(),
        "page_up" => b"\x1b[5~".to_vec(),
        "page_down" => b"\x1b[6~".to_vec(),
        "delete" => b"\x1b[3~".to_vec(),
        "space" => b" ".to_vec(),
        _ => Vec::new(),
    }
}

/// Terminal surface aligned with Ghostty:
/// - fixed cols×rows VT geometry (keyboard height must not reflow cols)
/// - paint cells **scale to fit** the surface so nothing is center-clipped
/// - DECAWM wrap for long lines
/// - scrollback via Ghostty `scroll_viewport` (finger-follows-content)
/// - direct native IME activation only after a gesture resolves to a tap
#[component]
pub fn Terminal(props: TerminalProps) -> Element {
    let controller = props.controller.clone().unwrap_or_default();
    let node_ref = use_native_element_ref();
    let runtime = use_hook(|| Rc::new(ComponentRuntime::new()));
    let surface_registration = use_hook(|| Rc::new(RefCell::new(None::<SurfaceRegistration>)));
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<u64>)));
    // Pointer tracking is transient interaction state, not paint state. A
    // reactive Signal here rerendered the entire terminal on every move and
    // again for every Ghostty row update, which made touch scrolling stutter.
    let gesture = use_hook(|| Rc::new(RefCell::new(TouchGesture::default())));
    let surface_metrics =
        use_hook(|| Rc::new(Cell::new(TerminalSurfaceMetrics::fallback(window_scale()))));
    let paint_settings = use_hook({
        let metrics = surface_metrics.get();
        let cursor_blink = props.cursor_blink;
        let background_color = props.background_color;
        move || {
            Rc::new(Cell::new(PaintSettings {
                metrics,
                cursor_phase: true,
                cursor_blink,
                background_color,
            }))
        }
    });
    let mut current_settings = paint_settings.get();
    current_settings.cursor_blink = props.cursor_blink;
    current_settings.background_color = props.background_color;
    paint_settings.set(current_settings);

    let callbacks = use_hook(TerminalCallbackSlots::default);
    callbacks.update(&props);
    let enable_blink = props.cursor_blink;
    let capture_input = props.capture_input;
    let surface_bg = props.background_color;
    let surface_fg = props.foreground_color;
    let ime_session = use_hook({
        let controller = controller.clone();
        let on_input = callbacks.on_input.clone();
        move || Rc::new(TerminalImeSession::new(controller, on_input))
    });

    let config = props.config.clone().unwrap_or_else(|| {
        let metrics = surface_metrics.get();
        TerminalConfig::default()
            .with_size(props.cols, props.rows)
            .with_cell_metrics(
                metrics.native_cell_width_px(),
                metrics.native_cell_height_px(),
            )
            .with_theme(
                Rgb::from_argb(surface_fg),
                Rgb::from_argb(surface_bg),
                Rgb::from_argb(surface_fg),
            )
            .with_cursor_style(CursorVisualStyle::Block, enable_blink)
    });
    // Stable VT geometry — never changed by layout/keyboard.
    let vt_cols = config.cols.max(1);
    let vt_rows = config.rows.max(1);
    let base_config = config.clone();
    let async_runtime = arkit_runtime::use_runtime_handle().tokio();

    use_hook({
        let runtime = runtime.clone();
        let controller = controller.clone();
        let settings = paint_settings.clone();
        let handle = async_runtime.clone();
        move || {
            dioxus_core::spawn(async move {
                loop {
                    let sleeper = handle.spawn(async {
                        tokio::time::sleep(std::time::Duration::from_millis(BLINK_MS)).await;
                    });
                    let _ = sleeper.await;
                    let mut next = settings.get();
                    next.cursor_phase = !next.cursor_phase;
                    settings.set(next);
                    let frame = controller.frame();
                    if next.cursor_blink && frame.cursor.visible && frame.cursor.blinking {
                        runtime.publish(frame, next);
                    }
                }
            });
        }
    });

    use_hook({
        let controller = controller.clone();
        let callbacks = callbacks.clone();
        let runtime = runtime.clone();
        let settings = paint_settings.clone();
        move || {
            controller.set_on_change(Box::new(move |frame, effects, error| {
                if let Some(error) = error {
                    if let Some(handler) = callbacks.on_error.get() {
                        handler.call(error);
                    }
                }
                if !effects.write_pty.is_empty() {
                    if let Some(h) = callbacks.on_write_pty.get() {
                        h.call(effects.write_pty.clone());
                    }
                }
                if effects.bell {
                    if let Some(h) = callbacks.on_bell.get() {
                        h.call(());
                    }
                }
                if let Some(title) = effects.title.clone() {
                    if let Some(h) = callbacks.on_title.get() {
                        h.call(title);
                    }
                }
                if let Some(pwd) = effects.pwd.clone() {
                    if let Some(h) = callbacks.on_pwd.get() {
                        h.call(pwd);
                    }
                }
                if let Some(h) = callbacks.on_frame.get() {
                    h.call(frame.clone());
                }
                runtime.publish(frame, settings.get());
            }));
        }
    });

    use_hook({
        let controller = controller.clone();
        let callbacks = callbacks.clone();
        let initial = props.initial.clone();
        let config = config.clone();
        move || match TerminalEngine::with_config(config) {
            Ok(mut engine) => {
                if let Some(banner) = initial {
                    engine.write_str(&banner);
                }
                engine.write_str("\x1b[?25h");
                controller.attach(engine);
            }
            Err(error) => {
                if let Some(handler) = callbacks.on_error.get() {
                    handler.call(error);
                }
            }
        }
    });

    let registration_slot = surface_registration.clone();
    let registered_slot = registered_node.clone();
    let registration_runtime = runtime.clone();
    let registration_errors = callbacks.on_error.clone();
    use_mounted_node(node_ref.clone(), move |node| {
        let Some(node) = node else {
            registration_slot.borrow_mut().take();
            registered_slot.set(None);
            return;
        };
        let native_key = node.epoch();
        if registered_slot.get() == Some(native_key) {
            return;
        }
        registration_slot.borrow_mut().take();
        let Some(sender) = registration_runtime.sender() else {
            return;
        };
        let registration = SurfaceRegistration::attach(&node, sender);
        match registration {
            Ok(registration) => {
                registration_slot.borrow_mut().replace(registration);
                registered_slot.set(Some(native_key));
                let teardown_registration = registration_slot.clone();
                let teardown_registered_node = registered_slot.clone();
                // SAFETY: cleanup only replaces XComponent callbacks and
                // releases worker senders before native node invalidation.
                let installed = unsafe {
                    node.install_native_teardown(move || {
                        teardown_registration.borrow_mut().take();
                        teardown_registered_node.set(None);
                    })
                };
                if !installed {
                    registration_slot.borrow_mut().take();
                    registered_slot.set(None);
                }
            }
            Err(error) => {
                if let Some(handler) = registration_errors.get() {
                    handler.call(error);
                }
            }
        }
    });

    use_drop({
        let controller = controller.clone();
        let ime_session = ime_session.clone();
        let surface_registration = surface_registration.clone();
        let runtime = runtime.clone();
        move || {
            ime_session.deactivate();
            surface_registration.borrow_mut().take();
            controller.detach();
            runtime.shutdown();
        }
    });

    let c_touch = controller.clone();
    let c_fit = controller.clone();
    let fit_config = base_config.clone();
    let ime_touch = ime_session.clone();
    let input_touch = callbacks.on_input.clone();
    let gesture_touch = gesture.clone();
    let touch_metrics = surface_metrics.clone();
    let fit_metrics = surface_metrics.clone();
    let fit_settings = paint_settings.clone();
    let fit_runtime = runtime.clone();

    rsx! {
        xcomponent {
            native_ref: node_ref,
            width: props.width.clone(),
            height: props.height.clone(),
            background_color: surface_bg,
            border_radius: 12.0,
            clip: true,
            onarea: move |evt: dioxus_core::Event<dioxus_elements::event::AreaData>| {
                let f = evt.data().frame;
                if !f.is_measured() {
                    return;
                }
                let next = TerminalSurfaceMetrics::fit(
                    f.width as f64,
                    f.height as f64,
                    window_scale(),
                    vt_cols,
                    vt_rows,
                );
                let previous = fit_metrics.get();
                if !next.differs_from(previous) {
                    return;
                }
                fit_metrics.set(next);
                let mut settings = fit_settings.get();
                settings.metrics = next;
                fit_settings.set(settings);

                // Ghostty and the native renderer share the same physical cell
                // box while VT columns/rows remain stable across IME resize.
                let native_width = next.native_cell_width_px();
                let native_height = next.native_cell_height_px();
                if native_width != fit_config.cell_width_px
                    || native_height != fit_config.cell_height_px
                {
                    let mut cfg = fit_config.clone();
                    cfg.cell_width_px = native_width;
                    cfg.cell_height_px = native_height;
                    cfg.cols = vt_cols;
                    cfg.rows = vt_rows;
                    c_fit.reconfigure(cfg);
                } else {
                    fit_runtime.publish(c_fit.frame(), settings);
                }
            },
            ontouch: move |evt| {
                if !capture_input {
                    return;
                }
                let Some(p) = evt.pointer else {
                    return;
                };
                match p.action {
                    PointerAction::Cancel => {
                        *gesture_touch.borrow_mut() = TouchGesture::default();
                    }
                    PointerAction::Down => {
                        *gesture_touch.borrow_mut() = TouchGesture::begin(p.x, p.y);
                    }
                    PointerAction::Move | PointerAction::Unknown => {
                        let metrics = touch_metrics.get();
                        let scroll_slop = metrics.scroll_slop_vp();
                        let row_vp = metrics.cell_height_vp.max(1.0) as f32;
                        let rows = {
                            let mut g = gesture_touch.borrow_mut();
                            if !g.active {
                                return;
                            }
                            if !g.is_scroll {
                                let dy = (p.y - g.origin_y).abs();
                                let dx = (p.x - g.origin_x).abs();
                                if dy > scroll_slop && dy >= dx {
                                    g.is_scroll = true;
                                }
                            }
                            if !g.is_scroll {
                                0
                            } else {
                                // Pointer coordinates are vp. Ghostty DELTA:
                                // negative = into history. Finger down makes
                                // older rows follow the content downward.
                                let step = p.y - g.last_y;
                                g.last_y = p.y;
                                g.pixel_acc += step;
                                let rows = (g.pixel_acc / row_vp) as i64;
                                g.pixel_acc -= rows as f32 * row_vp;
                                rows
                            }
                        };
                        if rows != 0 {
                            // Ghostty moves its cheap viewport pin immediately.
                            // The render worker independently replaces stale
                            // snapshots, matching Ghostty's queueRender model.
                            c_touch.scroll_by(-rows);
                        }
                    }
                    PointerAction::Up => {
                        let g = std::mem::take(&mut *gesture_touch.borrow_mut());
                        if g.is_scroll {
                            return;
                        }
                        let metrics = touch_metrics.get();
                        let scroll_slop = metrics.scroll_slop_vp();
                        let dx = (p.x - g.origin_x).abs();
                        let dy = (p.y - g.origin_y).abs();
                        if g.active && dx <= scroll_slop && dy <= scroll_slop {
                            // Repeatable even when the user manually dismissed
                            // a still-attached software keyboard.
                            ime_touch.show_keyboard();
                            let (x, y) =
                                metrics.content_position_px_from_vp(g.origin_x, g.origin_y);
                            let mut host = c_touch.encode_mouse(MouseInput {
                                action: MouseAction::Press,
                                button: MouseButton::Left,
                                x,
                                y,
                                mods: KeyMods::default(),
                            });
                            host.extend(c_touch.encode_mouse(MouseInput {
                                action: MouseAction::Release,
                                button: MouseButton::Left,
                                x,
                                y,
                                mods: KeyMods::default(),
                            }));
                            emit_host(&input_touch, host);
                        }
                    }
                }
            },
        }
    }
}

/// Physical-pixel → vp scale from the runtime window metrics.
fn window_scale() -> f64 {
    dioxus_core::try_consume_context::<arkit_runtime::WindowMetricsHandle>()
        .map(|h| h.get().scale)
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(1.0) as f64
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TouchGesture {
    active: bool,
    origin_x: f32,
    origin_y: f32,
    last_y: f32,
    /// Sub-row pixel accumulator for smooth scroll_by.
    pixel_acc: f32,
    is_scroll: bool,
}

impl Default for TouchGesture {
    fn default() -> Self {
        Self {
            active: false,
            origin_x: 0.0,
            origin_y: 0.0,
            last_y: 0.0,
            pixel_acc: 0.0,
            is_scroll: false,
        }
    }
}

impl TouchGesture {
    fn begin(x: f32, y: f32) -> Self {
        Self {
            active: true,
            origin_x: x,
            origin_y: y,
            last_y: y,
            pixel_acc: 0.0,
            is_scroll: false,
        }
    }
}

fn emit_host(on_input: &Cell<Option<EventHandler<Vec<u8>>>>, bytes: Vec<u8>) {
    if bytes.is_empty() {
        return;
    }
    if let Some(h) = on_input.get() {
        h.call(bytes);
    }
}
