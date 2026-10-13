//! Terminal demo — component paints; this app owns demo, real PTY and SSH I/O.
//!
//! ```text
//!   Terminal.on_input  ──►  Demo | PTY | SSH write
//!   Demo/PTY/SSH out   ──►  controller.feed_vt
//!   Terminal.on_write_pty ──►  same host write
//!   Arkit layout ──► terminal config + host PTY resize
//! ```

mod demos;
mod local_pty;
mod local_shell;
mod ssh_host;

use std::cell::RefCell;
use std::rc::Rc;

use arkit::prelude::*;
use tokio::sync::mpsc;

use local_pty::{LocalPty, PtyEvent};
use local_shell::LocalShell;
use ssh_host::{spawn_ssh, SshCmd, SshConnect, SshEvent};

const COLS: u16 = 40;
// Initial geometry only, before Arkit delivers the measured layout.
const ROWS: u16 = 24;
const CELL_WIDTH_VP: f32 = 10.0;
const CELL_HEIGHT_VP: f32 = 20.0;
const TERMINAL_PADDING_VP: f32 = 4.0;
const IME_TOP_GUTTER: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum HostMode {
    Local,
    Pty,
    Ssh,
}

struct HostState {
    mode: HostMode,
    local: LocalShell,
    ssh_tx: Option<mpsc::UnboundedSender<SshCmd>>,
    pty: Option<LocalPty>,
    pty_rx: Option<std::sync::mpsc::Receiver<PtyEvent>>,
    pty_probe: Option<Vec<u8>>,
}

impl HostState {
    fn new() -> Self {
        Self {
            mode: HostMode::Local,
            local: LocalShell::new(COLS, ROWS),
            ssh_tx: None,
            pty: None,
            pty_rx: None,
            pty_probe: None,
        }
    }

    /// Keyboard / encoded input from the UI → host.
    /// Local: returns VT to feed; SSH: async remote echo via event channel.
    fn write_input(&mut self, data: &[u8]) -> Vec<u8> {
        match self.mode {
            HostMode::Local => self.local.input(data),
            HostMode::Pty => {
                match self
                    .pty
                    .as_ref()
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::BrokenPipe, "PTY exited")
                    })
                    .and_then(|pty| pty.write(data))
                {
                    Ok(()) => Vec::new(),
                    Err(error) => format!("\r\n[PTY write: {error}]\r\n").into_bytes(),
                }
            }
            HostMode::Ssh => {
                if let Some(tx) = &self.ssh_tx {
                    let _ = tx.send(SshCmd::Data(data.to_vec()));
                }
                Vec::new()
            }
        }
    }

    /// Terminal → host (`write_pty` effect: DA/DSR/…). Not line-edited.
    fn write_pty_reply(&mut self, data: &[u8]) {
        match self.mode {
            HostMode::Local => {
                // No real PTY — local shell ignores device replies.
            }
            HostMode::Pty => {
                if let Some(pty) = &self.pty {
                    let _ = pty.write(data);
                }
            }
            HostMode::Ssh => {
                if let Some(tx) = &self.ssh_tx {
                    let _ = tx.send(SshCmd::Data(data.to_vec()));
                }
            }
        }
        let _ = data;
    }

    fn disconnect_to_local(&mut self) -> Vec<u8> {
        self.pty.take();
        self.pty_rx.take();
        self.pty_probe.take();
        if let Some(tx) = self.ssh_tx.take() {
            let _ = tx.send(SshCmd::Close);
        }
        self.mode = HostMode::Local;
        self.local.reset();
        let mut out = b"\r\n\x1b[33m[disconnected - local shell]\x1b[0m\r\n".to_vec();
        out.extend(self.local.prompt());
        out
    }

    fn connect_pty(&mut self, cols: u16, rows: u16) -> std::io::Result<()> {
        self.disconnect_to_local();
        self.mode = HostMode::Pty;
        let (pty, rx) = LocalPty::start(cols, rows)?;
        self.pty = Some(pty);
        self.pty_rx = Some(rx);
        Ok(())
    }

    fn take_pty_events(&mut self) -> Vec<PtyEvent> {
        self.pty_rx
            .as_ref()
            .map(|rx| rx.try_iter().take(8).collect())
            .unwrap_or_default()
    }

    fn observe_pty_probe(&mut self, bytes: &[u8]) -> bool {
        let Some(probe) = &mut self.pty_probe else {
            return false;
        };
        probe.extend_from_slice(bytes);
        if probe
            .windows(b"ARKIT_REAL_PTY".len())
            .any(|part| part == b"ARKIT_REAL_PTY")
        {
            self.pty_probe = None;
            return true;
        }
        if probe.len() > 128 {
            probe.drain(..probe.len() - 128);
        }
        false
    }
}

#[component]
pub fn TerminalPage() -> Element {
    let async_runtime = use_runtime_handle().tokio();
    let controller = use_hook(TerminalController::new);
    let host = use_hook(|| Rc::new(RefCell::new(HostState::new())));
    let window_metrics = use_window_metrics();
    let mut status = use_signal(|| String::from("host=local"));
    let mut title = use_signal(|| String::from("(title)"));
    let mut bell_n = use_signal(|| 0u32);
    let grid = use_signal(|| (COLS, ROWS));
    let terminal_ref = use_native_element_ref();
    use_layout_size(terminal_ref.clone(), {
        let host = host.clone();
        move |size| {
            let mut grid = grid;
            let mut status = status;
            let scale = window_metrics.scale.max(f32::EPSILON);
            let cols = ((size.width / scale - TERMINAL_PADDING_VP * 2.0) / CELL_WIDTH_VP)
                .floor()
                .clamp(1.0, u16::MAX as f32) as u16;
            let rows = ((size.height / scale - TERMINAL_PADDING_VP * 2.0) / CELL_HEIGHT_VP)
                .floor()
                .clamp(1.0, u16::MAX as f32) as u16;
            if *grid.peek() == (cols, rows) {
                return;
            }
            let mut host = host.borrow_mut();
            host.local.cols = cols;
            host.local.rows = rows;
            if let Some(pty) = &host.pty {
                if let Err(error) = pty.resize(cols, rows) {
                    status.set(format!("PTY resize failed: {error}"));
                }
            }
            if let Some(tx) = &host.ssh_tx {
                let _ = tx.send(SshCmd::Resize(cols, rows));
            }
            grid.set((cols, rows));
        }
    });

    let mut ssh_host = use_signal(|| String::from("127.0.0.1"));
    let mut ssh_port = use_signal(|| String::from("22"));
    let mut ssh_user = use_signal(|| String::from("root"));
    let mut ssh_pass = use_signal(String::new);

    // SSH inbound channel: task → UI poll.
    let (ssh_ev_tx, ssh_ev_rx) = use_hook(|| {
        let (tx, rx) = mpsc::unbounded_channel::<SshEvent>();
        (tx, Rc::new(RefCell::new(rx)))
    });

    // Drain SSH events into the terminal on a short timer.
    use_hook({
        let controller = controller.clone();
        let host = host.clone();
        let ssh_ev_rx = ssh_ev_rx.clone();
        let mut status = status;
        let handle = async_runtime.clone();
        move || {
            dioxus_core::spawn(async move {
                loop {
                    let sleeper = handle.spawn(async {
                        tokio::time::sleep(std::time::Duration::from_millis(32)).await;
                    });
                    let _ = sleeper.await;
                    let pty_events = host.borrow_mut().take_pty_events();
                    for event in pty_events {
                        match event {
                            PtyEvent::Started(pid) => status.set(format!("host=pty pid={pid}")),
                            PtyEvent::Output(bytes) => {
                                let passed = host.borrow_mut().observe_pty_probe(&bytes);
                                controller.feed_vt(&bytes);
                                if passed {
                                    status.set("PTY probe passed · real shell output".into());
                                }
                            }
                            PtyEvent::Closed(reason) => {
                                host.borrow_mut().pty.take();
                                host.borrow_mut().pty_rx.take();
                                status.set(format!("PTY exited · {reason}"));
                            }
                        }
                    }
                    let mut rx = ssh_ev_rx.borrow_mut();
                    while let Ok(ev) = rx.try_recv() {
                        if host.borrow().ssh_tx.is_none() {
                            continue;
                        }
                        match ev {
                            SshEvent::Connected => {
                                host.borrow_mut().mode = HostMode::Ssh;
                                status.set("host=ssh connected".into());
                            }
                            SshEvent::Output(bytes) => {
                                controller.feed_vt(&bytes);
                            }
                            SshEvent::Status(s) => status.set(s),
                            SshEvent::Closed(msg) => {
                                let vt = host.borrow_mut().disconnect_to_local();
                                controller.feed_vt(&vt);
                                status.set(msg);
                            }
                        }
                    }
                }
            });
        }
    });

    // Initial local banner after mount — feed as host output.
    use_hook({
        let controller = controller.clone();
        let host = host.clone();
        let handle = async_runtime.clone();
        move || {
            let banner = host.borrow().local.banner();
            // Defer slightly so Terminal has attached the engine.
            dioxus_core::spawn(async move {
                let sleeper = handle.spawn(async {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                });
                let _ = sleeper.await;
                controller.feed_vt(&banner);
            });
        }
    });

    let config = TerminalConfig::default()
        .with_size(grid().0, grid().1)
        .with_scrollback(5_000)
        // Keep in sync with arkit_terminal cell metrics (glyph must fit cell).
        .with_cell_metrics(
            (CELL_WIDTH_VP * window_metrics.scale).round().max(1.0) as u32,
            (CELL_HEIGHT_VP * window_metrics.scale).round().max(1.0) as u32,
        )
        .with_theme(
            Rgb::new(0xE2, 0xE8, 0xF0),
            Rgb::new(0x0B, 0x12, 0x20),
            Rgb::new(0x38, 0xBD, 0xF8),
        )
        .with_cursor_style(CursorVisualStyle::Block, true);

    let write_vt = {
        let controller = controller.clone();
        move |label: &'static str, payload: &'static str| {
            let controller = controller.clone();
            let mut status = status;
            move |()| {
                // Demo chips inject VT as *host output* (like a remote wrote them).
                controller.feed_vt(payload.as_bytes());
                status.set(format!("vt · {label}"));
            }
        }
    };

    let demo_hello = {
        let controller = controller.clone();
        let host = host.clone();
        let mut status = status;
        move |()| {
            let vt = host.borrow_mut().write_input(demos::HELLO_COMMAND);
            if !vt.is_empty() {
                controller.feed_vt(&vt);
            }
            status.set("demo · hello".into());
        }
    };

    let demo_colors = {
        let controller = controller.clone();
        let host = host.clone();
        let mut status = status;
        move |()| {
            controller.feed_vt(b"\r\n");
            controller.feed_vt(demos::COLORS.as_bytes());
            controller.feed_vt(&host.borrow().local.prompt());
            status.set("demo · colors".into());
        }
    };

    let demo_clear = {
        let controller = controller.clone();
        let host = host.clone();
        let mut status = status;
        move |()| {
            host.borrow_mut().local.reset();
            controller.feed_vt(b"\x1b[2J\x1b[H\x1b[?25h");
            controller.feed_vt(&host.borrow().local.banner());
            status.set("demo · clear".into());
        }
    };

    let demo_stress = {
        let controller = controller.clone();
        let host = host.clone();
        let mut status = status;
        move |()| {
            controller.feed_vt(demos::stress_transcript(1_200).as_bytes());
            controller.feed_vt(&host.borrow().local.prompt());
            status.set("demo · stress × 1200".into());
        }
    };

    let demo_keyboard = {
        let controller = controller.clone();
        let mut status = status;
        move |()| {
            controller.show_keyboard();
            status.set("ime · keyboard".into());
        }
    };

    let key_host = {
        let controller = controller.clone();
        let host = host.clone();
        move |label: &'static str, key: &'static str| {
            let controller = controller.clone();
            let host = host.clone();
            let mut status = status;
            move |()| {
                let bytes = controller.encode_key(key);
                let vt = host.borrow_mut().write_input(&bytes);
                if !vt.is_empty() {
                    controller.feed_vt(&vt);
                }
                status.set(format!("key · {label}"));
            }
        }
    };

    let connect_ssh = {
        let host = host.clone();
        let ssh_ev_tx = ssh_ev_tx.clone();
        let mut status = status;
        let handle = async_runtime.clone();
        move |()| {
            host.borrow_mut().disconnect_to_local();
            let port: u16 = ssh_port().parse().unwrap_or(22);
            let cfg = SshConnect {
                host: ssh_host(),
                port,
                username: ssh_user(),
                password: ssh_pass(),
                cols: grid().0,
                rows: grid().1,
            };
            // Drop previous session.
            if let Some(tx) = host.borrow_mut().ssh_tx.take() {
                let _ = tx.send(SshCmd::Close);
            }
            let tx = spawn_ssh(handle.clone(), cfg, ssh_ev_tx.clone());
            host.borrow_mut().ssh_tx = Some(tx);
            status.set("ssh connecting…".into());
        }
    };

    let connect_pty = {
        let controller = controller.clone();
        let host = host.clone();
        let mut status = status;
        move |()| {
            let (cols, rows) = grid();
            let result = host.borrow_mut().connect_pty(cols, rows);
            match result {
                Ok(()) => {
                    controller.feed_vt(b"\x1b[2J\x1b[H");
                    status.set("PTY starting…".into());
                }
                Err(error) => status.set(format!("PTY open failed: {error}")),
            }
        }
    };
    let probe_pty = {
        let host = host.clone();
        let mut status = status;
        move |()| {
            let mut host = host.borrow_mut();
            if host.mode != HostMode::Pty {
                status.set("Start Local PTY first".into());
                return;
            }
            host.pty_probe = Some(Vec::new());
            let error = host.write_input(
                b"test -t 0 && tty && pwd && ls -la && printf 'PTY_PIPE_%s\\n' OK | cat && printf 'ARKIT_REAL_%s\\n' PTY\r",
            );
            if error.is_empty() {
                status.set("PTY probe waiting for shell output…".into());
            } else {
                status.set(String::from_utf8_lossy(&error).into_owned());
            }
        }
    };

    let disconnect = {
        let controller = controller.clone();
        let host = host.clone();
        let mut status = status;
        move |()| {
            let vt = host.borrow_mut().disconnect_to_local();
            controller.feed_vt(&vt);
            status.set("host=local".into());
        }
    };

    // The host window uses resize mode, so `ime_area` can be zero even while
    // the keyboard is visible. Keep the raw keyboard height as the fallback
    // and pan the demo content by exactly one IME safe area.
    let raw_ime_lift = window_metrics
        .ime_area
        .bottom
        .max(window_metrics.keyboard_height);
    let ime_lift = if raw_ime_lift.is_finite() {
        (raw_ime_lift - IME_TOP_GUTTER).max(0.0)
    } else {
        0.0
    };
    let content_position = format!("0,{}", -ime_lift);

    rsx! {
        column {
            width: "100%",
            height: "100%",
            position: content_position,
            padding_top: 48.0,
            padding_right: 16.0,
            padding_bottom: 24.0,
            padding_left: 16.0,
            background_color: "#FF0F172A",

            row {
                width: "100%",
                column {
                    layout_weight: 1.0,
                    text {
                        content: "Terminal demo · tap to type".to_string(),
                        font_size: 18.0,
                        font_weight: 600_i32,
                        font_color: "#FFF8FAFC",
                    }
                }
                { chip("Keyboard", demo_keyboard) }
            }
            text {
                content: format!("title: {} · bell: {} · {}×{} · {}", title(), bell_n(), grid().0, grid().1, status()),
                font_size: 12.0,
                font_color: "#FF94A3B8",
                margin_top: 4.0,
                margin_bottom: 8.0,
            }

            row {
                width: "100%",
                margin_bottom: 6.0,
                { chip("Hello", demo_hello) }
                { chip("Colors", demo_colors) }
                { chip("Clear", demo_clear) }
                { chip("Stress", demo_stress) }
            }

            // SSH form — host wiring owned by the example app.
            row {
                width: "100%",
                margin_bottom: 6.0,
                textinput {
                    width: "28%",
                    height: 36.0,
                    value: ssh_host(),
                    placeholder: "host".to_string(),
                    font_size: 13.0,
                    font_color: "#FFE2E8F0",
                    background_color: "#FF1E293B",
                    border_radius: 8.0,
                    padding_left: 8.0,
                    oninput: move |e| ssh_host.set(e.string_value.clone()),
                }
                textinput {
                    width: "12%",
                    height: 36.0,
                    value: ssh_port(),
                    placeholder: "port".to_string(),
                    font_size: 13.0,
                    font_color: "#FFE2E8F0",
                    background_color: "#FF1E293B",
                    border_radius: 8.0,
                    margin_left: 6.0,
                    padding_left: 8.0,
                    oninput: move |e| ssh_port.set(e.string_value.clone()),
                }
                textinput {
                    width: "18%",
                    height: 36.0,
                    value: ssh_user(),
                    placeholder: "user".to_string(),
                    font_size: 13.0,
                    font_color: "#FFE2E8F0",
                    background_color: "#FF1E293B",
                    border_radius: 8.0,
                    margin_left: 6.0,
                    padding_left: 8.0,
                    oninput: move |e| ssh_user.set(e.string_value.clone()),
                }
                textinput {
                    width: "18%",
                    height: 36.0,
                    value: ssh_pass(),
                    placeholder: "password".to_string(),
                    font_size: 13.0,
                    font_color: "#FFE2E8F0",
                    background_color: "#FF1E293B",
                    border_radius: 8.0,
                    margin_left: 6.0,
                    padding_left: 8.0,
                    oninput: move |e| ssh_pass.set(e.string_value.clone()),
                }
            }
            row {
                width: "100%",
                margin_bottom: 8.0,
                { chip("SSH connect", connect_ssh) }
                { chip("Local PTY", connect_pty) }
                { chip("PTY probe", probe_pty) }
                { chip("disconnect", disconnect) }
                { chip("local banner", {
                    let controller = controller.clone();
                    let host = host.clone();
                    let mut status = status;
                    move |()| {
                        host.borrow_mut().disconnect_to_local();
                        let b = host.borrow().local.banner();
                        controller.feed_vt(&b);
                        status.set("host=local".into());
                    }
                }) }
            }

            row {
                width: "100%",
                margin_bottom: 6.0,
                { chip("bar", write_vt("bar", "\x1b[5 q")) }
                { chip("block", write_vt("block", "\x1b[1 q")) }
                { chip("uline", write_vt("uline", "\x1b[3 q")) }
            }
            row {
                width: "100%",
                margin_bottom: 10.0,
                { chip("←", key_host("left", "arrow_left")) }
                { chip("↓", key_host("down", "arrow_down")) }
                { chip("↑", key_host("up", "arrow_up")) }
                { chip("→", key_host("right", "arrow_right")) }
                { chip("enter", key_host("enter", "enter")) }
                { chip("bs", key_host("backspace", "backspace")) }
            }

            column {
                native_ref: terminal_ref,
                width: "100%",
                layout_weight: 1.0,
                Terminal {
                config: Some(config),
                controller: Some(controller.clone()),
                // No initial VT in component — example feeds host banner after mount.
                initial: None,
                width: "100%".to_string(),
                height: "100%".to_string(),
                cursor_blink: true,
                capture_input: true,
                on_title: move |t| {
                    title.set(t);
                },
                on_bell: move |_| {
                    bell_n.set(bell_n() + 1);
                    status.set("BEL".to_string());
                },
                // Host-bound input from soft keyboard — never feed_vt directly.
                on_input: {
                    let controller = controller.clone();
                    let host = host.clone();
                    move |bytes: Vec<u8>| {
                        let vt = host.borrow_mut().write_input(&bytes);
                        if !vt.is_empty() {
                            controller.feed_vt(&vt);
                        }
                    }
                },
                // Terminal → host (query replies). Not local line-edit input.
                on_write_pty: {
                    let host = host.clone();
                    let mut status = status;
                    move |bytes: Vec<u8>| {
                        status.set(format!("write_pty {}B", bytes.len()));
                        host.borrow_mut().write_pty_reply(&bytes);
                    }
                },
                }
            }

            text {
                content: "Tap grid = keyboard · drag = scrollback · Hello/Colors/Clear/Stress match @ohos-rs/terminal".to_string(),
                font_size: 11.0,
                font_color: "#FF64748B",
                margin_top: 10.0,
            }
        }
    }
}

fn chip(label: &'static str, mut onclick: impl FnMut(()) + 'static) -> Element {
    rsx! {
        button {
            margin_right: 8.0,
            margin_bottom: 4.0,
            background_color: "#FF1E293B",
            border_radius: 8.0,
            padding_top: 8.0,
            padding_right: 12.0,
            padding_bottom: 8.0,
            padding_left: 12.0,
            onclick: move |_| onclick(()),
            text {
                content: label.to_string(),
                font_size: 13.0,
                font_color: "#FFE2E8F0",
            }
        }
    }
}
