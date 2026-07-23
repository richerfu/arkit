//! Key / mouse / focus encoding aligned with libghostty-vt encoders.
//!
//! Encoders produce **host-bound** bytes. The embedder must write them to the
//! PTY/SSH/local host (`host_write`), never into `ghostty_terminal_vt_write`.

#[cfg(not(ghostty_vt_stub))]
use std::os::raw::c_char;
#[cfg(not(ghostty_vt_stub))]
use std::ptr;

use crate::config::TerminalConfig;
use crate::error::TerminalResult;
#[cfg(not(ghostty_vt_stub))]
use crate::error::{TerminalError, TerminalErrorKind};
use crate::ffi::GhosttyTerminal;
#[cfg(not(ghostty_vt_stub))]
use crate::ffi::GHOSTTY_SUCCESS;

/// Modifier bitmask matching Ghostty `GHOSTTY_MODS_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyMods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl KeyMods {
    pub fn bits(self) -> u16 {
        let mut m = 0u16;
        if self.shift {
            m |= 1 << 0;
        }
        if self.ctrl {
            m |= 1 << 1;
        }
        if self.alt {
            m |= 1 << 2;
        }
        if self.super_key {
            m |= 1 << 3;
        }
        m
    }
}

/// A physical key chord for the key encoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyChord {
    pub name: String,
    pub mods: KeyMods,
    /// Optional UTF-8 text produced by the platform for this key.
    pub utf8: Option<String>,
}

impl KeyChord {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mods: KeyMods::default(),
            utf8: None,
        }
    }

    pub fn with_utf8(mut self, utf8: impl Into<String>) -> Self {
        self.utf8 = Some(utf8.into());
        self
    }

    pub fn with_mods(mut self, mods: KeyMods) -> Self {
        self.mods = mods;
        self
    }
}

/// Mouse event for the mouse encoder (surface-space pixels).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseInput {
    pub action: MouseAction,
    pub button: MouseButton,
    pub x: f32,
    pub y: f32,
    pub mods: KeyMods,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    Press,
    Release,
    Motion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Unknown,
    Left,
    Right,
    Middle,
}

/// Encode a named key → host bytes (never feeds VT).
pub fn encode_named_key(terminal: GhosttyTerminal, name: &str) -> TerminalResult<Vec<u8>> {
    encode_key_chord(terminal, KeyChord::named(name))
}

/// Encode a key chord → host bytes.
pub fn encode_key_chord(terminal: GhosttyTerminal, chord: KeyChord) -> TerminalResult<Vec<u8>> {
    #[cfg(ghostty_vt_stub)]
    {
        let _ = terminal;
        if let Some(ref utf8) = chord.utf8 {
            if !utf8.is_empty() {
                return Ok(utf8.as_bytes().to_vec());
            }
        }
        Ok(legacy_bytes(&chord.name).to_vec())
    }
    #[cfg(not(ghostty_vt_stub))]
    {
        encode_key_chord_native(terminal, chord)
    }
}

/// Encode mouse → host bytes (empty when tracking off).
pub fn encode_mouse(
    terminal: GhosttyTerminal,
    event: MouseInput,
    config: &TerminalConfig,
) -> TerminalResult<Vec<u8>> {
    #[cfg(ghostty_vt_stub)]
    {
        let _ = (terminal, event, config);
        Ok(Vec::new())
    }
    #[cfg(not(ghostty_vt_stub))]
    {
        encode_mouse_native(terminal, event, config)
    }
}

/// Encode focus report → host bytes (empty when mode 1004 off).
pub fn encode_focus(terminal: GhosttyTerminal, gained: bool) -> TerminalResult<Vec<u8>> {
    #[cfg(ghostty_vt_stub)]
    {
        let _ = (terminal, gained);
        Ok(Vec::new())
    }
    #[cfg(not(ghostty_vt_stub))]
    {
        encode_focus_native(terminal, gained)
    }
}

fn legacy_bytes(name: &str) -> &'static [u8] {
    match name {
        "enter" | "return" => b"\r",
        "backspace" => b"\x7f",
        "tab" => b"\t",
        "escape" | "esc" => b"\x1b",
        "arrow_up" | "up" => b"\x1b[A",
        "arrow_down" | "down" => b"\x1b[B",
        "arrow_right" | "right" => b"\x1b[C",
        "arrow_left" | "left" => b"\x1b[D",
        "home" => b"\x1b[H",
        "end" => b"\x1b[F",
        "page_up" => b"\x1b[5~",
        "page_down" => b"\x1b[6~",
        "delete" => b"\x1b[3~",
        "space" => b" ",
        _ => b"",
    }
}

#[cfg(not(ghostty_vt_stub))]
fn map_key(name: &str) -> Option<crate::ffi::GhosttyKey> {
    use crate::ffi::GhosttyKey::*;
    Some(match name {
        "enter" | "return" => GHOSTTY_KEY_ENTER,
        "backspace" => GHOSTTY_KEY_BACKSPACE,
        "tab" => GHOSTTY_KEY_TAB,
        "escape" | "esc" => GHOSTTY_KEY_ESCAPE,
        "arrow_up" | "up" => GHOSTTY_KEY_ARROW_UP,
        "arrow_down" | "down" => GHOSTTY_KEY_ARROW_DOWN,
        "arrow_left" | "left" => GHOSTTY_KEY_ARROW_LEFT,
        "arrow_right" | "right" => GHOSTTY_KEY_ARROW_RIGHT,
        "home" => GHOSTTY_KEY_HOME,
        "end" => GHOSTTY_KEY_END,
        "page_up" => GHOSTTY_KEY_PAGE_UP,
        "page_down" => GHOSTTY_KEY_PAGE_DOWN,
        "delete" => GHOSTTY_KEY_DELETE,
        "space" => GHOSTTY_KEY_SPACE,
        "a" | "A" => GHOSTTY_KEY_A,
        "c" | "C" => GHOSTTY_KEY_C,
        _ => return None,
    })
}

#[cfg(not(ghostty_vt_stub))]
fn encode_key_chord_native(terminal: GhosttyTerminal, chord: KeyChord) -> TerminalResult<Vec<u8>> {
    use crate::ffi::{
        ghostty_key_encoder_encode, ghostty_key_encoder_free, ghostty_key_encoder_new,
        ghostty_key_encoder_setopt_from_terminal, ghostty_key_event_free, ghostty_key_event_new,
        ghostty_key_event_set_action, ghostty_key_event_set_key, ghostty_key_event_set_mods,
        ghostty_key_event_set_utf8, GhosttyKeyAction, GhosttyKeyEncoder, GhosttyKeyEvent,
    };

    // Prefer platform text for printable input (IME commits).
    if let Some(ref utf8) = chord.utf8 {
        if !utf8.is_empty() && chord.mods.bits() == 0 && map_key(&chord.name).is_none() {
            return Ok(utf8.as_bytes().to_vec());
        }
    }

    let Some(key) = map_key(&chord.name) else {
        if let Some(ref utf8) = chord.utf8 {
            return Ok(utf8.as_bytes().to_vec());
        }
        let legacy = legacy_bytes(&chord.name);
        return Ok(legacy.to_vec());
    };

    let mut encoder: GhosttyKeyEncoder = ptr::null_mut();
    let rc = unsafe { ghostty_key_encoder_new(ptr::null(), &mut encoder) };
    if rc != GHOSTTY_SUCCESS || encoder.is_null() {
        return Err(TerminalError::new(
            TerminalErrorKind::Engine,
            format!("key_encoder_new failed ({rc:?})"),
        ));
    }
    unsafe {
        ghostty_key_encoder_setopt_from_terminal(encoder, terminal);
    }

    let mut event: GhosttyKeyEvent = ptr::null_mut();
    let rc = unsafe { ghostty_key_event_new(ptr::null(), &mut event) };
    if rc != GHOSTTY_SUCCESS || event.is_null() {
        unsafe { ghostty_key_encoder_free(encoder) };
        return Err(TerminalError::new(
            TerminalErrorKind::Engine,
            format!("key_event_new failed ({rc:?})"),
        ));
    }

    unsafe {
        ghostty_key_event_set_action(event, GhosttyKeyAction::GHOSTTY_KEY_ACTION_PRESS);
        ghostty_key_event_set_key(event, key);
        ghostty_key_event_set_mods(event, chord.mods.bits());
        if let Some(ref utf8) = chord.utf8 {
            ghostty_key_event_set_utf8(event, utf8.as_ptr().cast::<c_char>(), utf8.len());
        }
    }

    let mut buf = [0 as c_char; 128];
    let mut written: usize = 0;
    let rc = unsafe {
        ghostty_key_encoder_encode(encoder, event, buf.as_mut_ptr(), buf.len(), &mut written)
    };
    unsafe {
        ghostty_key_event_free(event);
        ghostty_key_encoder_free(encoder);
    }

    if rc != GHOSTTY_SUCCESS || written == 0 {
        let legacy = legacy_bytes(&chord.name);
        if !legacy.is_empty() {
            return Ok(legacy.to_vec());
        }
        if let Some(ref utf8) = chord.utf8 {
            return Ok(utf8.as_bytes().to_vec());
        }
        return Ok(Vec::new());
    }

    let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), written) };
    Ok(bytes.to_vec())
}

#[cfg(not(ghostty_vt_stub))]
fn encode_mouse_native(
    terminal: GhosttyTerminal,
    event: MouseInput,
    config: &TerminalConfig,
) -> TerminalResult<Vec<u8>> {
    use crate::ffi::{
        ghostty_mouse_encoder_encode, ghostty_mouse_encoder_free, ghostty_mouse_encoder_new,
        ghostty_mouse_encoder_setopt_from_terminal, ghostty_mouse_event_free,
        ghostty_mouse_event_new, ghostty_mouse_event_set_action, ghostty_mouse_event_set_button,
        ghostty_mouse_event_set_mods, ghostty_mouse_event_set_position, GhosttyMouseAction,
        GhosttyMouseButton, GhosttyMouseEncoder, GhosttyMouseEvent, GhosttyMousePosition,
    };

    let mut encoder: GhosttyMouseEncoder = ptr::null_mut();
    let rc = unsafe { ghostty_mouse_encoder_new(ptr::null(), &mut encoder) };
    if rc != GHOSTTY_SUCCESS || encoder.is_null() {
        return Err(TerminalError::new(
            TerminalErrorKind::Engine,
            format!("mouse_encoder_new failed ({rc:?})"),
        ));
    }
    unsafe {
        ghostty_mouse_encoder_setopt_from_terminal(encoder, terminal);
    }

    let mut me: GhosttyMouseEvent = ptr::null_mut();
    let rc = unsafe { ghostty_mouse_event_new(ptr::null(), &mut me) };
    if rc != GHOSTTY_SUCCESS || me.is_null() {
        unsafe { ghostty_mouse_encoder_free(encoder) };
        return Err(TerminalError::new(
            TerminalErrorKind::Engine,
            format!("mouse_event_new failed ({rc:?})"),
        ));
    }

    let action = match event.action {
        MouseAction::Press => GhosttyMouseAction::GHOSTTY_MOUSE_ACTION_PRESS,
        MouseAction::Release => GhosttyMouseAction::GHOSTTY_MOUSE_ACTION_RELEASE,
        MouseAction::Motion => GhosttyMouseAction::GHOSTTY_MOUSE_ACTION_MOTION,
    };
    let button = match event.button {
        MouseButton::Left => GhosttyMouseButton::GHOSTTY_MOUSE_BUTTON_LEFT,
        MouseButton::Right => GhosttyMouseButton::GHOSTTY_MOUSE_BUTTON_RIGHT,
        MouseButton::Middle => GhosttyMouseButton::GHOSTTY_MOUSE_BUTTON_MIDDLE,
        MouseButton::Unknown => GhosttyMouseButton::GHOSTTY_MOUSE_BUTTON_UNKNOWN,
    };
    let pos = GhosttyMousePosition {
        x: event.x,
        y: event.y,
    };

    unsafe {
        ghostty_mouse_event_set_action(me, action);
        ghostty_mouse_event_set_button(me, button);
        ghostty_mouse_event_set_mods(me, event.mods.bits());
        ghostty_mouse_event_set_position(me, pos);
        let _ = config;
    }

    let mut buf = [0 as c_char; 128];
    let mut written: usize = 0;
    let rc = unsafe {
        ghostty_mouse_encoder_encode(encoder, me, buf.as_mut_ptr(), buf.len(), &mut written)
    };
    unsafe {
        ghostty_mouse_event_free(me);
        ghostty_mouse_encoder_free(encoder);
    }

    if rc != GHOSTTY_SUCCESS || written == 0 {
        return Ok(Vec::new());
    }
    let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), written) };
    Ok(bytes.to_vec())
}

#[cfg(not(ghostty_vt_stub))]
fn encode_focus_native(terminal: GhosttyTerminal, gained: bool) -> TerminalResult<Vec<u8>> {
    use crate::ffi::{ghostty_focus_encode, GhosttyFocusEvent};
    let _ = terminal;

    let event = if gained {
        GhosttyFocusEvent::GHOSTTY_FOCUS_GAINED
    } else {
        GhosttyFocusEvent::GHOSTTY_FOCUS_LOST
    };
    let mut buf = [0 as c_char; 16];
    let mut written: usize = 0;
    let rc = unsafe { ghostty_focus_encode(event, buf.as_mut_ptr(), buf.len(), &mut written) };
    if rc != GHOSTTY_SUCCESS || written == 0 {
        return Ok(Vec::new());
    }
    let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), written) };
    Ok(bytes.to_vec())
}
