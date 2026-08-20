//! Key / mouse / focus encoding.
//!
//! Encoders produce **host-bound** bytes. The embedder must write them to the
//! PTY/SSH/local host, never into [`crate::TerminalEngine::feed_vt`].

use crate::config::TerminalConfig;
use crate::error::TerminalResult;

/// Modifier bitmask (shift/ctrl/alt/super).
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

/// Terminal modes the encoder needs and the embedder must not track itself.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EncodeState {
    pub app_cursor: bool,
    pub mouse_reporting: bool,
    pub sgr_mouse: bool,
    pub utf8_mouse: bool,
    pub x10_mouse: bool,
    pub mouse_drag: bool,
    pub mouse_motion: bool,
    pub focus_report: bool,
}

pub fn encode_named_key(state: EncodeState, name: &str) -> TerminalResult<Vec<u8>> {
    encode_key_chord(state, KeyChord::named(name))
}

pub fn encode_key_chord(state: EncodeState, chord: KeyChord) -> TerminalResult<Vec<u8>> {
    if let Some(ref utf8) = chord.utf8 {
        if !utf8.is_empty() && chord.mods.bits() == 0 && !is_named_key(&chord.name) {
            return Ok(utf8.as_bytes().to_vec());
        }
    }
    if let Some(bytes) = encode_named(&chord.name, chord.mods, state.app_cursor) {
        return Ok(bytes);
    }
    if let Some(ref utf8) = chord.utf8 {
        return Ok(utf8.as_bytes().to_vec());
    }
    Ok(Vec::new())
}

pub fn encode_mouse(
    state: EncodeState,
    event: MouseInput,
    config: &TerminalConfig,
) -> TerminalResult<Vec<u8>> {
    if !state.mouse_reporting && !state.x10_mouse {
        return Ok(Vec::new());
    }
    if event.mods.shift {
        return Ok(Vec::new());
    }
    let button: u8 = match event.button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        MouseButton::Unknown => 3,
    };
    match event.action {
        MouseAction::Motion => {
            if button >= 3 && !state.mouse_motion {
                return Ok(Vec::new());
            }
            if button < 3 && !state.mouse_drag && !state.mouse_motion {
                return Ok(Vec::new());
            }
        }
        MouseAction::Press | MouseAction::Release => {
            if state.x10_mouse && (event.action != MouseAction::Press || button > 2) {
                return Ok(Vec::new());
            }
        }
    }
    let cell_w = config.cell_width_px.max(1) as f32;
    let cell_h = config.cell_height_px.max(1) as f32;
    let col = (event.x / cell_w).floor().max(0.0) as u16;
    let row = (event.y / cell_h).floor().max(0.0) as u16;
    let mut encoded = match event.action {
        MouseAction::Motion => button.saturating_add(32),
        _ => button,
    };
    if !state.x10_mouse {
        if event.mods.alt {
            encoded += 8;
        }
        if event.mods.ctrl {
            encoded += 16;
        }
    }
    let pressed = event.action != MouseAction::Release;
    Ok(mouse_report(
        encoded,
        col,
        row,
        pressed,
        state.sgr_mouse,
        state.utf8_mouse,
    ))
}

pub fn encode_focus(state: EncodeState, gained: bool) -> TerminalResult<Vec<u8>> {
    if !state.focus_report {
        return Ok(Vec::new());
    }
    Ok(if gained {
        b"\x1b[I".to_vec()
    } else {
        b"\x1b[O".to_vec()
    })
}

fn is_named_key(name: &str) -> bool {
    matches!(
        name,
        "enter"
            | "return"
            | "backspace"
            | "tab"
            | "escape"
            | "esc"
            | "arrow_up"
            | "up"
            | "arrow_down"
            | "down"
            | "arrow_right"
            | "right"
            | "arrow_left"
            | "left"
            | "home"
            | "end"
            | "page_up"
            | "page_down"
            | "delete"
            | "space"
    )
}

fn encode_named(name: &str, mods: KeyMods, app_cursor: bool) -> Option<Vec<u8>> {
    let param = {
        let mut value = 1u8;
        if mods.shift {
            value += 1;
        }
        if mods.alt {
            value += 2;
        }
        if mods.ctrl {
            value += 4;
        }
        if mods.super_key {
            value += 8;
        }
        value
    };
    let modified = param != 1;
    let csi = |final_byte: u8, ss3: bool| -> Vec<u8> {
        if modified {
            format!("\x1b[1;{param}{}", final_byte as char).into_bytes()
        } else if ss3 {
            vec![0x1b, b'O', final_byte]
        } else {
            vec![0x1b, b'[', final_byte]
        }
    };
    Some(match name {
        "enter" | "return" => b"\r".to_vec(),
        "backspace" => b"\x7f".to_vec(),
        "tab" => {
            if mods.shift {
                b"\x1b[Z".to_vec()
            } else {
                b"\t".to_vec()
            }
        }
        "escape" | "esc" => b"\x1b".to_vec(),
        "space" => b" ".to_vec(),
        "arrow_up" | "up" => csi(b'A', app_cursor),
        "arrow_down" | "down" => csi(b'B', app_cursor),
        "arrow_right" | "right" => csi(b'C', app_cursor),
        "arrow_left" | "left" => csi(b'D', app_cursor),
        "home" => csi(b'H', app_cursor),
        "end" => csi(b'F', app_cursor),
        "page_up" => {
            if modified {
                format!("\x1b[5;{param}~").into_bytes()
            } else {
                b"\x1b[5~".to_vec()
            }
        }
        "page_down" => {
            if modified {
                format!("\x1b[6;{param}~").into_bytes()
            } else {
                b"\x1b[6~".to_vec()
            }
        }
        "delete" => {
            if modified {
                format!("\x1b[3;{param}~").into_bytes()
            } else {
                b"\x1b[3~".to_vec()
            }
        }
        _ => return None,
    })
}

fn mouse_report(button: u8, col: u16, row: u16, pressed: bool, sgr: bool, utf8: bool) -> Vec<u8> {
    let x = col.saturating_add(1);
    let y = row.saturating_add(1);
    if sgr {
        let end = if pressed { 'M' } else { 'm' };
        return format!("\x1b[<{button};{x};{y}{end}").into_bytes();
    }
    let encoded = if pressed { button } else { button | 3 };
    let mut out = vec![0x1b, b'[', b'M', 32u8.saturating_add(encoded)];
    for value in [x, y] {
        if utf8 && value >= 95 {
            let encoded = char::from_u32(32 + u32::from(value)).unwrap_or('\u{20}');
            let mut buffer = [0u8; 4];
            out.extend_from_slice(encoded.encode_utf8(&mut buffer).as_bytes());
        } else {
            out.push(32u8.saturating_add(value.min(223) as u8));
        }
    }
    out
}
