//! Finite host-output workloads for measuring the production renderer on-device.
//! These bytes bypass PTY allocation; the UI labels that distinction explicitly.

use std::fmt::Write;
use std::time::Duration;

use arkit::TerminalInbox;
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct FrameProbe {
    task: JoinHandle<()>,
}

impl FrameProbe {
    pub fn start(
        runtime: &Handle,
        inbox: TerminalInbox,
        cols: u16,
        rows: u16,
    ) -> (Self, mpsc::UnboundedReceiver<String>) {
        let (status, events) = mpsc::unbounded_channel();
        let task = runtime.spawn(async move {
            for phase in ["ASCII scroll", "CJK scroll", "styled redraw"] {
                let _ = status.send(format!("render probe · {phase} · no PTY"));
                inbox.push(b"\x1b[2J\x1b[H\x1b[?25l");
                let deadline = tokio::time::Instant::now() + Duration::from_secs(8);
                let mut ticker = tokio::time::interval(Duration::from_millis(2));
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                let mut sequence = 0u32;
                while tokio::time::Instant::now() < deadline {
                    ticker.tick().await;
                    let output = Self::output(phase, sequence, cols, rows);
                    inbox.push(output.as_bytes());
                    sequence += 1;
                }
            }
            inbox.push(b"\x1b[0m\x1b[?25h\r\nrender probe complete (no PTY)\r\n");
            let _ = status.send("render probe complete · no PTY · measure FPS externally".into());
        });
        (Self { task }, events)
    }

    fn output(phase: &str, sequence: u32, cols: u16, rows: u16) -> String {
        match phase {
            "ASCII scroll" => format!("{sequence:06} 0123456789 abcdefghijklmnopqrstuvwxyz\r\n"),
            "CJK scroll" => format!("{sequence:06} 中文终端 流畅滚动 e\u{301} 🙂 🚀\r\n"),
            _ => {
                let mut output = String::from("\x1b[H");
                for row in 0..rows {
                    let color = 31 + (u32::from(row) + sequence) % 6;
                    let _ = write!(output, "\x1b[{};1H\x1b[{color}m", row + 1);
                    for col in 0..cols {
                        output.push(char::from(b'A' + ((u32::from(col) + sequence) % 26) as u8));
                    }
                }
                output.push_str("\x1b[0m");
                output
            }
        }
    }
}

impl Drop for FrameProbe {
    fn drop(&mut self) {
        self.task.abort();
    }
}
