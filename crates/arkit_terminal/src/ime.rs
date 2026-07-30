//! Explicit OpenHarmony input-method session for the terminal surface.
//!
//! A terminal is not a text editor: it has no editable backing string and it
//! must decide whether a pointer gesture is a tap or a scroll before opening
//! the keyboard. Binding an invisible ArkUI `TextInput` to the whole surface
//! violates both constraints. This module talks to the native IME directly and
//! forwards only committed input to the host.

use std::cell::Cell;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use arkit_prelude::EventHandler;
use ohos_ime_binding::{AttachOptions, KeyboardStatus, IME};

use crate::component::TerminalController;

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
static ACTIVE_SESSION_ID: AtomicU64 = AtomicU64::new(0);

pub(crate) struct TerminalImeSession {
    id: u64,
    ime: IME,
    sink: Rc<HostInputSink>,
    keyboard_visible: Rc<Cell<bool>>,
}

impl TerminalImeSession {
    pub(crate) fn new(
        controller: TerminalController,
        on_input: Rc<Cell<Option<EventHandler<Vec<u8>>>>>,
    ) -> Self {
        Self {
            id: next_session_id(),
            ime: IME::new(AttachOptions::new(true)),
            sink: Rc::new(HostInputSink {
                controller,
                on_input,
            }),
            keyboard_visible: Rc::new(Cell::new(false)),
        }
    }

    /// Activate this terminal and ask the already-attached IME to show.
    ///
    /// Calling this for every confirmed tap is intentional. OpenHarmony may
    /// hide the keyboard without detaching the input-method proxy; a native
    /// focus flag would remain unchanged, whereas `ShowKeyboard` is repeatable.
    pub(crate) fn show_keyboard(&self) {
        ACTIVE_SESSION_ID.store(self.id, Ordering::Release);
        // A manual Back dismissal may leave the proxy attached even though a
        // subsequent ShowKeyboard is rejected by the platform. Reattach only
        // for that hidden state; taps while already visible remain flicker-free.
        if !self.keyboard_visible.get() {
            self.ime.detach();
        }
        self.install_callbacks();
        self.ime.show_keyboard();
    }

    pub(crate) fn deactivate(&self) {
        if ACTIVE_SESSION_ID
            .compare_exchange(self.id, 0, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.keyboard_visible.set(false);
            self.ime.detach();
        }
    }

    fn install_callbacks(&self) {
        let session_id = self.id;
        let sink = Rc::downgrade(&self.sink);
        self.ime.insert_text(move |text| {
            with_active_sink(session_id, &sink, |sink| sink.insert_text(&text));
        });

        let sink = Rc::downgrade(&self.sink);
        self.ime.on_delete(move |count| {
            with_active_sink(session_id, &sink, |sink| sink.delete_backward(count));
        });

        let sink = Rc::downgrade(&self.sink);
        self.ime.on_enter(move |_| {
            with_active_sink(session_id, &sink, HostInputSink::enter);
        });

        let keyboard_visible = Rc::downgrade(&self.keyboard_visible);
        self.ime.on_status_change(move |status| {
            if ACTIVE_SESSION_ID.load(Ordering::Acquire) != session_id {
                return;
            }
            if let Some(keyboard_visible) = keyboard_visible.upgrade() {
                keyboard_visible.set(matches!(status, KeyboardStatus::Show));
            }
        });
    }
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

fn with_active_sink(
    session_id: u64,
    sink: &Weak<HostInputSink>,
    callback: impl FnOnce(&HostInputSink),
) {
    if ACTIVE_SESSION_ID.load(Ordering::Acquire) != session_id {
        return;
    }
    if let Some(sink) = sink.upgrade() {
        callback(&sink);
    }
}

fn next_session_id() -> u64 {
    loop {
        let id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        if id != 0 {
            return id;
        }
    }
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
