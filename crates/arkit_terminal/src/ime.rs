//! Explicit OpenHarmony input-method session for the terminal surface.
//!
//! A terminal is not a text editor: it has no editable backing string and it
//! must decide whether a pointer gesture is a tap or a scroll before opening
//! the keyboard. Binding an invisible ArkUI `TextInput` to the whole surface
//! violates both constraints. This module talks to the native IME directly and
//! forwards only committed input to the host.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use arkit_prelude::{dioxus_core, EventHandler};
use ohos_ime_binding::{AttachOptions, KeyboardStatus, IME};

use crate::component::TerminalController;

pub(crate) struct TerminalImeSession {
    ime: IME,
    events: tokio::sync::mpsc::UnboundedSender<ImeEvent>,
    active: Arc<AtomicBool>,
    callbacks_installed: Cell<bool>,
}

impl TerminalImeSession {
    pub(crate) fn new(
        controller: TerminalController,
        on_input: Rc<Cell<Option<EventHandler<Vec<u8>>>>>,
    ) -> Self {
        let sink = Rc::new(HostInputSink {
            controller,
            on_input,
        });
        let (events, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
        let event_sink = sink.clone();
        dioxus_core::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                event_sink.handle(event);
            }
        });
        Self {
            ime: IME::new(AttachOptions::new(false)),
            events,
            active: Arc::new(AtomicBool::new(false)),
            callbacks_installed: Cell::new(false),
        }
    }

    pub(crate) fn visible_flag(&self) -> Arc<AtomicBool> {
        self.active.clone()
    }

    pub(crate) fn hide_keyboard(&self) {
        self.active.store(false, Ordering::Release);
        if let Err(error) = self.ime.try_hide_keyboard() {
            ohos_hilog_binding::error(format!("arkit_terminal: failed to hide IME: {error}"));
        }
    }

    /// Activate this terminal and show its IME if it is not already visible.
    pub(crate) fn show_keyboard(&self) {
        if self
            .active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        self.install_callbacks();
        if let Err(error) = self.ime.try_show_keyboard() {
            self.active.store(false, Ordering::Release);
            ohos_hilog_binding::error(format!("arkit_terminal: failed to show IME: {error}"));
        }
    }

    pub(crate) fn mark_backgrounded(&self) {
        self.active.store(false, Ordering::Release);
    }

    pub(crate) fn deactivate(&self) {
        self.active.store(false, Ordering::Release);
        self.ime.detach();
    }

    fn install_callbacks(&self) {
        if self.callbacks_installed.replace(true) {
            return;
        }
        let active = self.active.clone();
        let events = self.events.clone();
        self.ime.insert_text(move |text| {
            send_if_active(&active, &events, ImeEvent::Insert(text));
        });

        let active = self.active.clone();
        let events = self.events.clone();
        self.ime.on_delete(move |count| {
            send_if_active(&active, &events, ImeEvent::DeleteBackward(count));
        });

        let active = self.active.clone();
        let events = self.events.clone();
        self.ime.on_enter(move |_| {
            send_if_active(&active, &events, ImeEvent::Enter);
        });

        let active = self.active.clone();
        self.ime.on_status_change(move |status| {
            match status {
                KeyboardStatus::Show => {
                    active.store(true, Ordering::Release);
                }
                KeyboardStatus::Hide => {
                    active.store(false, Ordering::Release);
                }
                // `None` is a transient status, not proof that the visible
                // keyboard has been dismissed.
                KeyboardStatus::None => {}
            }
        });
    }
}

enum ImeEvent {
    Insert(String),
    DeleteBackward(i32),
    Enter,
}

impl Drop for TerminalImeSession {
    fn drop(&mut self) {
        self.deactivate();
    }
}

struct HostInputSink {
    controller: TerminalController,
    on_input: Rc<Cell<Option<EventHandler<Vec<u8>>>>>,
}

impl HostInputSink {
    fn handle(&self, event: ImeEvent) {
        match event {
            ImeEvent::Insert(text) => self.insert_text(&text),
            ImeEvent::DeleteBackward(count) => self.delete_backward(count),
            ImeEvent::Enter => self.enter(),
        }
    }

    fn insert_text(&self, text: &str) {
        self.emit(encode_committed_text(&self.controller, text));
    }

    fn delete_backward(&self, count: i32) {
        let count = usize::try_from(count.max(0)).unwrap_or(0);
        if count == 0 {
            return;
        }
        let key = self.controller.encode_key("backspace");
        if key.is_empty() {
            return;
        }
        self.emit(key.repeat(count));
    }

    fn enter(&self) {
        self.emit(self.controller.encode_key("enter"));
    }

    fn emit(&self, bytes: Vec<u8>) {
        if bytes.is_empty() {
            return;
        }
        if let Some(handler) = self.on_input.get() {
            handler.call(bytes);
        }
    }
}

fn send_if_active(
    active: &AtomicBool,
    events: &tokio::sync::mpsc::UnboundedSender<ImeEvent>,
    event: ImeEvent,
) {
    if !active.load(Ordering::Acquire) {
        return;
    }
    let _ = events.send(event);
}

fn encode_committed_text(controller: &TerminalController, text: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut plain = String::new();
    for ch in text.chars() {
        match ch {
            '\n' | '\r' => {
                flush_plain(controller, &mut plain, &mut out);
                out.extend(controller.encode_key("enter"));
            }
            '\u{8}' | '\u{7f}' => {
                flush_plain(controller, &mut plain, &mut out);
                out.extend(controller.encode_key("backspace"));
            }
            c if c.is_control() => {}
            c => plain.push(c),
        }
    }
    flush_plain(controller, &mut plain, &mut out);
    out
}

fn flush_plain(controller: &TerminalController, plain: &mut String, out: &mut Vec<u8>) {
    if plain.is_empty() {
        return;
    }
    out.extend(controller.encode_text(plain));
    plain.clear();
}
