//! rio-vt terminal engine — parser, grid, effects, host-bound encoders.
//!
//! Host I/O stays with the embedder:
//!
//! ```text
//!   keyboard/IME ──encode_*──► host bytes ──► your PTY / SSH / shell
//!   PTY / SSH / shell ──► feed_vt / write_bytes ──► paint
//!   write_pty effect ──► your host write (same path)
//! ```
//!
//! Encoders never feed the VT parser. Only host output (and synthetic demo
//! VT) should use [`TerminalEngine::feed_vt`].

use rio_vt::ansi::CursorShape;
use rio_vt::config::colors::NamedColor;
use rio_vt::crosswords::grid::Scroll;
use rio_vt::crosswords::{Crosswords, CrosswordsSize};
use rio_vt::event::WindowId;
use rio_vt::performer::handler::Processor;

use crate::capture;
use crate::config::{Rgb, TerminalConfig, TerminalEffects};
use crate::effects::EffectsListener;
use crate::error::{TerminalError, TerminalErrorKind, TerminalResult};
use crate::frame::{CursorVisualStyle, TerminalCursor, TerminalFrame};
use crate::input::{self, EncodeState, KeyChord, MouseInput};

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

/// Owns the rio-vt grid, parser, and effect listener.
pub struct TerminalEngine {
    term: Crosswords<EffectsListener>,
    parser: Processor,
    listener: EffectsListener,
    config: TerminalConfig,
    last_effects: TerminalEffects,
    last_pwd: Option<String>,
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
        let listener = EffectsListener::new();
        listener.set_size(
            config.cols,
            config.rows,
            saturating_u16(config.cols as u32 * config.cell_width_px),
            saturating_u16(config.rows as u32 * config.cell_height_px),
        );
        let size = CrosswordsSize::new_with_dimensions(
            config.cols as usize,
            config.rows as usize,
            config.cols as u32 * config.cell_width_px,
            config.rows as u32 * config.cell_height_px,
            config.cell_width_px,
            config.cell_height_px,
        );
        let mut term = Crosswords::new(
            size,
            cursor_shape(config.default_cursor_style),
            listener.clone(),
            WindowId::from(1),
            1,
            config.scrollback,
        );
        apply_config(&mut term, &listener, &config);
        Ok(Self {
            term,
            parser: Processor::default(),
            listener,
            config,
            last_effects: TerminalEffects::default(),
            last_pwd: None,
        })
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
        apply_config(&mut self.term, &self.listener, &config);
        self.config = config;
        Ok(())
    }

    /// Host → terminal: feed PTY/SSH/shell output into the VT parser.
    ///
    /// After the write, if the viewport was pinned to the active area it
    /// stays at the bottom. Overflow rows enter scrollback automatically.
    pub fn feed_vt(&mut self, data: &[u8]) {
        self.write_bytes(data);
    }

    /// Host → terminal (same as [`feed_vt`]).
    pub fn write_bytes(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        let was_active = self.viewport_active();
        self.parser.advance(&mut self.term, data);
        self.last_effects.extend(self.listener.take());
        let pwd = self
            .term
            .current_directory
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned());
        if pwd != self.last_pwd {
            self.last_effects.pwd = pwd.clone();
            self.last_pwd = pwd;
        }
        if was_active {
            self.scroll_to_bottom();
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    /// Effects produced by the last VT write (write_pty, bell, title, pwd).
    pub fn take_effects(&mut self) -> TerminalEffects {
        self.last_effects.extend(self.listener.take());
        std::mem::take(&mut self.last_effects)
    }

    /// Whether the viewport is pinned to the live bottom.
    pub fn viewport_active(&self) -> bool {
        self.term.display_offset() == 0
    }

    /// Scroll viewport to the live bottom (active area).
    pub fn scroll_to_bottom(&mut self) {
        self.term.scroll_display(Scroll::Bottom);
    }

    /// Scroll viewport to the top of scrollback.
    pub fn scroll_to_top(&mut self) {
        self.term.scroll_display(Scroll::Top);
    }

    /// Scroll by a signed row delta (negative = into history / up).
    pub fn scroll_by(&mut self, delta_rows: i64) {
        let delta = (-delta_rows).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        self.term.scroll_display(Scroll::Delta(delta));
    }

    /// Scroll so absolute history row `row` is the first visible line.
    pub fn scroll_to_row(&mut self, row: u64) {
        let history = self.term.history_size() as u64;
        let target = history.saturating_sub(row) as i32;
        let current = self.term.display_offset() as i32;
        self.term.scroll_display(Scroll::Delta(target - current));
    }

    pub fn capture(&mut self) -> TerminalResult<TerminalFrame> {
        Ok(capture::capture_frame(&self.term, &self.config))
    }

    pub fn cursor(&mut self) -> TerminalResult<TerminalCursor> {
        Ok(self.capture()?.cursor)
    }

    /// Encode a named key → **host-bound** bytes (never feeds VT).
    pub fn encode_key(&self, name: &str) -> TerminalResult<Vec<u8>> {
        input::encode_named_key(self.encode_state(), name)
    }

    /// Encode a key chord → host-bound bytes.
    pub fn encode_key_chord(&self, chord: KeyChord) -> TerminalResult<Vec<u8>> {
        input::encode_key_chord(self.encode_state(), chord)
    }

    /// Encode printable text as host-bound UTF-8 (what a PTY expects for typing).
    pub fn encode_text(&self, text: &str) -> Vec<u8> {
        let _ = self;
        text.as_bytes().to_vec()
    }

    /// Encode mouse → host-bound bytes (empty when tracking is off).
    pub fn encode_mouse(&self, event: MouseInput) -> TerminalResult<Vec<u8>> {
        input::encode_mouse(self.encode_state(), event, &self.config)
    }

    /// Encode focus report → host-bound bytes (empty when mode 1004 is off).
    pub fn encode_focus(&self, gained: bool) -> TerminalResult<Vec<u8>> {
        input::encode_focus(self.encode_state(), gained)
    }

    pub(crate) fn encode_state_bits(&self) -> u32 {
        self.encode_state().to_bits()
    }

    fn encode_state(&self) -> EncodeState {
        use rio_vt::crosswords::Mode;
        let mode = self.term.mode();
        EncodeState {
            app_cursor: mode.contains(Mode::APP_CURSOR),
            mouse_reporting: mode.intersects(Mode::MOUSE_MODE),
            sgr_mouse: mode.contains(Mode::SGR_MOUSE),
            utf8_mouse: mode.contains(Mode::UTF8_MOUSE),
            x10_mouse: mode.contains(Mode::MOUSE_REPORT_X10),
            mouse_drag: mode.contains(Mode::MOUSE_DRAG),
            mouse_motion: mode.contains(Mode::MOUSE_MOTION),
            focus_report: mode.contains(Mode::FOCUS_IN_OUT),
        }
    }
}

fn apply_config(
    term: &mut Crosswords<EffectsListener>,
    listener: &EffectsListener,
    config: &TerminalConfig,
) {
    listener.set_size(
        config.cols,
        config.rows,
        saturating_u16(config.cols as u32 * config.cell_width_px),
        saturating_u16(config.rows as u32 * config.cell_height_px),
    );
    term.resize(CrosswordsSize::new_with_dimensions(
        config.cols as usize,
        config.rows as usize,
        config.cols as u32 * config.cell_width_px,
        config.rows as u32 * config.cell_height_px,
        config.cell_width_px,
        config.cell_height_px,
    ));
    if let Some(fg) = config.foreground {
        term.colors[NamedColor::Foreground] = Some(rgb_to_arr(fg));
    }
    if let Some(bg) = config.background {
        term.colors[NamedColor::Background] = Some(rgb_to_arr(bg));
    }
    if let Some(cursor) = config.cursor_color {
        term.colors[NamedColor::Cursor] = Some(rgb_to_arr(cursor));
    }
    if let Some(ref palette) = config.palette {
        for (i, rgb) in palette.iter().enumerate() {
            term.colors[i] = Some(rgb_to_arr(*rgb));
        }
    }
    let shape = cursor_shape(config.default_cursor_style);
    term.default_cursor_shape = shape;
    term.cursor_shape = shape;
    term.blinking_cursor = config.default_cursor_blink;
}

fn cursor_shape(style: CursorVisualStyle) -> CursorShape {
    match style {
        CursorVisualStyle::Bar => CursorShape::Beam,
        CursorVisualStyle::Underline => CursorShape::Underline,
        CursorVisualStyle::Block | CursorVisualStyle::BlockHollow => CursorShape::Block,
    }
}

fn rgb_to_arr(rgb: Rgb) -> [f32; 4] {
    [
        rgb.r as f32 / 255.0,
        rgb.g as f32 / 255.0,
        rgb.b as f32 / 255.0,
        1.0,
    ]
}

fn saturating_u16(value: u32) -> u16 {
    value.min(u16::MAX as u32) as u16
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
        assert!(frame.cursor.visible);
        assert_eq!((frame.cursor.col, frame.cursor.row), (5, 1));
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
        assert_eq!(before, after);
    }
}
