use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::native_surface::NativeSurface;
use crate::renderer::TerminalRenderer;
use crate::surface::TerminalSurfaceMetrics;
use crate::TerminalFrame;

pub(crate) struct RenderPacket {
    pub(crate) frame: TerminalFrame,
    pub(crate) metrics: TerminalSurfaceMetrics,
    pub(crate) cursor_phase: bool,
    pub(crate) cursor_blink: bool,
    pub(crate) background_color: u32,
}

pub(crate) enum WorkerMessage {
    SurfaceAvailable(NativeSurface),
    SurfaceLost,
    RenderReady,
    Shutdown,
}

pub(crate) struct WorkerHandle {
    sender: Sender<WorkerMessage>,
    latest: Arc<Mutex<Option<RenderPacket>>>,
    render_pending: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    pub(crate) fn spawn() -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let latest = Arc::new(Mutex::new(None));
        let render_pending = Arc::new(AtomicBool::new(false));
        let worker_latest = latest.clone();
        let worker_pending = render_pending.clone();
        let worker_sender = sender.clone();
        let thread = thread::Builder::new()
            .name("arkit-terminal".into())
            .spawn(move || {
                run_worker(receiver, worker_sender, worker_latest, worker_pending);
            })?;
        Ok(Self {
            sender,
            latest,
            render_pending,
            thread: Some(thread),
        })
    }

    pub(crate) fn sender(&self) -> Sender<WorkerMessage> {
        self.sender.clone()
    }

    /// Replace any frame not yet consumed by the renderer.
    ///
    /// Ghostty may publish more viewport snapshots than the native window can
    /// present. Keeping exactly the newest one prevents stale scroll frames
    /// from becoming visible after the finger has already moved on.
    pub(crate) fn publish(&self, packet: RenderPacket) {
        match self.latest.lock() {
            Ok(mut latest) => *latest = Some(packet),
            Err(poisoned) => *poisoned.into_inner() = Some(packet),
        }
        schedule_render(&self.sender, &self.render_pending);
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(WorkerMessage::Shutdown);
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                ohos_hilog_binding::error("arkit_terminal: render worker panicked");
            }
        }
    }
}

fn run_worker(
    receiver: Receiver<WorkerMessage>,
    sender: Sender<WorkerMessage>,
    latest: Arc<Mutex<Option<RenderPacket>>>,
    render_pending: Arc<AtomicBool>,
) {
    let mut renderer: Option<TerminalRenderer> = None;
    let mut current = None;

    while let Ok(message) = receiver.recv() {
        match message {
            WorkerMessage::SurfaceAvailable(window) => {
                let result = match renderer.as_mut() {
                    Some(renderer) => renderer.bind_surface(window),
                    None => TerminalRenderer::new(window).map(|created| {
                        renderer = Some(created);
                    }),
                };
                if let Err(error) = result {
                    ohos_hilog_binding::error(format!(
                        "arkit_terminal: failed to bind GPU surface: {error}"
                    ));
                    if let Some(renderer) = renderer.as_mut() {
                        renderer.unbind_surface();
                    }
                    continue;
                }
                render_current(renderer.as_mut(), current.as_ref());
            }
            WorkerMessage::SurfaceLost => {
                if let Some(renderer) = renderer.as_mut() {
                    renderer.unbind_surface();
                }
            }
            WorkerMessage::RenderReady => {
                current = take_latest(&latest).or(current);
                render_current(renderer.as_mut(), current.as_ref());

                // Clear after presentation. If a producer replaced the frame
                // while the swapchain was blocked, schedule one new message
                // behind any pending surface lifecycle messages.
                render_pending.store(false, Ordering::Release);
                if has_latest(&latest) {
                    schedule_render(&sender, &render_pending);
                }
            }
            WorkerMessage::Shutdown => break,
        }
    }
}

fn render_current(renderer: Option<&mut TerminalRenderer>, packet: Option<&RenderPacket>) {
    let (Some(renderer), Some(packet)) = (renderer, packet) else {
        return;
    };
    if let Err(error) = renderer.render(packet) {
        ohos_hilog_binding::error(format!("arkit_terminal: native render failed: {error}"));
    }
}

fn take_latest(latest: &Mutex<Option<RenderPacket>>) -> Option<RenderPacket> {
    match latest.lock() {
        Ok(mut latest) => latest.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    }
}

fn has_latest(latest: &Mutex<Option<RenderPacket>>) -> bool {
    match latest.lock() {
        Ok(latest) => latest.is_some(),
        Err(poisoned) => poisoned.into_inner().is_some(),
    }
}

fn schedule_render(sender: &Sender<WorkerMessage>, pending: &AtomicBool) {
    if pending
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
        && sender.send(WorkerMessage::RenderReady).is_err()
    {
        pending.store(false, Ordering::Release);
    }
}
