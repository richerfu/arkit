//! Event listener that turns rio-vt side effects into [`TerminalEffects`].

use std::sync::{Arc, Mutex};

use rio_vt::config::colors::ColorRgb;
use rio_vt::event::{EventListener, RioEvent, WindowId, WindowSize};

use crate::config::TerminalEffects;

#[derive(Clone)]
pub struct EffectsListener {
    pending: Arc<Mutex<TerminalEffects>>,
    size: Arc<Mutex<WindowSize>>,
}

impl EffectsListener {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(TerminalEffects::default())),
            size: Arc::new(Mutex::new(WindowSize::default())),
        }
    }

    pub fn set_size(&self, cols: u16, rows: u16, pixel_width: u16, pixel_height: u16) {
        if let Ok(mut size) = self.size.lock() {
            *size = WindowSize {
                cols,
                rows,
                width: pixel_width,
                height: pixel_height,
            };
        }
    }

    pub fn take(&self) -> TerminalEffects {
        self.pending
            .lock()
            .map(|mut pending| pending.take())
            .unwrap_or_default()
    }
}

impl EventListener for EffectsListener {
    fn send_event(&self, event: RioEvent, _id: WindowId) {
        self.dispatch(event);
    }

    fn send_event_with_high_priority(&self, event: RioEvent, _id: WindowId) {
        self.dispatch(event);
    }
}

impl EffectsListener {
    fn dispatch(&self, event: RioEvent) {
        let Ok(mut pending) = self.pending.lock() else {
            return;
        };
        match event {
            RioEvent::PtyWrite(_, text) => pending.write_pty.extend_from_slice(text.as_bytes()),
            RioEvent::Bell => pending.bell = true,
            RioEvent::Title(title) | RioEvent::TitleWithSubtitle(title, _) => {
                pending.title = Some(title);
            }
            RioEvent::ColorRequest(_, index, formatter) => {
                let (r, g, b) = crate::capture::xterm_color(index);
                pending
                    .write_pty
                    .extend_from_slice(formatter(ColorRgb { r, g, b }).as_bytes());
            }
            RioEvent::TextAreaSizeRequest(_, formatter) => {
                let size = self.size.lock().map(|s| *s).unwrap_or_default();
                pending
                    .write_pty
                    .extend_from_slice(formatter(size).as_bytes());
            }
            _ => {}
        }
    }
}
