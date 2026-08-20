//! Snapshot a [`TerminalFrame`] from rio-vt `Crosswords` visible rows.

use rio_vt::ansi::CursorShape;
use rio_vt::config::colors::term::TermColors;
use rio_vt::config::colors::{AnsiColor, NamedColor};
use rio_vt::crosswords::grid::Dimensions;
use rio_vt::crosswords::pos::Column;
use rio_vt::crosswords::square::{ContentTag, Wide};
use rio_vt::crosswords::style::StyleFlags;
use rio_vt::crosswords::Crosswords;
use rio_vt::event::EventListener;

use crate::config::TerminalConfig;
use crate::frame::{
    rgb_to_argb, CursorVisualStyle, TerminalCell, TerminalCursor, TerminalFrame, TerminalScrollbar,
};

pub fn capture_frame<U: EventListener>(
    term: &Crosswords<U>,
    config: &TerminalConfig,
) -> TerminalFrame {
    let cols = term.grid.columns() as u16;
    let rows = term.grid.screen_lines() as u16;
    let default_fg = resolve_named(&term.colors, NamedColor::Foreground, config);
    let default_bg = resolve_named(&term.colors, NamedColor::Background, config);
    let cursor_color = resolve_named(&term.colors, NamedColor::Cursor, config);

    let visible = term.visible_rows();
    let mut cells = Vec::with_capacity(cols as usize * rows as usize);
    for row in visible.iter().take(rows as usize) {
        for col in 0..cols as usize {
            let square = row[Column(col)];
            cells.push(read_cell(term, square, default_fg, default_bg, config));
        }
    }
    while cells.len() < cols as usize * rows as usize {
        cells.push(TerminalCell {
            fg: default_fg,
            bg: default_bg,
            width: 1,
            is_spacer: false,
            ..TerminalCell::default()
        });
    }

    let cursor_state = term.cursor();
    let style = match cursor_state.content {
        CursorShape::Beam => CursorVisualStyle::Bar,
        CursorShape::Underline => CursorVisualStyle::Underline,
        CursorShape::Hidden | CursorShape::Block => CursorVisualStyle::Block,
    };
    let col = cursor_state
        .pos
        .col
        .0
        .min(usize::from(cols.saturating_sub(1))) as u16;
    let row = cursor_state.pos.row.0.max(0) as u16;
    let visible = cursor_state.content != CursorShape::Hidden && term.display_offset() == 0;

    let history = term.history_size() as u64;
    let offset = term.display_offset() as u64;
    TerminalFrame {
        cols,
        rows,
        default_fg,
        default_bg,
        cells,
        cursor: TerminalCursor {
            col,
            row: row.min(rows.saturating_sub(1)),
            visible,
            blinking: term.blinking_cursor,
            style,
            color: Some(cursor_color),
        },
        scrollbar: TerminalScrollbar {
            total: history.saturating_add(rows as u64).max(1),
            offset: history.saturating_sub(offset),
            len: rows as u64,
            viewport_active: offset == 0,
        },
    }
}

fn read_cell<U: EventListener>(
    term: &Crosswords<U>,
    square: rio_vt::crosswords::square::Square,
    default_fg: u32,
    default_bg: u32,
    config: &TerminalConfig,
) -> TerminalCell {
    let (width, is_spacer) = match square.wide() {
        Wide::Wide => (2u8, false),
        Wide::Spacer | Wide::LeadingSpacer => (0u8, true),
        Wide::Narrow => (1, false),
    };

    if square.is_bg_only() || is_spacer {
        let bg = match square.content_tag() {
            ContentTag::BgPalette => {
                resolve_indexed(&term.colors, square.bg_palette_index(), config)
            }
            ContentTag::BgRgb => {
                let (r, g, b) = square.bg_rgb();
                rgb_to_argb(r, g, b)
            }
            ContentTag::Codepoint => default_bg,
        };
        return TerminalCell {
            grapheme: String::new(),
            fg: default_fg,
            bg,
            width,
            is_spacer,
            ..TerminalCell::default()
        };
    }

    let style = term.grid.style_of(&square);
    let flags = style.flags;
    let mut grapheme = String::new();
    let ch = square.c();
    if ch != '\0' {
        grapheme.push(ch);
    }
    if square.has_grapheme() {
        if let Some(extras) = square
            .extras_id()
            .and_then(|id| term.grid.extras_table.get(id))
        {
            grapheme.extend(extras.zerowidth.iter());
        }
    }
    if flags.contains(StyleFlags::HIDDEN) {
        grapheme.clear();
    }

    TerminalCell {
        grapheme: if is_spacer { String::new() } else { grapheme },
        fg: resolve_color(&term.colors, style.fg, config),
        bg: resolve_color(&term.colors, style.bg, config),
        bold: flags.contains(StyleFlags::BOLD),
        italic: flags.contains(StyleFlags::ITALIC),
        faint: flags.contains(StyleFlags::DIM),
        underline: flags.intersects(StyleFlags::ALL_UNDERLINES),
        underline_kind: underline_kind(flags),
        strikethrough: flags.contains(StyleFlags::STRIKEOUT),
        inverse: flags.contains(StyleFlags::INVERSE),
        selected: false,
        width,
        is_spacer,
    }
}

fn underline_kind(flags: StyleFlags) -> u8 {
    if flags.contains(StyleFlags::DOUBLE_UNDERLINE) {
        2
    } else if flags.contains(StyleFlags::UNDERCURL) {
        3
    } else if flags.contains(StyleFlags::DOTTED_UNDERLINE) {
        4
    } else if flags.contains(StyleFlags::DASHED_UNDERLINE) {
        5
    } else if flags.contains(StyleFlags::UNDERLINE) {
        1
    } else {
        0
    }
}

fn resolve_color(colors: &TermColors, color: AnsiColor, config: &TerminalConfig) -> u32 {
    match color {
        AnsiColor::Spec(rgb) => rgb_to_argb(rgb.r, rgb.g, rgb.b),
        AnsiColor::Named(name) => resolve_named(colors, name, config),
        AnsiColor::Indexed(index) => resolve_indexed(colors, index, config),
    }
}

fn resolve_named(colors: &TermColors, name: NamedColor, config: &TerminalConfig) -> u32 {
    if let Some(arr) = colors[name] {
        return arr_to_argb(arr);
    }
    match name {
        NamedColor::Foreground => config
            .foreground
            .map(crate::config::Rgb::to_argb)
            .unwrap_or(0xFFE2_E8F0),
        NamedColor::Background => config
            .background
            .map(crate::config::Rgb::to_argb)
            .unwrap_or(0xFF0B_1220),
        NamedColor::Cursor => config
            .cursor_color
            .map(crate::config::Rgb::to_argb)
            .unwrap_or(0xFFE2_E8F0),
        other => {
            let (r, g, b) = xterm_color(other as usize);
            rgb_to_argb(r, g, b)
        }
    }
}

fn resolve_indexed(colors: &TermColors, index: u8, _config: &TerminalConfig) -> u32 {
    if let Some(arr) = colors[index as usize] {
        return arr_to_argb(arr);
    }
    let (r, g, b) = xterm_color(index as usize);
    rgb_to_argb(r, g, b)
}

fn arr_to_argb(arr: [f32; 4]) -> u32 {
    let byte = |channel: f32| (channel * 255.0).round().clamp(0.0, 255.0) as u8;
    rgb_to_argb(byte(arr[0]), byte(arr[1]), byte(arr[2]))
}

/// Xterm 256-color cube plus the named/default slots used by color queries.
pub(crate) fn xterm_color(index: usize) -> (u8, u8, u8) {
    const ANSI: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (205, 0, 0),
        (0, 205, 0),
        (205, 205, 0),
        (0, 0, 238),
        (205, 0, 205),
        (0, 205, 205),
        (229, 229, 229),
        (127, 127, 127),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (92, 92, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];
    match index {
        0..=15 => ANSI[index],
        16..=231 => {
            let n = index - 16;
            let level = |v: usize| if v == 0 { 0 } else { (v * 40 + 55) as u8 };
            (level(n / 36), level((n % 36) / 6), level(n % 6))
        }
        232..=255 => {
            let gray = (8 + (index - 232) * 10) as u8;
            (gray, gray, gray)
        }
        256 | 258 => (0xE2, 0xE8, 0xF0), // foreground / cursor
        257 => (0x0B, 0x12, 0x20),       // background
        _ => {
            if index < ANSI.len() {
                ANSI[index]
            } else {
                (0xE2, 0xE8, 0xF0)
            }
        }
    }
}
