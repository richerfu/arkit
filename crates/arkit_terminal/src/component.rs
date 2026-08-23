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
//!   host output          ──►  feed_vt (mailbox)
//!   on_write_pty         ──►  your host
//!   vsync + GPU paint    ──►  this component
//! ```

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use arkit_hooks::{use_mounted_node, use_native_element_ref};
use arkit_prelude::*;
use dioxus_elements::event::PointerAction;

use crate::config::{Rgb, TerminalConfig};
use crate::error::TerminalError;
use crate::frame::{CursorVisualStyle, TerminalFrame};
use crate::ime::TerminalImeSession;
use crate::input::{self, EncodeState, KeyChord, KeyMods, MouseAction, MouseButton, MouseInput};
use crate::native_surface::SurfaceRegistration;
use crate::surface::TerminalSurfaceMetrics;
use crate::worker::{PaintSettings, TerminalShared, WorkerHandle, WorkerMessage};

struct ComponentRuntime {
    worker: RefCell<Option<WorkerHandle>>,
}

impl ComponentRuntime {
    fn new(shared: Arc<TerminalShared>) -> Self {
        let worker = match WorkerHandle::spawn(shared) {
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

    fn sender(&self) -> Option<Sender<WorkerMessage>> {
        self.worker.borrow().as_ref().map(WorkerHandle::sender)
    }

    fn vsync_pending(&self) -> Option<Arc<std::sync::atomic::AtomicBool>> {
        self.worker
            .borrow()
            .as_ref()
            .map(WorkerHandle::vsync_pending)
    }

    fn take_notices(
        &self,
    ) -> Option<tokio::sync::mpsc::UnboundedReceiver<crate::worker::UiNotice>> {
        self.worker
            .borrow()
            .as_ref()
            .and_then(WorkerHandle::take_notices)
    }

    fn update_paint(&self, settings: PaintSettings) {
        if let Some(worker) = self.worker.borrow().as_ref() {
            worker.shared().set_paint(settings);
            worker.request_draw();
        }
    }

    fn send(&self, message: WorkerMessage) {
        if let Some(worker) = self.worker.borrow().as_ref() {
            worker.shared().send(message);
        }
    }

    fn shutdown(&self) {
        self.worker.borrow_mut().take();
    }
}

/// Props for [`Terminal`].
#[derive(Props, Clone, PartialEq)]
pub struct TerminalProps {
    /// Colors, cursor, cell metrics, and grid size.
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

/// Thread-safe host→VT byte sink. SSH/PTY writers can push without the UI.
#[derive(Clone)]
pub struct TerminalInbox {
    shared: Arc<TerminalShared>,
}

impl TerminalInbox {
    pub fn push(&self, bytes: &[u8]) {
        self.shared.push_bytes(bytes);
    }
}

/// Cloneable handle for imperative control of a mounted terminal.
///
/// Host output ([`Self::feed_vt`]) only appends to a shared mailbox. rio-vt
/// parse, grid capture, and wgpu present run on the `arkit-terminal` worker
/// and are paced by the XComponent vsync callback.
#[derive(Clone)]
pub struct TerminalController {
    shared: Arc<TerminalShared>,
    control: Rc<RefCell<Option<Sender<WorkerMessage>>>>,
    show_keyboard: Rc<RefCell<Option<ShowKeyboardFn>>>,
    hide_keyboard: Rc<RefCell<Option<HideKeyboardFn>>>,
    ime_visible: Rc<RefCell<Arc<std::sync::atomic::AtomicBool>>>,
}

type ShowKeyboardFn = Rc<dyn Fn()>;
type HideKeyboardFn = Rc<dyn Fn()>;

impl Default for TerminalController {
    fn default() -> Self {
        Self {
            shared: Arc::new(TerminalShared::new()),
            control: Rc::new(RefCell::new(None)),
            show_keyboard: Rc::new(RefCell::new(None)),
            hide_keyboard: Rc::new(RefCell::new(None)),
            ime_visible: Rc::new(RefCell::new(Arc::new(std::sync::atomic::AtomicBool::new(
                false,
            )))),
        }
    }
}

impl PartialEq for TerminalController {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.shared, &other.shared)
    }
}

impl TerminalController {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the native IME, matching `@ohos-rs/terminal` `showKeyboard()`.
    pub fn show_keyboard(&self) {
        if let Some(show) = self.show_keyboard.borrow().clone() {
            show();
        }
    }

    pub fn hide_keyboard(&self) {
        if let Some(hide) = self.hide_keyboard.borrow().clone() {
            hide();
        }
    }

    pub fn hide_keyboard_if_visible(&self) -> bool {
        if !self
            .ime_visible
            .borrow()
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return false;
        }
        self.hide_keyboard();
        true
    }

    pub(crate) fn bind_show_keyboard(&self, show: ShowKeyboardFn) {
        *self.show_keyboard.borrow_mut() = Some(show);
    }

    pub(crate) fn bind_hide_keyboard(
        &self,
        hide: HideKeyboardFn,
        visible: Arc<std::sync::atomic::AtomicBool>,
    ) {
        *self.hide_keyboard.borrow_mut() = Some(hide);
        *self.ime_visible.borrow_mut() = visible;
    }

    pub(crate) fn shared(&self) -> Arc<TerminalShared> {
        self.shared.clone()
    }

    pub fn inbox(&self) -> TerminalInbox {
        TerminalInbox {
            shared: self.shared.clone(),
        }
    }

    fn bind_control(&self, sender: Sender<WorkerMessage>) {
        *self.control.borrow_mut() = Some(sender.clone());
        self.shared.set_control(Some(sender));
    }

    fn send_control(&self, message: WorkerMessage) {
        self.shared.send(message);
    }

    fn request_draw(&self) {
        self.shared.request_draw();
    }

    /// Host → terminal (PTY/SSH output).
    ///
    /// Appends to a mailbox. Parsing and GPU present happen on the render
    /// worker, aligned to XComponent vsync when the surface is live.
    pub fn feed_vt(&self, data: &[u8]) {
        self.write_bytes(data);
    }

    pub fn write_str(&self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    pub fn write_bytes(&self, data: &[u8]) {
        self.shared.push_bytes(data);
    }

    /// Encode a named key → host-bound bytes (does not paint / does not feed VT).
    pub fn encode_key(&self, name: &str) -> Vec<u8> {
        input::encode_named_key(self.encode_state(), name)
            .unwrap_or_else(|_| fallback_key_bytes(name))
    }

    pub fn encode_key_chord(&self, chord: KeyChord) -> Vec<u8> {
        input::encode_key_chord(self.encode_state(), chord.clone()).unwrap_or_else(|_| {
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
        let mut config = TerminalConfig::default();
        config.cell_width_px = self.shared.cell_width_px();
        config.cell_height_px = self.shared.cell_height_px();
        input::encode_mouse(self.encode_state(), event, &config).unwrap_or_default()
    }

    pub fn encode_focus(&self, gained: bool) -> Vec<u8> {
        input::encode_focus(self.encode_state(), gained).unwrap_or_default()
    }

    pub fn reconfigure(&self, config: TerminalConfig) {
        self.shared
            .set_cell_metrics(config.cell_width_px, config.cell_height_px);
        self.send_control(WorkerMessage::Reconfigure(config));
    }

    /// Pin viewport to live bottom (after host output / user “jump to end”).
    pub fn scroll_to_bottom(&self) {
        self.send_control(WorkerMessage::ScrollToBottom);
    }

    /// Jump to the top of scrollback history.
    pub fn scroll_to_top(&self) {
        self.send_control(WorkerMessage::ScrollToTop);
    }

    /// Scroll history by signed rows (negative = up into scrollback).
    ///
    /// Signed row delta — not UI layout scrolling of a tall bitmap. The paint
    /// surface always shows one viewport.
    pub fn scroll_by(&self, delta_rows: i64) {
        if delta_rows == 0 {
            return;
        }
        self.shared.add_scroll_delta(delta_rows);
        self.request_draw();
    }

    /// Absolute history row as the first visible line.
    pub fn scroll_to_row(&self, row: u64) {
        self.send_control(WorkerMessage::ScrollToRow(row));
    }

    pub fn snapshot(&self) -> String {
        self.shared.last_frame().plain()
    }

    pub fn frame(&self) -> TerminalFrame {
        self.shared.last_frame()
    }

    pub fn generation(&self) -> u64 {
        self.shared.generation()
    }

    fn encode_state(&self) -> EncodeState {
        EncodeState::from_bits(self.shared.encode_bits())
    }

    fn detach(&self) {
        self.shared.clear_pending();
        self.control.borrow_mut().take();
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

/// Terminal surface:
/// - fixed cols×rows VT geometry (keyboard height must not reflow cols)
/// - paint cells **scale to fit** the surface so nothing is center-clipped
/// - DECAWM wrap for long lines
/// - scrollback via viewport pin (finger-follows-content)
/// - direct native IME activation only after a gesture resolves to a tap
#[component]
pub fn Terminal(props: TerminalProps) -> Element {
    let controller = props.controller.clone().unwrap_or_default();
    let node_ref = use_native_element_ref();
    let runtime = use_hook({
        let shared = controller.shared();
        move || Rc::new(ComponentRuntime::new(shared))
    });
    if let Some(sender) = runtime.sender() {
        controller.bind_control(sender);
    }
    let surface_registration = use_hook(|| Rc::new(RefCell::new(None::<SurfaceRegistration>)));
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<u64>)));
    // Pointer tracking is transient interaction state, not paint state. A
    // reactive Signal here rerendered the entire terminal on every move and
    // again for every VT row update, which made touch scrolling stutter.
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
                cursor_blink,
                background_color,
            }))
        }
    });
    {
        let mut current_settings = paint_settings.get();
        if current_settings.cursor_blink != props.cursor_blink
            || current_settings.background_color != props.background_color
        {
            current_settings.cursor_blink = props.cursor_blink;
            current_settings.background_color = props.background_color;
            paint_settings.set(current_settings);
            runtime.update_paint(current_settings);
        }
    }

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
    controller.bind_show_keyboard({
        let ime_session = ime_session.clone();
        Rc::new(move || ime_session.show_keyboard())
    });
    controller.bind_hide_keyboard(
        {
            let ime_session = ime_session.clone();
            Rc::new(move || ime_session.hide_keyboard())
        },
        ime_session.visible_flag(),
    );

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

    use_hook({
        let callbacks = callbacks.clone();
        let runtime = runtime.clone();
        let controller = controller.clone();
        move || {
            if let Some(mut rx) = runtime.take_notices() {
                dioxus_core::spawn(async move {
                    while let Some(notice) = rx.recv().await {
                        dispatch_notice(&callbacks, &controller, notice);
                    }
                });
            }
        }
    });

    use_hook({
        let runtime = runtime.clone();
        let initial = props.initial.clone();
        let config = config.clone();
        move || {
            runtime.send(WorkerMessage::Attach { config, initial });
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
        let Some(vsync_pending) = registration_runtime.vsync_pending() else {
            return;
        };
        let registration = SurfaceRegistration::attach(&node, sender, vsync_pending);
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

                // The engine and the native renderer share the same physical
                // cell box while VT columns/rows remain stable across IME resize.
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
                }
                fit_runtime.update_paint(settings);
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
                        let (x, y) = pointer_pos(&p);
                        *gesture_touch.borrow_mut() = TouchGesture::begin(x, y);
                    }
                    PointerAction::Move | PointerAction::Unknown => {
                        let metrics = touch_metrics.get();
                        let scroll_slop = metrics.scroll_slop_vp();
                        let row_vp = metrics.cell_height_vp.max(1.0) as f32;
                        let (x, y) = pointer_pos(&p);
                        let rows = {
                            let mut g = gesture_touch.borrow_mut();
                            if !g.active {
                                return;
                            }
                            g.note_move(x, y, scroll_slop);
                            if !g.is_scroll {
                                0
                            } else {
                                // Pointer coordinates are vp. Negative delta
                                // is into history. Finger down makes older
                                // rows follow the content downward.
                                let step = y - g.last_y;
                                g.last_y = y;
                                g.pixel_acc += step;
                                let rows = (g.pixel_acc / row_vp) as i64;
                                g.pixel_acc -= rows as f32 * row_vp;
                                rows
                            }
                        };
                        if rows != 0 {
                            // Move the viewport pin immediately. The render
                            // worker independently replaces stale snapshots.
                            c_touch.scroll_by(-rows);
                        }
                    }
                    PointerAction::Up => {
                        let metrics = touch_metrics.get();
                        let scroll_slop = metrics.scroll_slop_vp();
                        let (x, y) = pointer_pos(&p);
                        let g = {
                            let mut g = std::mem::take(&mut *gesture_touch.borrow_mut());
                            if g.active {
                                g.note_move(x, y, scroll_slop);
                            }
                            g
                        };
                        // Any drag suppresses IME, even if the finger lifts
                        // near the origin. HarmonyOS sometimes reports Up at
                        // the Down coordinate after a coalesced Move stream.
                        if !g.active || g.is_drag(scroll_slop) {
                            return;
                        }
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
    max_abs_dx: f32,
    max_abs_dy: f32,
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
            max_abs_dx: 0.0,
            max_abs_dy: 0.0,
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
            max_abs_dx: 0.0,
            max_abs_dy: 0.0,
        }
    }

    fn note_move(&mut self, x: f32, y: f32, scroll_slop: f32) {
        let dx = (x - self.origin_x).abs();
        let dy = (y - self.origin_y).abs();
        self.max_abs_dx = self.max_abs_dx.max(dx);
        self.max_abs_dy = self.max_abs_dy.max(dy);
        if !self.is_scroll && self.max_abs_dy > scroll_slop && self.max_abs_dy >= self.max_abs_dx {
            self.is_scroll = true;
        }
    }

    fn is_drag(self, scroll_slop: f32) -> bool {
        self.is_scroll || self.max_abs_dx > scroll_slop || self.max_abs_dy > scroll_slop
    }
}

fn pointer_pos(pointer: &dioxus_elements::event::PointerPayload) -> (f32, f32) {
    if pointer.has_window_position() {
        (pointer.window_x, pointer.window_y)
    } else {
        (pointer.x, pointer.y)
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

fn dispatch_notice(
    callbacks: &TerminalCallbackSlots,
    controller: &TerminalController,
    notice: crate::worker::UiNotice,
) {
    if let Some(error) = notice.error {
        if let Some(handler) = callbacks.on_error.get() {
            handler.call(error);
        }
    }
    let effects = notice.effects;
    if !effects.write_pty.is_empty() {
        if let Some(handler) = callbacks.on_write_pty.get() {
            handler.call(effects.write_pty);
        }
    }
    if effects.bell {
        if let Some(handler) = callbacks.on_bell.get() {
            handler.call(());
        }
    }
    if let Some(title) = effects.title {
        if let Some(handler) = callbacks.on_title.get() {
            handler.call(title);
        }
    }
    if let Some(pwd) = effects.pwd {
        if let Some(handler) = callbacks.on_pwd.get() {
            handler.call(pwd);
        }
    }
    if let Some(handler) = callbacks.on_frame.get() {
        handler.call(controller.frame());
    }
}
