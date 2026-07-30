//! C effect callbacks for `GHOSTTY_TERMINAL_OPT_*` (write_pty, bell, title, pwd).
//!
//! Callbacks must not re-enter `ghostty_terminal_vt_write`. They only queue
//! into [`crate::config::TerminalEffects`]; the engine drains after each write.

use std::cell::RefCell;
#[cfg(not(ghostty_vt_stub))]
use std::os::raw::c_void;
#[cfg(not(ghostty_vt_stub))]
use std::ptr;

use crate::config::TerminalEffects;

/// Heap state passed as Ghostty userdata for the lifetime of a terminal.
pub struct EffectsBridge {
    pub pending: RefCell<TerminalEffects>,
}

impl EffectsBridge {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            pending: RefCell::new(TerminalEffects::default()),
        })
    }

    #[cfg(not(ghostty_vt_stub))]
    pub fn as_userdata(this: &Self) -> *mut c_void {
        (this as *const EffectsBridge).cast_mut().cast()
    }

    pub fn take(&self) -> TerminalEffects {
        self.pending.borrow_mut().take()
    }
}

/// Apply config colors / cursor defaults via `ghostty_terminal_set` + resize.
pub fn apply_config(terminal: crate::ffi::GhosttyTerminal, config: &crate::config::TerminalConfig) {
    #[cfg(not(ghostty_vt_stub))]
    {
        apply_config_native(terminal, config);
    }
    #[cfg(ghostty_vt_stub)]
    {
        let _ = (terminal, config);
    }
}

/// Install userdata + effect callbacks on a live terminal.
pub fn install_effects(terminal: crate::ffi::GhosttyTerminal, bridge: &EffectsBridge) {
    #[cfg(not(ghostty_vt_stub))]
    {
        install_effects_native(terminal, bridge);
    }
    #[cfg(ghostty_vt_stub)]
    {
        let _ = (terminal, bridge);
    }
}

#[cfg(not(ghostty_vt_stub))]
fn apply_config_native(
    terminal: crate::ffi::GhosttyTerminal,
    config: &crate::config::TerminalConfig,
) {
    use crate::ffi::{
        ghostty_terminal_resize, ghostty_terminal_set, GhosttyColorRgb, GhosttyTerminalCursorStyle,
        GhosttyTerminalOption,
    };
    use crate::frame::CursorVisualStyle;

    // SAFETY: stack values live for each set/resize call.
    unsafe {
        let _ = ghostty_terminal_resize(
            terminal,
            config.cols,
            config.rows,
            config.cell_width_px,
            config.cell_height_px,
        );

        if let Some(fg) = config.foreground {
            let c = GhosttyColorRgb {
                r: fg.r,
                g: fg.g,
                b: fg.b,
            };
            let _ = ghostty_terminal_set(
                terminal,
                GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_COLOR_FOREGROUND,
                (&raw const c).cast(),
            );
        }
        if let Some(bg) = config.background {
            let c = GhosttyColorRgb {
                r: bg.r,
                g: bg.g,
                b: bg.b,
            };
            let _ = ghostty_terminal_set(
                terminal,
                GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_COLOR_BACKGROUND,
                (&raw const c).cast(),
            );
        }
        if let Some(cur) = config.cursor_color {
            let c = GhosttyColorRgb {
                r: cur.r,
                g: cur.g,
                b: cur.b,
            };
            let _ = ghostty_terminal_set(
                terminal,
                GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_COLOR_CURSOR,
                (&raw const c).cast(),
            );
        }
        if let Some(ref palette) = config.palette {
            let mut arr = [GhosttyColorRgb { r: 0, g: 0, b: 0 }; 256];
            for (i, rgb) in palette.iter().enumerate() {
                arr[i] = GhosttyColorRgb {
                    r: rgb.r,
                    g: rgb.g,
                    b: rgb.b,
                };
            }
            let _ = ghostty_terminal_set(
                terminal,
                GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_COLOR_PALETTE,
                arr.as_ptr().cast(),
            );
        }

        let style = match config.default_cursor_style {
            CursorVisualStyle::Bar => GhosttyTerminalCursorStyle::GHOSTTY_TERMINAL_CURSOR_STYLE_BAR,
            CursorVisualStyle::Underline => {
                GhosttyTerminalCursorStyle::GHOSTTY_TERMINAL_CURSOR_STYLE_UNDERLINE
            }
            CursorVisualStyle::BlockHollow => {
                GhosttyTerminalCursorStyle::GHOSTTY_TERMINAL_CURSOR_STYLE_BLOCK_HOLLOW
            }
            CursorVisualStyle::Block => {
                GhosttyTerminalCursorStyle::GHOSTTY_TERMINAL_CURSOR_STYLE_BLOCK
            }
        };
        let _ = ghostty_terminal_set(
            terminal,
            GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_DEFAULT_CURSOR_STYLE,
            (&raw const style).cast(),
        );
        let blink = config.default_cursor_blink;
        let _ = ghostty_terminal_set(
            terminal,
            GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_DEFAULT_CURSOR_BLINK,
            (&raw const blink).cast(),
        );

        if let Some(limit) = config.kitty_image_storage_limit {
            let _ = ghostty_terminal_set(
                terminal,
                GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_KITTY_IMAGE_STORAGE_LIMIT,
                (&raw const limit).cast(),
            );
        }
        if let Some(glyph) = config.glyph_protocol {
            let _ = ghostty_terminal_set(
                terminal,
                GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_GLYPH_PROTOCOL,
                (&raw const glyph).cast(),
            );
        }

        // DEC modes (packed: bits0–14 value, bit15 ANSI=1 / DEC private=0).
        // DECTCEM 25, cursor blink 12, DECAWM wraparound 7 (long lines wrap,
        // they must not vanish past the right margin).
        let mode_cursor_visible: u16 = 25;
        let mode_cursor_blink: u16 = 12;
        let mode_wraparound: u16 = 7;
        let _ = crate::ffi::ghostty_terminal_mode_set(terminal, mode_cursor_visible, true);
        let _ = crate::ffi::ghostty_terminal_mode_set(
            terminal,
            mode_cursor_blink,
            config.default_cursor_blink,
        );
        let _ = crate::ffi::ghostty_terminal_mode_set(terminal, mode_wraparound, true);
    }
}

#[cfg(not(ghostty_vt_stub))]
fn install_effects_native(terminal: crate::ffi::GhosttyTerminal, bridge: &EffectsBridge) {
    use crate::ffi::{ghostty_terminal_set, GhosttyTerminalOption};

    let ud = EffectsBridge::as_userdata(bridge);
    // SAFETY: bridge owned by engine for terminal lifetime; callbacks only queue.
    unsafe {
        let _ = ghostty_terminal_set(
            terminal,
            GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_USERDATA,
            ud,
        );
        // Callback options: pass function pointer as the value (not a pointer-to-fn).
        let _ = ghostty_terminal_set(
            terminal,
            GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_WRITE_PTY,
            write_pty_cb as *const c_void,
        );
        let _ = ghostty_terminal_set(
            terminal,
            GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_BELL,
            bell_cb as *const c_void,
        );
        let _ = ghostty_terminal_set(
            terminal,
            GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_TITLE_CHANGED,
            title_changed_cb as *const c_void,
        );
        let _ = ghostty_terminal_set(
            terminal,
            GhosttyTerminalOption::GHOSTTY_TERMINAL_OPT_PWD_CHANGED,
            pwd_changed_cb as *const c_void,
        );
    }
}

#[cfg(not(ghostty_vt_stub))]
unsafe extern "C" fn write_pty_cb(
    _terminal: crate::ffi::GhosttyTerminal,
    userdata: *mut c_void,
    data: *const u8,
    len: usize,
) {
    // SAFETY: userdata is EffectsBridge; data valid for this call only.
    unsafe {
        if userdata.is_null() || data.is_null() || len == 0 {
            return;
        }
        let bridge = &*(userdata as *const EffectsBridge);
        let bytes = std::slice::from_raw_parts(data, len);
        bridge
            .pending
            .borrow_mut()
            .write_pty
            .extend_from_slice(bytes);
    }
}

#[cfg(not(ghostty_vt_stub))]
unsafe extern "C" fn bell_cb(_terminal: crate::ffi::GhosttyTerminal, userdata: *mut c_void) {
    unsafe {
        if userdata.is_null() {
            return;
        }
        let bridge = &*(userdata as *const EffectsBridge);
        bridge.pending.borrow_mut().bell = true;
    }
}

#[cfg(not(ghostty_vt_stub))]
unsafe extern "C" fn title_changed_cb(
    terminal: crate::ffi::GhosttyTerminal,
    userdata: *mut c_void,
) {
    unsafe {
        if userdata.is_null() {
            return;
        }
        let bridge = &*(userdata as *const EffectsBridge);
        if let Some(title) = read_string_data(
            terminal,
            crate::ffi::GhosttyTerminalData::GHOSTTY_TERMINAL_DATA_TITLE,
        ) {
            bridge.pending.borrow_mut().title = Some(title);
        }
    }
}

#[cfg(not(ghostty_vt_stub))]
unsafe extern "C" fn pwd_changed_cb(terminal: crate::ffi::GhosttyTerminal, userdata: *mut c_void) {
    unsafe {
        if userdata.is_null() {
            return;
        }
        let bridge = &*(userdata as *const EffectsBridge);
        if let Some(pwd) = read_string_data(
            terminal,
            crate::ffi::GhosttyTerminalData::GHOSTTY_TERMINAL_DATA_PWD,
        ) {
            bridge.pending.borrow_mut().pwd = Some(pwd);
        }
    }
}

#[cfg(not(ghostty_vt_stub))]
unsafe fn read_string_data(
    terminal: crate::ffi::GhosttyTerminal,
    key: crate::ffi::GhosttyTerminalData,
) -> Option<String> {
    use crate::ffi::{ghostty_terminal_get, GhosttyString, GHOSTTY_SUCCESS};
    unsafe {
        let mut s = GhosttyString {
            ptr: ptr::null(),
            len: 0,
        };
        let rc = ghostty_terminal_get(terminal, key, (&raw mut s).cast::<c_void>());
        if rc != GHOSTTY_SUCCESS {
            return None;
        }
        if s.ptr.is_null() || s.len == 0 {
            return Some(String::new());
        }
        let bytes = std::slice::from_raw_parts(s.ptr, s.len);
        Some(String::from_utf8_lossy(bytes).into_owned())
    }
}
