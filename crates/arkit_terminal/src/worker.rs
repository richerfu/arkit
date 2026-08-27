use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use ohos_vsync_binding::{OH_NativeVSync_ExpectedRateRange, Vsync};

use crate::config::{TerminalConfig, TerminalEffects};
use crate::engine::TerminalEngine;
use crate::error::{TerminalError, TerminalErrorKind};
use crate::frame::TerminalFrame;
use crate::native_surface::NativeSurface;
use crate::renderer::TerminalRenderer;
use crate::surface::TerminalSurfaceMetrics;

const BLINK_NS: u64 = 530_000_000;
const VSYNC_MIN_FPS: i32 = 30;
const VSYNC_MAX_FPS: i32 = 120;
const VSYNC_EXPECTED_FPS: i32 = 120;

struct WindowVsync {
    handle: Vsync,
    generation: u64,
    requested: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct PaintSettings {
    pub(crate) metrics: TerminalSurfaceMetrics,
    pub(crate) cursor_blink: bool,
    pub(crate) background_color: u32,
}

impl Default for PaintSettings {
    fn default() -> Self {
        Self {
            metrics: TerminalSurfaceMetrics::fallback(1.0),
            cursor_blink: true,
            background_color: 0xFF0B_1220,
        }
    }
}

pub(crate) struct RenderPacket<'a> {
    pub(crate) frame: &'a TerminalFrame,
    pub(crate) metrics: TerminalSurfaceMetrics,
    pub(crate) cursor_phase: bool,
    pub(crate) cursor_blink: bool,
    pub(crate) background_color: u32,
}

pub(crate) struct UiNotice {
    pub(crate) effects: TerminalEffects,
    pub(crate) error: Option<TerminalError>,
}

pub(crate) enum WorkerMessage {
    SurfaceAvailable(NativeSurface),
    SurfaceLost,
    VSync {
        generation: u64,
        timestamp: u64,
    },
    Attach {
        config: TerminalConfig,
        initial: Option<String>,
    },
    Reconfigure(TerminalConfig),
    ScrollToBottom,
    ScrollToTop,
    ScrollToRow(u64),
    Wake,
    Shutdown,
}

/// Bytes, encoder snapshot, and the last painted frame. Host I/O only
/// appends here; the render worker owns rio-vt and wgpu.
pub(crate) struct TerminalShared {
    pending: Mutex<Vec<u8>>,
    dirty: AtomicBool,
    vsync_live: AtomicBool,
    wake_pending: AtomicBool,
    encode_bits: AtomicU32,
    cell_width_px: AtomicU32,
    cell_height_px: AtomicU32,
    scroll_delta: AtomicI64,
    generation: AtomicU64,
    last_frame: Mutex<TerminalFrame>,
    paint: Mutex<PaintSettings>,
    control: Mutex<Option<Sender<WorkerMessage>>>,
}

impl TerminalShared {
    pub(crate) fn new() -> Self {
        Self {
            pending: Mutex::new(Vec::new()),
            dirty: AtomicBool::new(false),
            vsync_live: AtomicBool::new(false),
            wake_pending: AtomicBool::new(false),
            encode_bits: AtomicU32::new(0),
            cell_width_px: AtomicU32::new(8),
            cell_height_px: AtomicU32::new(18),
            scroll_delta: AtomicI64::new(0),
            generation: AtomicU64::new(0),
            last_frame: Mutex::new(TerminalFrame::default()),
            paint: Mutex::new(PaintSettings::default()),
            control: Mutex::new(None),
        }
    }

    pub(crate) fn set_control(&self, sender: Option<Sender<WorkerMessage>>) {
        *lock_mutex(&self.control) = sender;
    }

    pub(crate) fn send(&self, message: WorkerMessage) {
        if let Some(sender) = lock_mutex(&self.control).as_ref() {
            let _ = sender.send(message);
        }
    }

    pub(crate) fn request_draw(&self) {
        if self.vsync_live() {
            return;
        }
        let sender = lock_mutex(&self.control);
        let Some(sender) = sender.as_ref() else {
            return;
        };
        if self
            .wake_pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            && sender.send(WorkerMessage::Wake).is_err()
        {
            self.wake_pending.store(false, Ordering::Release);
        }
    }

    pub(crate) fn push_bytes(&self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        lock_mutex(&self.pending).extend_from_slice(data);
        self.dirty.store(true, Ordering::Release);
        self.request_draw();
    }

    pub(crate) fn take_bytes(&self) -> Vec<u8> {
        std::mem::take(&mut *lock_mutex(&self.pending))
    }

    pub(crate) fn clear_pending(&self) {
        lock_mutex(&self.pending).clear();
        self.scroll_delta.store(0, Ordering::Release);
    }

    pub(crate) fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
    }

    pub(crate) fn vsync_live(&self) -> bool {
        self.vsync_live.load(Ordering::Acquire)
    }

    pub(crate) fn encode_bits(&self) -> u32 {
        self.encode_bits.load(Ordering::Acquire)
    }

    pub(crate) fn set_encode_bits(&self, bits: u32) {
        self.encode_bits.store(bits, Ordering::Release);
    }

    pub(crate) fn cell_width_px(&self) -> u32 {
        self.cell_width_px.load(Ordering::Acquire).max(1)
    }

    pub(crate) fn cell_height_px(&self) -> u32 {
        self.cell_height_px.load(Ordering::Acquire).max(1)
    }

    pub(crate) fn set_cell_metrics(&self, width_px: u32, height_px: u32) {
        self.cell_width_px.store(width_px.max(1), Ordering::Release);
        self.cell_height_px
            .store(height_px.max(1), Ordering::Release);
    }

    pub(crate) fn add_scroll_delta(&self, delta_rows: i64) {
        if delta_rows == 0 {
            return;
        }
        self.scroll_delta.fetch_add(delta_rows, Ordering::AcqRel);
        self.dirty.store(true, Ordering::Release);
    }

    pub(crate) fn take_scroll_delta(&self) -> i64 {
        self.scroll_delta.swap(0, Ordering::AcqRel)
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub(crate) fn last_frame(&self) -> TerminalFrame {
        lock_mutex(&self.last_frame).clone()
    }

    pub(crate) fn set_paint(&self, settings: PaintSettings) {
        *lock_mutex(&self.paint) = settings;
        self.set_cell_metrics(
            settings.metrics.native_cell_width_px(),
            settings.metrics.native_cell_height_px(),
        );
        self.dirty.store(true, Ordering::Release);
    }

    fn paint(&self) -> PaintSettings {
        *lock_mutex(&self.paint)
    }

    fn store_frame(&self, frame: TerminalFrame) {
        *lock_mutex(&self.last_frame) = frame;
        self.generation.fetch_add(1, Ordering::AcqRel);
    }

    fn has_pending_work(&self) -> bool {
        self.dirty.load(Ordering::Acquire)
            || self.scroll_delta.load(Ordering::Acquire) != 0
            || !lock_mutex(&self.pending).is_empty()
    }
}

pub(crate) struct WorkerHandle {
    sender: Sender<WorkerMessage>,
    shared: Arc<TerminalShared>,
    notices: Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<UiNotice>>>,
    thread: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    pub(crate) fn spawn(shared: Arc<TerminalShared>) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let (notice_tx, notice_rx) = tokio::sync::mpsc::unbounded_channel();
        let worker_shared = shared.clone();
        let worker_sender = sender.clone();
        shared.set_control(Some(sender.clone()));
        let thread = thread::Builder::new()
            .name("arkit-terminal".into())
            .spawn(move || {
                run_worker(receiver, worker_sender, worker_shared, notice_tx);
            })?;
        if shared.dirty.load(Ordering::Acquire) {
            shared.request_draw();
        }
        Ok(Self {
            sender,
            shared,
            notices: Mutex::new(Some(notice_rx)),
            thread: Some(thread),
        })
    }

    pub(crate) fn sender(&self) -> Sender<WorkerMessage> {
        self.sender.clone()
    }

    pub(crate) fn shared(&self) -> &Arc<TerminalShared> {
        &self.shared
    }

    pub(crate) fn take_notices(&self) -> Option<tokio::sync::mpsc::UnboundedReceiver<UiNotice>> {
        lock_mutex(&self.notices).take()
    }

    pub(crate) fn request_draw(&self) {
        self.shared.request_draw();
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(WorkerMessage::Shutdown);
        self.shared.set_control(None);
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                ohos_hilog_binding::error("arkit_terminal: render worker panicked");
            }
        }
    }
}

fn schedule_vsync(sender: &Sender<WorkerMessage>, generation: u64, timestamp: u64) {
    let _ = sender.send(WorkerMessage::VSync {
        generation,
        timestamp,
    });
}

fn run_worker(
    receiver: Receiver<WorkerMessage>,
    sender: Sender<WorkerMessage>,
    shared: Arc<TerminalShared>,
    notices: tokio::sync::mpsc::UnboundedSender<UiNotice>,
) {
    let mut state = WorkerState {
        engine: None,
        renderer: None,
        vsync: None,
        next_vsync_generation: 1,
        sender,
        shared,
        notices,
        last_phase: true,
    };

    while let Ok(message) = receiver.recv() {
        if matches!(message, WorkerMessage::Shutdown) {
            break;
        }
        let mut vsync_ts = None;
        let mut present = false;
        if !dispatch(&mut state, message, &mut vsync_ts, &mut present) {
            break;
        }
        while let Ok(more) = receiver.try_recv() {
            if matches!(more, WorkerMessage::Shutdown) {
                return;
            }
            if !dispatch(&mut state, more, &mut vsync_ts, &mut present) {
                return;
            }
        }
        let received_vsync = vsync_ts.is_some();
        if received_vsync || present {
            pump(&mut state, vsync_ts);
            if received_vsync {
                state.request_next_vsync();
            }
        }
    }
}

struct WorkerState {
    engine: Option<TerminalEngine>,
    renderer: Option<TerminalRenderer>,
    vsync: Option<WindowVsync>,
    next_vsync_generation: u64,
    sender: Sender<WorkerMessage>,
    shared: Arc<TerminalShared>,
    notices: tokio::sync::mpsc::UnboundedSender<UiNotice>,
    last_phase: bool,
}

fn dispatch(
    state: &mut WorkerState,
    message: WorkerMessage,
    vsync_ts: &mut Option<u64>,
    present: &mut bool,
) -> bool {
    match message {
        WorkerMessage::SurfaceAvailable(window) => {
            let surface_id = window.surface_id();
            let result = match state.renderer.as_mut() {
                Some(renderer) => renderer.bind_surface(window),
                None => TerminalRenderer::new(window).map(|created| {
                    state.renderer = Some(created);
                }),
            };
            if let Err(error) = result {
                ohos_hilog_binding::error(format!(
                    "arkit_terminal: failed to bind GPU surface: {error}"
                ));
                if let Some(renderer) = state.renderer.as_mut() {
                    renderer.unbind_surface();
                }
                state.stop_vsync();
                return true;
            }
            state.bind_vsync(surface_id);
            *vsync_ts = None;
            state.shared.mark_dirty();
            *present = true;
        }
        WorkerMessage::SurfaceLost => {
            if let Some(renderer) = state.renderer.as_mut() {
                renderer.unbind_surface();
            }
            state.stop_vsync();
            *vsync_ts = None;
        }
        WorkerMessage::VSync {
            generation,
            timestamp,
        } => {
            if state.accept_vsync(generation) {
                state.shared.vsync_live.store(true, Ordering::Release);
                *vsync_ts = Some(timestamp);
            }
        }
        WorkerMessage::Attach { config, initial } => match TerminalEngine::with_config(config) {
            Ok(mut engine) => {
                if let Some(banner) = initial {
                    engine.write_str(&banner);
                }
                engine.write_str("\x1b[?25h");
                sync_encoder(&state.shared, &engine);
                state.engine = Some(engine);
                state.shared.mark_dirty();
                *present = true;
            }
            Err(error) => {
                let _ = state.notices.send(UiNotice {
                    effects: TerminalEffects::default(),
                    error: Some(error),
                });
            }
        },
        WorkerMessage::Reconfigure(config) => {
            state
                .shared
                .set_cell_metrics(config.cell_width_px, config.cell_height_px);
            if let Some(engine) = state.engine.as_mut() {
                if let Err(error) = engine.reconfigure(config) {
                    let _ = state.notices.send(UiNotice {
                        effects: TerminalEffects::default(),
                        error: Some(error),
                    });
                } else {
                    sync_encoder(&state.shared, engine);
                    state.shared.mark_dirty();
                    *present = true;
                }
            }
        }
        WorkerMessage::ScrollToBottom => {
            if let Some(engine) = state.engine.as_mut() {
                engine.scroll_to_bottom();
                state.shared.mark_dirty();
                *present = true;
            }
        }
        WorkerMessage::ScrollToTop => {
            if let Some(engine) = state.engine.as_mut() {
                engine.scroll_to_top();
                state.shared.mark_dirty();
                *present = true;
            }
        }
        WorkerMessage::ScrollToRow(row) => {
            if let Some(engine) = state.engine.as_mut() {
                engine.scroll_to_row(row);
                state.shared.mark_dirty();
                *present = true;
            }
        }
        WorkerMessage::Wake => {
            state.shared.wake_pending.store(false, Ordering::Release);
            *present = true;
        }
        WorkerMessage::Shutdown => return false,
    }
    true
}

impl WorkerState {
    fn bind_vsync(&mut self, surface_id: Option<u64>) {
        self.stop_vsync();
        let Some(surface_id) = surface_id else {
            return;
        };
        let Some(handle) = Vsync::try_new_for_associated_window(surface_id, "arkit-terminal")
        else {
            ohos_hilog_binding::error(format!(
                "arkit_terminal: failed to create vsync for surface {surface_id}"
            ));
            return;
        };
        let rate_result = handle.set_expected_frame_rate_range(OH_NativeVSync_ExpectedRateRange {
            min: VSYNC_MIN_FPS,
            max: VSYNC_MAX_FPS,
            expected: VSYNC_EXPECTED_FPS,
        });
        if rate_result != 0 {
            ohos_hilog_binding::error(format!(
                "arkit_terminal: failed to set vsync frame rate: {rate_result}"
            ));
        }
        let generation = self.next_vsync_generation;
        self.next_vsync_generation = generation.wrapping_add(1);
        self.vsync = Some(WindowVsync {
            handle,
            generation,
            requested: false,
        });
        self.request_next_vsync();
    }

    fn stop_vsync(&mut self) {
        self.vsync.take();
        self.shared.vsync_live.store(false, Ordering::Release);
    }

    fn accept_vsync(&mut self, generation: u64) -> bool {
        let Some(vsync) = self.vsync.as_mut() else {
            return false;
        };
        if generation != vsync.generation {
            return false;
        }
        vsync.requested = false;
        true
    }

    fn request_next_vsync(&mut self) {
        let Some(vsync) = self.vsync.as_mut() else {
            return;
        };
        if vsync.requested {
            return;
        }
        let sender = self.sender.clone();
        let generation = vsync.generation;
        let result = vsync.handle.request_frame_once(move |timestamp| {
            let timestamp = u64::try_from(timestamp).unwrap_or(0);
            schedule_vsync(&sender, generation, timestamp);
        });
        if result != 0 {
            ohos_hilog_binding::error(format!(
                "arkit_terminal: failed to request vsync frame: {result}"
            ));
            self.stop_vsync();
        } else if let Some(vsync) = self.vsync.as_mut() {
            vsync.requested = true;
        }
    }
}

fn pump(state: &mut WorkerState, vsync_ts: Option<u64>) {
    drain_vt(state);
    let paint = state.shared.paint();
    let phase = cursor_phase(vsync_ts, state.last_phase);
    let blink_changed = paint.cursor_blink && phase != state.last_phase;
    state.last_phase = phase;
    let dirty = state.shared.dirty.swap(false, Ordering::AcqRel);
    if !dirty && !blink_changed {
        return;
    }
    present(state, &paint, phase);
    if !state.shared.vsync_live() && state.shared.has_pending_work() {
        state.shared.request_draw();
    }
}

fn drain_vt(state: &mut WorkerState) {
    let bytes = state.shared.take_bytes();
    let scroll = state.shared.take_scroll_delta();
    let Some(engine) = state.engine.as_mut() else {
        if !bytes.is_empty() {
            state.shared.push_bytes(&bytes);
        }
        if scroll != 0 {
            state.shared.add_scroll_delta(scroll);
        }
        return;
    };
    if !bytes.is_empty() {
        engine.write_bytes(&bytes);
    }
    if scroll != 0 {
        engine.scroll_by(scroll);
    }
    if bytes.is_empty() && scroll == 0 {
        return;
    }
    sync_encoder(&state.shared, engine);
    let effects = engine.take_effects();
    if !effects.is_empty() {
        let _ = state.notices.send(UiNotice {
            effects,
            error: None,
        });
    }
}

fn present(state: &mut WorkerState, paint: &PaintSettings, phase: bool) {
    let Some(engine) = state.engine.as_mut() else {
        return;
    };
    let frame = match engine.capture() {
        Ok(frame) => frame,
        Err(error) => {
            let _ = state.notices.send(UiNotice {
                effects: TerminalEffects::default(),
                error: Some(error),
            });
            return;
        }
    };
    if let Some(renderer) = state.renderer.as_mut() {
        let packet = RenderPacket {
            frame: &frame,
            metrics: paint.metrics,
            cursor_phase: phase,
            cursor_blink: paint.cursor_blink,
            background_color: paint.background_color,
        };
        if let Err(error) = renderer.render(&packet) {
            ohos_hilog_binding::error(format!("arkit_terminal: native render failed: {error}"));
            let _ = state.notices.send(UiNotice {
                effects: TerminalEffects::default(),
                error: Some(TerminalError::new(TerminalErrorKind::Io, error)),
            });
        }
    }
    state.shared.store_frame(frame);
}

fn sync_encoder(shared: &TerminalShared, engine: &TerminalEngine) {
    shared.set_encode_bits(engine.encode_state_bits());
    let config = engine.config();
    shared.set_cell_metrics(config.cell_width_px, config.cell_height_px);
}

fn cursor_phase(vsync_ts: Option<u64>, fallback: bool) -> bool {
    match vsync_ts {
        Some(0) | None => fallback,
        Some(timestamp) => (timestamp / BLINK_NS).is_multiple_of(2),
    }
}

fn lock_mutex<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mailbox_appends_without_an_engine() {
        let shared = TerminalShared::new();
        shared.push_bytes(b"abc");
        shared.push_bytes(b"def");
        assert_eq!(shared.take_bytes(), b"abcdef");
        assert!(shared.take_bytes().is_empty());
    }

    #[test]
    fn scroll_delta_coalesces() {
        let shared = TerminalShared::new();
        shared.add_scroll_delta(-3);
        shared.add_scroll_delta(-1);
        assert_eq!(shared.take_scroll_delta(), -4);
        assert_eq!(shared.take_scroll_delta(), 0);
    }
}
