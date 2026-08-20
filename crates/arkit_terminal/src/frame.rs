//! Screen frame model captured from the rio-vt visible grid.
//!
//! Per-cell graphemes + styles, palette-resolved colors, and viewport
//! cursor visual style. The GPU renderer packs this into cell instances.

/// Visual caret style (DECSCUSR / render-state cursor visual style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorVisualStyle {
    Bar,
    #[default]
    Block,
    Underline,
    BlockHollow,
}

/// Caret within the viewport (cell coordinates, 0-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TerminalCursor {
    pub col: u16,
    pub row: u16,
    pub visible: bool,
    pub blinking: bool,
    pub style: CursorVisualStyle,
    /// Explicit cursor color when the terminal set one (ARGB).
    pub color: Option<u32>,
}

/// One terminal cell after render-state resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalCell {
    pub grapheme: String,
    /// Resolved ARGB foreground (already palette-expanded).
    pub fg: u32,
    /// Resolved ARGB background.
    pub bg: u32,
    pub bold: bool,
    pub italic: bool,
    pub faint: bool,
    pub underline: bool,
    /// 0 none, 1 single, 2 double, 3 curly, 4 dotted, 5 dashed.
    pub underline_kind: u8,
    pub strikethrough: bool,
    pub inverse: bool,
    pub selected: bool,
    /// Display width in grid columns: `1` narrow, `2` wide (CJK/emoji).
    /// Spacers use `0` and must not be painted (see [`Self::is_spacer`]).
    pub width: u8,
    /// Wide-character spacer (tail/head). Do not render; grid still advances.
    pub is_spacer: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            grapheme: String::new(),
            fg: 0xFFE2_E8F0,
            bg: 0xFF0B_1220,
            bold: false,
            italic: false,
            faint: false,
            underline: false,
            underline_kind: 0,
            strikethrough: false,
            inverse: false,
            selected: false,
            width: 1,
            is_spacer: false,
        }
    }
}

impl TerminalCell {
    /// Effective paint colors after inverse / selection / faint.
    pub fn paint_colors(&self, default_fg: u32, default_bg: u32) -> (u32, u32) {
        let mut fg = self.fg;
        let mut bg = self.bg;
        if self.inverse || self.selected {
            std::mem::swap(&mut fg, &mut bg);
        }
        if self.faint {
            fg = dim_argb(fg);
        }
        if fg == 0 {
            fg = default_fg;
        }
        if bg == 0 {
            bg = default_bg;
        }
        (fg, bg)
    }
}

/// A run of consecutive cells with identical paint attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalRun {
    pub text: String,
    pub fg: u32,
    pub bg: u32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    /// Number of grid columns this run occupies.
    pub cols: u16,
    /// When set, this run is the caret (visual applied by the renderer).
    pub cursor: Option<CursorVisualStyle>,
    /// Paint color for bar / underline / hollow caret stroke (ARGB).
    pub cursor_color: Option<u32>,
}

/// Scrollbar / history window.
///
/// The paint surface is always one **viewport** of `len` rows. Older lines
/// live in scrollback; `offset` is the first visible row from the top of
/// the scrollable area. When [`Self::viewport_active`] is true the viewport
/// is pinned to the live bottom (standard “follow output” mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TerminalScrollbar {
    /// Total scrollable rows (scrollback + active viewport).
    pub total: u64,
    /// First visible row index from the top of history.
    pub offset: u64,
    /// Viewport height in rows (usually equals [`TerminalFrame::rows`]).
    pub len: u64,
    /// `true` when pinned to the active (bottom) area.
    pub viewport_active: bool,
}

/// Full viewport frame captured from rio-vt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalFrame {
    pub cols: u16,
    pub rows: u16,
    pub default_fg: u32,
    pub default_bg: u32,
    /// Row-major cells: `cells[row * cols + col]`.
    pub cells: Vec<TerminalCell>,
    pub cursor: TerminalCursor,
    /// History / pin state for the current viewport.
    pub scrollbar: TerminalScrollbar,
}

impl Default for TerminalFrame {
    fn default() -> Self {
        Self {
            cols: 0,
            rows: 0,
            default_fg: 0xFFE2_E8F0,
            default_bg: 0xFF0B_1220,
            cells: Vec::new(),
            cursor: TerminalCursor::default(),
            scrollbar: TerminalScrollbar::default(),
        }
    }
}

impl TerminalFrame {
    pub fn cell(&self, col: u16, row: u16) -> Option<&TerminalCell> {
        let i = row as usize * self.cols as usize + col as usize;
        self.cells.get(i)
    }

    /// Collapse each row into attribute-runs, applying caret visual style.
    pub fn rows_as_runs(&self) -> Vec<Vec<TerminalRun>> {
        self.rows_as_runs_with_cursor(true)
    }

    pub(crate) fn rows_as_runs_with_cursor(
        &self,
        cursor_phase_visible: bool,
    ) -> Vec<Vec<TerminalRun>> {
        let mut out = Vec::with_capacity(self.rows as usize);
        for row in 0..self.rows {
            out.push(self.row_runs(row, cursor_phase_visible));
        }
        out
    }

    fn row_runs(&self, row: u16, cursor_phase_visible: bool) -> Vec<TerminalRun> {
        let mut runs: Vec<TerminalRun> = Vec::new();
        let base = row as usize * self.cols as usize;
        let mut col = 0u16;
        while col < self.cols {
            let mut cell = self
                .cells
                .get(base + col as usize)
                .cloned()
                .unwrap_or_default();

            // A normal wide tail is skipped because the primary advances by
            // two columns. If capture starts on an orphan spacer/head, it
            // still owns one grid column; reserve a blank cell instead of
            // shifting every following glyph and the cursor to the left.
            if cell.is_spacer || cell.width == 0 {
                cell.grapheme.clear();
                cell.width = 1;
                cell.is_spacer = false;
            }

            let width = cell.width.max(1) as u16;
            // Cap so we never walk past the row edge.
            let width = width.min(self.cols.saturating_sub(col).max(1));
            let (mut fg, mut bg) = cell.paint_colors(self.default_fg, self.default_bg);
            let mut cursor_style = None;
            let mut cursor_paint = None;

            // Cursor may sit on the wide cell's primary column or its tail
            // spacer — attach caret paint to the primary cell in both cases.
            let cursor_on_cell = cursor_phase_visible
                && self.cursor.visible
                && self.cursor.row == row
                && (self.cursor.col == col
                    || (width > 1
                        && self.cursor.col > col
                        && self.cursor.col < col.saturating_add(width)));

            if cursor_on_cell {
                cursor_style = Some(self.cursor.style);
                let cursor_color =
                    self.cursor
                        .color
                        .unwrap_or(if self.default_fg != self.default_bg {
                            self.default_fg
                        } else {
                            0xFF38_BDF8
                        });
                cursor_paint = Some(cursor_color);
                match self.cursor.style {
                    CursorVisualStyle::Block => {
                        // Solid block: invert the single cell only
                        // (character stays; no second overlay glyph).
                        bg = cursor_color;
                        // Prefer cell's original bg as ink so the glyph stays
                        // readable on the block fill.
                        fg = if cell.bg != 0 && cell.bg != cursor_color {
                            cell.bg
                        } else {
                            self.default_bg
                        };
                        if fg == bg {
                            fg = self.default_fg;
                        }
                    }
                    CursorVisualStyle::BlockHollow => {}
                    CursorVisualStyle::Bar | CursorVisualStyle::Underline => {
                        // Stroke only in the component; keep cell ink/paper.
                    }
                }
            }

            // Grid placeholder: one space per column so empty wide cells
            // still reserve two half-width slots on the native surface.
            let text = if cell.grapheme.is_empty() {
                " ".repeat(width as usize)
            } else {
                cell.grapheme.clone()
            };

            let key = (
                fg,
                bg,
                cell.bold,
                cell.italic,
                cell.underline,
                cell.strikethrough,
                cursor_style,
                cursor_paint,
            );
            if let Some(last) = runs.last_mut() {
                let last_key = (
                    last.fg,
                    last.bg,
                    last.bold,
                    last.italic,
                    last.underline,
                    last.strikethrough,
                    last.cursor,
                    last.cursor_color,
                );
                // Merge only plain single-width runs. Wide cells stay alone so
                // each fullwidth glyph owns a 2-column box. Never merge caret.
                let can_merge = last_key == key
                    && cursor_style.is_none()
                    && last.cursor.is_none()
                    && width == 1
                    && cell.width <= 1
                    && !cell.is_spacer
                    && !last.text.chars().any(is_likely_wide_char)
                    && !text.chars().any(is_likely_wide_char);
                if can_merge {
                    last.text.push_str(&text);
                    last.cols = last.cols.saturating_add(width);
                    col = col.saturating_add(width);
                    continue;
                }
            }
            runs.push(TerminalRun {
                text,
                fg,
                bg,
                bold: cell.bold,
                italic: cell.italic,
                underline: cell.underline,
                strikethrough: cell.strikethrough,
                cols: width,
                cursor: cursor_style,
                cursor_color: cursor_paint,
            });
            col = col.saturating_add(width);
        }
        runs
    }

    /// Plain text dump (debug / tests). Spacers omitted; wide cells once.
    pub fn plain(&self) -> String {
        let mut s = String::new();
        for row in 0..self.rows {
            let mut col = 0u16;
            while col < self.cols {
                if let Some(cell) = self.cell(col, row) {
                    if cell.is_spacer || cell.width == 0 {
                        col = col.saturating_add(1);
                        continue;
                    }
                    if cell.grapheme.is_empty() {
                        s.push(' ');
                    } else {
                        s.push_str(&cell.grapheme);
                    }
                    col = col.saturating_add(cell.width.max(1) as u16);
                } else {
                    break;
                }
            }
            if row + 1 < self.rows {
                s.push('\n');
            }
        }
        s
    }
}

/// Terminal display width of a single codepoint (1 or 2).
#[allow(dead_code)] // used by stub capture and public embedders
pub fn east_asian_width(c: char) -> u8 {
    if is_likely_wide_char(c) {
        2
    } else {
        1
    }
}

/// Conservative fullwidth detector used only to avoid merging CJK into
/// narrow half-width runs (font metrics then mis-align).
fn is_likely_wide_char(c: char) -> bool {
    let u = c as u32;
    matches!(
        u,
        0x1100..=0x115F
            | 0x2329..=0x232A
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE6F
            | 0xFF01..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F300..=0x1FAFF
            | 0x20000..=0x3FFFD
    )
}

pub fn rgb_to_argb(r: u8, g: u8, b: u8) -> u32 {
    0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn dim_argb(color: u32) -> u32 {
    let a = (color >> 24) & 0xFF;
    let r = ((color >> 16) & 0xFF) * 2 / 3;
    let g = ((color >> 8) & 0xFF) * 2 / 3;
    let b = (color & 0xFF) * 2 / 3;
    (a << 24) | (r << 16) | (g << 8) | b
}

#[cfg(test)]
mod tests {
    use super::{CursorVisualStyle, TerminalCell, TerminalCursor, TerminalFrame};

    fn frame(cols: u16, cells: Vec<TerminalCell>) -> TerminalFrame {
        TerminalFrame {
            cols,
            rows: 1,
            cells,
            cursor: TerminalCursor::default(),
            ..TerminalFrame::default()
        }
    }

    #[test]
    fn cursor_splits_runs_without_changing_grid_width() {
        let mut frame = frame(
            3,
            "abc"
                .chars()
                .map(|ch| TerminalCell {
                    grapheme: ch.to_string(),
                    ..TerminalCell::default()
                })
                .collect(),
        );
        frame.cursor = TerminalCursor {
            col: 1,
            row: 0,
            visible: true,
            style: CursorVisualStyle::Block,
            ..TerminalCursor::default()
        };

        let runs = frame.rows_as_runs().remove(0);
        assert_eq!(runs.iter().map(|run| run.cols).sum::<u16>(), 3);
        assert_eq!(runs.iter().filter(|run| run.cursor.is_some()).count(), 1);
        assert_eq!(
            runs.iter()
                .find(|run| run.cursor.is_some())
                .map(|run| run.cols),
            Some(1)
        );
    }

    #[test]
    fn orphan_spacer_keeps_following_cells_in_their_columns() {
        let cells = vec![
            TerminalCell {
                width: 0,
                is_spacer: true,
                ..TerminalCell::default()
            },
            TerminalCell {
                grapheme: "x".into(),
                ..TerminalCell::default()
            },
        ];
        let runs = frame(2, cells).rows_as_runs().remove(0);

        assert_eq!(runs.iter().map(|run| run.cols).sum::<u16>(), 2);
        assert_eq!(
            runs.iter().map(|run| run.text.as_str()).collect::<String>(),
            " x"
        );
    }
}
