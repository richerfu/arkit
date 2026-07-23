//! Embedder configuration mapped to `GHOSTTY_TERMINAL_OPT_*` and create options.
//!
//! Covers the libghostty-vt configuration surface that arkit exposes today:
//! geometry, scrollback, cell pixel size, default colors, default cursor
//! style/blink, and optional protocol toggles. Effect callbacks (write_pty,
//! bell, title, …) are registered separately via [`crate::effects`].

use crate::frame::CursorVisualStyle;

/// RGB color (0–255 channels) used for Ghostty color options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_argb(argb: u32) -> Self {
        Self {
            r: ((argb >> 16) & 0xFF) as u8,
            g: ((argb >> 8) & 0xFF) as u8,
            b: (argb & 0xFF) as u8,
        }
    }

    pub fn to_argb(self) -> u32 {
        0xFF00_0000 | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// Full terminal configuration applied at create / reconfigure time.
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalConfig {
    /// Grid columns (cells).
    pub cols: u16,
    /// Grid rows (cells).
    pub rows: u16,
    /// Maximum scrollback lines (`GhosttyTerminalOptions.max_scrollback`).
    pub scrollback: usize,
    /// Cell width in pixels for `ghostty_terminal_resize` / size reports.
    pub cell_width_px: u32,
    /// Cell height in pixels.
    pub cell_height_px: u32,
    /// Default foreground (`GHOSTTY_TERMINAL_OPT_COLOR_FOREGROUND`).
    pub foreground: Option<Rgb>,
    /// Default background (`GHOSTTY_TERMINAL_OPT_COLOR_BACKGROUND`).
    pub background: Option<Rgb>,
    /// Default cursor color (`GHOSTTY_TERMINAL_OPT_COLOR_CURSOR`).
    pub cursor_color: Option<Rgb>,
    /// Optional full 256-color palette override.
    pub palette: Option<Box<[Rgb; 256]>>,
    /// Default cursor visual style for DECSCUSR reset (`OPT_DEFAULT_CURSOR_STYLE`).
    pub default_cursor_style: CursorVisualStyle,
    /// Default cursor blink for DECSCUSR reset (`OPT_DEFAULT_CURSOR_BLINK`).
    pub default_cursor_blink: bool,
    /// Kitty graphics storage limit in bytes (`0` = disabled). `None` = leave default.
    pub kitty_image_storage_limit: Option<u64>,
    /// Enable Glyph Protocol APC handling.
    pub glyph_protocol: Option<bool>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            scrollback: 10_000,
            cell_width_px: 8,
            cell_height_px: 18,
            foreground: Some(Rgb::new(0xE2, 0xE8, 0xF0)),
            background: Some(Rgb::new(0x0B, 0x12, 0x20)),
            cursor_color: Some(Rgb::new(0xE2, 0xE8, 0xF0)),
            palette: None,
            default_cursor_style: CursorVisualStyle::Block,
            default_cursor_blink: false,
            kitty_image_storage_limit: None,
            glyph_protocol: None,
        }
    }
}

impl TerminalConfig {
    pub fn with_size(mut self, cols: u16, rows: u16) -> Self {
        self.cols = cols;
        self.rows = rows;
        self
    }

    pub fn with_scrollback(mut self, scrollback: usize) -> Self {
        self.scrollback = scrollback;
        self
    }

    pub fn with_cell_metrics(mut self, width_px: u32, height_px: u32) -> Self {
        self.cell_width_px = width_px;
        self.cell_height_px = height_px;
        self
    }

    pub fn with_theme(mut self, fg: Rgb, bg: Rgb, cursor: Rgb) -> Self {
        self.foreground = Some(fg);
        self.background = Some(bg);
        self.cursor_color = Some(cursor);
        self
    }

    pub fn with_cursor_style(mut self, style: CursorVisualStyle, blink: bool) -> Self {
        self.default_cursor_style = style;
        self.default_cursor_blink = blink;
        self
    }
}

/// Side-channel effect events produced during VT processing (libghostty effects).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerminalEffects {
    /// Bytes the terminal wants written to the PTY (query responses).
    pub write_pty: Vec<u8>,
    /// BEL received.
    pub bell: bool,
    /// Title changed (latest value, if any).
    pub title: Option<String>,
    /// Pwd changed (latest value, if any).
    pub pwd: Option<String>,
}

impl TerminalEffects {
    pub fn take(&mut self) -> TerminalEffects {
        std::mem::take(self)
    }

    pub fn is_empty(&self) -> bool {
        self.write_pty.is_empty() && !self.bell && self.title.is_none() && self.pwd.is_none()
    }

    pub(crate) fn extend(&mut self, mut next: TerminalEffects) {
        self.write_pty.append(&mut next.write_pty);
        self.bell |= next.bell;
        if next.title.is_some() {
            self.title = next.title;
        }
        if next.pwd.is_some() {
            self.pwd = next.pwd;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TerminalEffects;

    #[test]
    fn effects_accumulate_bytes_and_keep_latest_metadata() {
        let mut effects = TerminalEffects {
            write_pty: vec![1],
            bell: false,
            title: Some("old".into()),
            pwd: None,
        };
        effects.extend(TerminalEffects {
            write_pty: vec![2, 3],
            bell: true,
            title: Some("new".into()),
            pwd: Some("/tmp".into()),
        });

        assert_eq!(effects.write_pty, [1, 2, 3]);
        assert!(effects.bell);
        assert_eq!(effects.title.as_deref(), Some("new"));
        assert_eq!(effects.pwd.as_deref(), Some("/tmp"));
    }
}
