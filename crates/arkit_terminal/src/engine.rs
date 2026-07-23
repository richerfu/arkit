//! libghostty-vt terminal engine — config, render-state paint, effects, encoders.
//!
//! ## Ghostty I/O model (embedder responsibility)
//!
//! ```text
//!   keyboard/IME ──encode_*──► host bytes ──► your PTY / SSH / shell
//!   PTY / SSH / shell ──► feed_vt / write_bytes ──► paint
//!   write_pty effect ──► your host write (same path)
//! ```
//!
//! Encoders **never** call `ghostty_terminal_vt_write`. Only host output
//! (and synthetic demo VT) should use [`TerminalEngine::feed_vt`].

use std::ptr;

use crate::capture;
use crate::config::{TerminalConfig, TerminalEffects};
use crate::effects::{self, EffectsBridge};
use crate::error::{TerminalError, TerminalErrorKind, TerminalResult};
use crate::ffi::{self, GhosttyTerminal, GhosttyTerminalOptions, GHOSTTY_SUCCESS};
use crate::frame::{TerminalCursor, TerminalFrame};
use crate::input::{self, KeyChord, MouseInput};

/// Viewport scroll target (Ghostty `ghostty_terminal_scroll_viewport`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollTarget {
    Top,
    Bottom,
    /// Negative = up into history.
    Delta(i64),
    Row(u64),
}

#[cfg(not(ghostty_vt_stub))]
use crate::ffi::GhosttyRenderState;

/// Grid geometry (subset of [`TerminalConfig`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
    pub scrollback: usize,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            scrollback: 10_000,
        }
    }
}

impl From<&TerminalConfig> for TerminalSize {
    fn from(c: &TerminalConfig) -> Self {
        Self {
            cols: c.cols,
            rows: c.rows,
            scrollback: c.scrollback,
        }
    }
}

/// Owns Ghostty terminal + render state + effects bridge.
pub struct TerminalEngine {
    terminal: GhosttyTerminal,
    #[cfg(not(ghostty_vt_stub))]
    render: GhosttyRenderState,
    config: TerminalConfig,
    effects: Box<EffectsBridge>,
    last_effects: TerminalEffects,
    #[cfg(ghostty_vt_stub)]
    stub_buffer: String,
}

impl TerminalEngine {
    pub fn new(size: TerminalSize) -> TerminalResult<Self> {
        Self::with_config(
            TerminalConfig::default()
                .with_size(size.cols, size.rows)
                .with_scrollback(size.scrollback),
        )
    }

    pub fn with_config(config: TerminalConfig) -> TerminalResult<Self> {
        if config.cols == 0 || config.rows == 0 {
            return Err(TerminalError::new(
                TerminalErrorKind::InvalidSize,
                "cols and rows must be > 0",
            ));
        }
        let opts = GhosttyTerminalOptions {
            cols: config.cols,
            rows: config.rows,
            max_scrollback: config.scrollback,
        };
        let mut terminal: GhosttyTerminal = ptr::null_mut();
        // SAFETY: out-pointer valid.
        let rc = unsafe { ffi::ghostty_terminal_new(ptr::null(), &mut terminal, opts) };
        if rc != GHOSTTY_SUCCESS || terminal.is_null() {
            return Err(TerminalError::new(
                TerminalErrorKind::Engine,
                format!("ghostty_terminal_new failed ({rc:?})"),
            ));
        }

        let effects = EffectsBridge::new();
        effects::install_effects(terminal, &effects);
        effects::apply_config(terminal, &config);

        #[cfg(not(ghostty_vt_stub))]
        {
            let mut render: GhosttyRenderState = ptr::null_mut();
            let rc = unsafe { ffi::ghostty_render_state_new(ptr::null(), &mut render) };
            if rc != GHOSTTY_SUCCESS || render.is_null() {
                unsafe { ffi::ghostty_terminal_free(terminal) };
                return Err(TerminalError::new(
                    TerminalErrorKind::Engine,
                    format!("ghostty_render_state_new failed ({rc:?})"),
                ));
            }
            Ok(Self {
                terminal,
                render,
                config,
                effects,
                last_effects: TerminalEffects::default(),
            })
        }

        #[cfg(ghostty_vt_stub)]
        {
            Ok(Self {
                terminal,
                config,
                effects,
                last_effects: TerminalEffects::default(),
                stub_buffer: String::new(),
            })
        }
    }

    pub fn config(&self) -> &TerminalConfig {
        &self.config
    }

    pub fn size(&self) -> TerminalSize {
        TerminalSize::from(&self.config)
    }

    /// Re-apply config (colors, cursor defaults, cell metrics, resize).
    pub fn reconfigure(&mut self, config: TerminalConfig) -> TerminalResult<()> {
        if config.cols == 0 || config.rows == 0 {
            return Err(TerminalError::new(
                TerminalErrorKind::InvalidSize,
                "cols and rows must be > 0",
            ));
        }
        effects::apply_config(self.terminal, &config);
        self.config = config;
        Ok(())
    }

    /// Host → terminal: feed PTY/SSH/shell output into the VT parser.
    ///
    /// Alias of the Ghostty path `ghostty_terminal_vt_write`.
    ///
    /// After the write, if the viewport was pinned to the active area
    /// (Ghostty “follow output”), it stays at the bottom. Overflow rows
    /// enter scrollback automatically (`max_scrollback`); they are not lost.
    pub fn feed_vt(&mut self, data: &[u8]) {
        self.write_bytes(data);
    }

    /// Host → terminal (same as [`feed_vt`]).
    pub fn write_bytes(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        // Follow Ghostty: only re-pin bottom when already on the active area.
        // If the user scrolled into history, leave the viewport alone.
        let was_active = self.viewport_active();
        #[cfg(ghostty_vt_stub)]
        {
            self.stub_buffer.push_str(&String::from_utf8_lossy(data));
        }
        // SAFETY: terminal live.
        unsafe {
            ffi::ghostty_terminal_vt_write(self.terminal, data.as_ptr(), data.len());
        }
        self.last_effects.extend(self.effects.take());
        if was_active {
            self.scroll_to_bottom();
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    /// Effects produced by the last VT write (write_pty, bell, title, pwd).
    pub fn take_effects(&mut self) -> TerminalEffects {
        std::mem::take(&mut self.last_effects)
    }

    /// Whether the viewport is pinned to the live bottom (`VIEWPORT_ACTIVE`).
    pub fn viewport_active(&self) -> bool {
        #[cfg(ghostty_vt_stub)]
        {
            true
        }
        #[cfg(not(ghostty_vt_stub))]
        {
            let mut active = true;
            unsafe {
                let _ = ffi::ghostty_terminal_get(
                    self.terminal,
                    ffi::GhosttyTerminalData::GHOSTTY_TERMINAL_DATA_VIEWPORT_ACTIVE,
                    (&raw mut active).cast::<std::ffi::c_void>(),
                );
            }
            active
        }
    }

    /// Scroll viewport to the live bottom (active area).
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_viewport(ScrollTarget::Bottom);
    }

    /// Scroll viewport to the top of scrollback.
    pub fn scroll_to_top(&mut self) {
        self.scroll_viewport(ScrollTarget::Top);
    }

    /// Scroll by a signed row delta (negative = into history / up).
    pub fn scroll_by(&mut self, delta_rows: i64) {
        self.scroll_viewport(ScrollTarget::Delta(delta_rows));
    }

    /// Scroll so absolute history row `row` is the first visible line.
    pub fn scroll_to_row(&mut self, row: u64) {
        self.scroll_viewport(ScrollTarget::Row(row));
    }

    fn scroll_viewport(&mut self, target: ScrollTarget) {
        #[cfg(ghostty_vt_stub)]
        {
            let _ = target;
        }
        #[cfg(not(ghostty_vt_stub))]
        {
            use ffi::{
                ghostty_terminal_scroll_viewport, GhosttyTerminalScrollViewport,
                GhosttyTerminalScrollViewportTag, GhosttyTerminalScrollViewportValue,
            };
            let behavior = match target {
                ScrollTarget::Top => GhosttyTerminalScrollViewport {
                    tag: GhosttyTerminalScrollViewportTag::GHOSTTY_SCROLL_VIEWPORT_TOP,
                    value: unsafe { std::mem::zeroed() },
                },
                ScrollTarget::Bottom => GhosttyTerminalScrollViewport {
                    tag: GhosttyTerminalScrollViewportTag::GHOSTTY_SCROLL_VIEWPORT_BOTTOM,
                    value: unsafe { std::mem::zeroed() },
                },
                ScrollTarget::Delta(d) => GhosttyTerminalScrollViewport {
                    tag: GhosttyTerminalScrollViewportTag::GHOSTTY_SCROLL_VIEWPORT_DELTA,
                    value: GhosttyTerminalScrollViewportValue { delta: d as isize },
                },
                ScrollTarget::Row(r) => GhosttyTerminalScrollViewport {
                    tag: GhosttyTerminalScrollViewportTag::GHOSTTY_SCROLL_VIEWPORT_ROW,
                    value: GhosttyTerminalScrollViewportValue { row: r as usize },
                },
            };
            // SAFETY: terminal live; tagged union matches Ghostty ABI.
            unsafe {
                ghostty_terminal_scroll_viewport(self.terminal, behavior);
            }
        }
    }

    pub fn capture(&mut self) -> TerminalResult<TerminalFrame> {
        #[cfg(not(ghostty_vt_stub))]
        {
            capture::capture_frame(
                self.terminal,
                self.render,
                self.config.cols,
                self.config.rows,
            )
        }
        #[cfg(ghostty_vt_stub)]
        {
            Ok(capture::capture_frame_stub(
                self.config.cols,
                self.config.rows,
                &self.stub_buffer,
            ))
        }
    }

    pub fn cursor(&mut self) -> TerminalResult<TerminalCursor> {
        Ok(self.capture()?.cursor)
    }

    /// Encode a named key → **host-bound** bytes (never feeds VT).
    pub fn encode_key(&self, name: &str) -> TerminalResult<Vec<u8>> {
        input::encode_named_key(self.terminal, name)
    }

    /// Encode a key chord → host-bound bytes.
    pub fn encode_key_chord(&self, chord: KeyChord) -> TerminalResult<Vec<u8>> {
        input::encode_key_chord(self.terminal, chord)
    }

    /// Encode printable text as host-bound UTF-8 (what a PTY expects for typing).
    pub fn encode_text(&self, text: &str) -> Vec<u8> {
        let _ = self;
        text.as_bytes().to_vec()
    }

    /// Encode mouse → host-bound bytes (empty when tracking is off).
    pub fn encode_mouse(&self, event: MouseInput) -> TerminalResult<Vec<u8>> {
        input::encode_mouse(self.terminal, event, &self.config)
    }

    /// Encode focus report → host-bound bytes (empty when mode 1004 is off).
    pub fn encode_focus(&self, gained: bool) -> TerminalResult<Vec<u8>> {
        input::encode_focus(self.terminal, gained)
    }

    pub fn raw_terminal(&self) -> GhosttyTerminal {
        self.terminal
    }
}

impl Drop for TerminalEngine {
    fn drop(&mut self) {
        #[cfg(not(ghostty_vt_stub))]
        {
            if !self.render.is_null() {
                // SAFETY: `render` is the live state created by this engine and
                // is released exactly once during drop.
                unsafe { ffi::ghostty_render_state_free(self.render) };
                self.render = ptr::null_mut();
            }
        }
        if !self.terminal.is_null() {
            // SAFETY: `terminal` is the live handle created by this engine and
            // is released exactly once after the render state.
            unsafe { ffi::ghostty_terminal_free(self.terminal) };
            self.terminal = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Rgb;
    use crate::frame::CursorVisualStyle;

    #[test]
    fn config_theme_and_render_capture() {
        let config = TerminalConfig::default()
            .with_size(40, 8)
            .with_scrollback(0)
            .with_theme(
                Rgb::new(0xE2, 0xE8, 0xF0),
                Rgb::new(0x0B, 0x12, 0x20),
                Rgb::new(0x38, 0xBD, 0xF8),
            )
            .with_cursor_style(CursorVisualStyle::Block, false);
        let mut engine = TerminalEngine::with_config(config).expect("engine");
        engine.feed_vt(b"Hello \x1b[1;32mworld\x1b[0m!\r\n\x1b[2;6H");
        let frame = engine.capture().expect("frame");
        assert_eq!(frame.cols, 40);
        assert!(frame
            .cells
            .iter()
            .any(|c| c.bold || c.fg != frame.default_fg));
        if !cfg!(ghostty_vt_stub) {
            assert!(frame.cursor.visible);
            assert_eq!((frame.cursor.col, frame.cursor.row), (5, 1));
        }
    }

    #[test]
    fn key_encode_is_host_bound_not_vt() {
        let mut engine = TerminalEngine::new(TerminalSize {
            cols: 40,
            rows: 8,
            scrollback: 0,
        })
        .expect("engine");
        engine.feed_vt(b"abc");
        let before = engine.capture().expect("frame").plain();
        let bytes = engine.encode_key("arrow_left").expect("left");
        assert!(!bytes.is_empty());
        let after = engine.capture().expect("frame").plain();
        // Encoding must not mutate the grid.
        assert_eq!(before, after);
    }
}
