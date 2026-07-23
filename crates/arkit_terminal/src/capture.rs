//! Capture a [`TerminalFrame`] from libghostty-vt `GhosttyRenderState`.
//!
//! Mirrors Ghostty `example/c-vt-render`: update render state → colors →
//! row/cell iterators → resolve styles → cursor visual.

#[cfg(not(ghostty_vt_stub))]
use std::os::raw::c_void;
#[cfg(not(ghostty_vt_stub))]
use std::ptr;

#[cfg(not(ghostty_vt_stub))]
use crate::error::{TerminalError, TerminalErrorKind, TerminalResult};
#[cfg(not(ghostty_vt_stub))]
use crate::ffi::{self, GHOSTTY_SUCCESS};
#[cfg(not(ghostty_vt_stub))]
use crate::frame::rgb_to_argb;
use crate::frame::{
    CursorVisualStyle, TerminalCell, TerminalCursor, TerminalFrame, TerminalScrollbar,
};

#[cfg(not(ghostty_vt_stub))]
use crate::ffi::{
    GhosttyBuffer, GhosttyColorRgb, GhosttyRenderState, GhosttyRenderStateColors,
    GhosttyRenderStateCursorVisualStyle, GhosttyRenderStateData, GhosttyRenderStateDirty,
    GhosttyRenderStateOption, GhosttyRenderStateRowCells, GhosttyRenderStateRowCellsData,
    GhosttyRenderStateRowData, GhosttyRenderStateRowIterator, GhosttyRenderStateRowOption,
    GhosttyStyle, GhosttyTerminal,
};

/// Snapshot the terminal through the official render-state path.
#[cfg(not(ghostty_vt_stub))]
pub fn capture_frame(
    terminal: GhosttyTerminal,
    render: GhosttyRenderState,
    mut cols: u16,
    mut rows: u16,
) -> TerminalResult<TerminalFrame> {
    // SAFETY: handles owned by TerminalEngine; live for this call.
    let rc = unsafe { ffi::ghostty_render_state_update(render, terminal) };
    if rc != GHOSTTY_SUCCESS {
        return Err(TerminalError::new(
            TerminalErrorKind::Format,
            format!("ghostty_render_state_update failed ({rc:?})"),
        ));
    }

    // Colors (palette for resolution is available but FG/BG_COLOR on cells
    // already return resolved RGB).
    // SAFETY: sized-struct init pattern from Ghostty docs.
    let mut colors: GhosttyRenderStateColors = unsafe { std::mem::zeroed() };
    colors.size = std::mem::size_of::<GhosttyRenderStateColors>();
    let rc = unsafe { ffi::ghostty_render_state_colors_get(render, &mut colors) };
    if rc != GHOSTTY_SUCCESS {
        return Err(TerminalError::new(
            TerminalErrorKind::Format,
            format!("ghostty_render_state_colors_get failed ({rc:?})"),
        ));
    }
    let default_fg = rgb_to_argb(
        colors.foreground.r,
        colors.foreground.g,
        colors.foreground.b,
    );
    let default_bg = rgb_to_argb(
        colors.background.r,
        colors.background.g,
        colors.background.b,
    );

    // Prefer live geometry from render-state (matches Ghostty viewport size).
    let mut rs_cols = cols;
    let mut rs_rows = rows;
    unsafe {
        let _ = ffi::ghostty_render_state_get(
            render,
            GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_COLS,
            (&raw mut rs_cols).cast::<c_void>(),
        );
        let _ = ffi::ghostty_render_state_get(
            render,
            GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_ROWS,
            (&raw mut rs_rows).cast::<c_void>(),
        );
    }
    if rs_cols > 0 {
        cols = rs_cols;
    }
    if rs_rows > 0 {
        rows = rs_rows;
    }

    let cursor = read_cursor(render, default_fg);
    let scrollbar = read_scrollbar(terminal, rows);

    let mut row_iter: GhosttyRenderStateRowIterator = ptr::null_mut();
    let rc = unsafe { ffi::ghostty_render_state_row_iterator_new(ptr::null(), &mut row_iter) };
    if rc != GHOSTTY_SUCCESS || row_iter.is_null() {
        return Err(TerminalError::new(
            TerminalErrorKind::Format,
            format!("row_iterator_new failed ({rc:?})"),
        ));
    }
    // SAFETY: populate iterator from render state.
    let rc = unsafe {
        ffi::ghostty_render_state_get(
            render,
            GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_ROW_ITERATOR,
            (&raw mut row_iter).cast::<c_void>(),
        )
    };
    if rc != GHOSTTY_SUCCESS {
        unsafe { ffi::ghostty_render_state_row_iterator_free(row_iter) };
        return Err(TerminalError::new(
            TerminalErrorKind::Format,
            format!("ROW_ITERATOR get failed ({rc:?})"),
        ));
    }

    let mut cells_handle: GhosttyRenderStateRowCells = ptr::null_mut();
    let rc = unsafe { ffi::ghostty_render_state_row_cells_new(ptr::null(), &mut cells_handle) };
    if rc != GHOSTTY_SUCCESS || cells_handle.is_null() {
        unsafe { ffi::ghostty_render_state_row_iterator_free(row_iter) };
        return Err(TerminalError::new(
            TerminalErrorKind::Format,
            format!("row_cells_new failed ({rc:?})"),
        ));
    }

    let mut cells: Vec<TerminalCell> = Vec::with_capacity(cols as usize * rows as usize);
    let mut row_i = 0u16;
    // SAFETY: iterator/cells owned by us for this snapshot.
    while unsafe { ffi::ghostty_render_state_row_iterator_next(row_iter) } {
        if row_i >= rows {
            break;
        }
        let rc = unsafe {
            ffi::ghostty_render_state_row_get(
                row_iter,
                GhosttyRenderStateRowData::GHOSTTY_RENDER_STATE_ROW_DATA_CELLS,
                (&raw mut cells_handle).cast::<c_void>(),
            )
        };
        if rc != GHOSTTY_SUCCESS {
            break;
        }

        let mut col_i = 0u16;
        while unsafe { ffi::ghostty_render_state_row_cells_next(cells_handle) } {
            if col_i >= cols {
                break;
            }
            let cell = read_cell(cells_handle, default_fg, default_bg);
            cells.push(cell);
            col_i += 1;
        }
        // Pad short rows.
        while col_i < cols {
            cells.push(TerminalCell {
                fg: default_fg,
                bg: default_bg,
                width: 1,
                is_spacer: false,
                ..TerminalCell::default()
            });
            col_i += 1;
        }

        // Clear per-row dirty (official example pattern).
        let clean = false;
        let _ = unsafe {
            ffi::ghostty_render_state_row_set(
                row_iter,
                GhosttyRenderStateRowOption::GHOSTTY_RENDER_STATE_ROW_OPTION_DIRTY,
                (&raw const clean).cast::<c_void>(),
            )
        };
        row_i += 1;
    }

    // Pad missing rows.
    while row_i < rows {
        for _ in 0..cols {
            cells.push(TerminalCell {
                fg: default_fg,
                bg: default_bg,
                width: 1,
                is_spacer: false,
                ..TerminalCell::default()
            });
        }
        row_i += 1;
    }

    // Reset global dirty.
    let clean_state = GhosttyRenderStateDirty::GHOSTTY_RENDER_STATE_DIRTY_FALSE;
    let _ = unsafe {
        ffi::ghostty_render_state_set(
            render,
            GhosttyRenderStateOption::GHOSTTY_RENDER_STATE_OPTION_DIRTY,
            (&raw const clean_state).cast::<c_void>(),
        )
    };

    unsafe {
        ffi::ghostty_render_state_row_cells_free(cells_handle);
        ffi::ghostty_render_state_row_iterator_free(row_iter);
    }

    Ok(TerminalFrame {
        cols,
        rows,
        default_fg,
        default_bg,
        cells,
        cursor,
        scrollbar,
    })
}

#[cfg(not(ghostty_vt_stub))]
fn read_scrollbar(terminal: GhosttyTerminal, viewport_rows: u16) -> TerminalScrollbar {
    use crate::ffi::{ghostty_terminal_get, GhosttyTerminalData, GhosttyTerminalScrollbar};
    let mut bar = GhosttyTerminalScrollbar {
        total: viewport_rows as u64,
        offset: 0,
        len: viewport_rows as u64,
    };
    let mut active = true;
    unsafe {
        let _ = ghostty_terminal_get(
            terminal,
            GhosttyTerminalData::GHOSTTY_TERMINAL_DATA_SCROLLBAR,
            (&raw mut bar).cast::<c_void>(),
        );
        let _ = ghostty_terminal_get(
            terminal,
            GhosttyTerminalData::GHOSTTY_TERMINAL_DATA_VIEWPORT_ACTIVE,
            (&raw mut active).cast::<c_void>(),
        );
    }
    TerminalScrollbar {
        total: bar.total.max(1),
        offset: bar.offset,
        len: if bar.len == 0 {
            viewport_rows as u64
        } else {
            bar.len
        },
        viewport_active: active,
    }
}

#[cfg(not(ghostty_vt_stub))]
fn read_cursor(render: GhosttyRenderState, default_fg: u32) -> TerminalCursor {
    let mut visible = false;
    let mut in_viewport = false;
    let mut blinking = false;
    let mut col: u16 = 0;
    let mut row: u16 = 0;
    let mut style =
        GhosttyRenderStateCursorVisualStyle::GHOSTTY_RENDER_STATE_CURSOR_VISUAL_STYLE_BLOCK;
    let mut cursor_color = GhosttyColorRgb { r: 0, g: 0, b: 0 };
    let mut has_color = false;

    // SAFETY: out pointers match documented types for each data key.
    unsafe {
        let _ = ffi::ghostty_render_state_get(
            render,
            GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_CURSOR_VISIBLE,
            (&raw mut visible).cast::<c_void>(),
        );
        let _ = ffi::ghostty_render_state_get(
            render,
            GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_CURSOR_VIEWPORT_HAS_VALUE,
            (&raw mut in_viewport).cast::<c_void>(),
        );
        let _ = ffi::ghostty_render_state_get(
            render,
            GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_CURSOR_BLINKING,
            (&raw mut blinking).cast::<c_void>(),
        );
        if in_viewport {
            let _ = ffi::ghostty_render_state_get(
                render,
                GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_CURSOR_VIEWPORT_X,
                (&raw mut col).cast::<c_void>(),
            );
            let _ = ffi::ghostty_render_state_get(
                render,
                GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_CURSOR_VIEWPORT_Y,
                (&raw mut row).cast::<c_void>(),
            );
        }
        let _ = ffi::ghostty_render_state_get(
            render,
            GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_CURSOR_VISUAL_STYLE,
            (&raw mut style).cast::<c_void>(),
        );
        let _ = ffi::ghostty_render_state_get(
            render,
            GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_COLOR_CURSOR_HAS_VALUE,
            (&raw mut has_color).cast::<c_void>(),
        );
        if has_color {
            let _ = ffi::ghostty_render_state_get(
                render,
                GhosttyRenderStateData::GHOSTTY_RENDER_STATE_DATA_COLOR_CURSOR,
                (&raw mut cursor_color).cast::<c_void>(),
            );
        }
    }

    let visual = match style {
        GhosttyRenderStateCursorVisualStyle::GHOSTTY_RENDER_STATE_CURSOR_VISUAL_STYLE_BAR => {
            CursorVisualStyle::Bar
        }
        GhosttyRenderStateCursorVisualStyle::GHOSTTY_RENDER_STATE_CURSOR_VISUAL_STYLE_UNDERLINE => {
            CursorVisualStyle::Underline
        }
        GhosttyRenderStateCursorVisualStyle::GHOSTTY_RENDER_STATE_CURSOR_VISUAL_STYLE_BLOCK_HOLLOW => {
            CursorVisualStyle::BlockHollow
        }
        _ => CursorVisualStyle::Block,
    };

    TerminalCursor {
        col,
        row,
        visible: visible && in_viewport,
        blinking,
        style: visual,
        color: if has_color {
            Some(rgb_to_argb(cursor_color.r, cursor_color.g, cursor_color.b))
        } else {
            let _ = default_fg;
            None
        },
    }
}

#[cfg(not(ghostty_vt_stub))]
fn read_cell(cells: GhosttyRenderStateRowCells, default_fg: u32, default_bg: u32) -> TerminalCell {
    use crate::ffi::{ghostty_cell_get, GhosttyCell, GhosttyCellData, GhosttyCellWide};

    let mut style: GhosttyStyle = unsafe { std::mem::zeroed() };
    style.size = std::mem::size_of::<GhosttyStyle>();
    let mut selected = false;
    let mut fg_rgb = GhosttyColorRgb { r: 0, g: 0, b: 0 };
    let mut bg_rgb = GhosttyColorRgb { r: 0, g: 0, b: 0 };
    let mut raw: GhosttyCell = 0;

    // UTF-8 grapheme via GhosttyBuffer (base + combining). 64 bytes is
    // enough for typical CJK / ZWJ emoji clusters in a single cell.
    let mut utf8_storage = [0u8; 64];
    let mut buf = GhosttyBuffer {
        ptr: utf8_storage.as_mut_ptr(),
        cap: utf8_storage.len(),
        len: 0,
    };

    // SAFETY: cells positioned by row_cells_next; out types match headers.
    let (has_fg, has_bg) = unsafe {
        let _ = ffi::ghostty_render_state_row_cells_get(
            cells,
            GhosttyRenderStateRowCellsData::GHOSTTY_RENDER_STATE_ROW_CELLS_DATA_RAW,
            (&raw mut raw).cast::<c_void>(),
        );
        let _ = ffi::ghostty_render_state_row_cells_get(
            cells,
            GhosttyRenderStateRowCellsData::GHOSTTY_RENDER_STATE_ROW_CELLS_DATA_STYLE,
            (&raw mut style).cast::<c_void>(),
        );
        let _ = ffi::ghostty_render_state_row_cells_get(
            cells,
            GhosttyRenderStateRowCellsData::GHOSTTY_RENDER_STATE_ROW_CELLS_DATA_SELECTED,
            (&raw mut selected).cast::<c_void>(),
        );
        let rc_fg = ffi::ghostty_render_state_row_cells_get(
            cells,
            GhosttyRenderStateRowCellsData::GHOSTTY_RENDER_STATE_ROW_CELLS_DATA_FG_COLOR,
            (&raw mut fg_rgb).cast::<c_void>(),
        );
        let rc_bg = ffi::ghostty_render_state_row_cells_get(
            cells,
            GhosttyRenderStateRowCellsData::GHOSTTY_RENDER_STATE_ROW_CELLS_DATA_BG_COLOR,
            (&raw mut bg_rgb).cast::<c_void>(),
        );
        let _ = ffi::ghostty_render_state_row_cells_get(
            cells,
            GhosttyRenderStateRowCellsData::GHOSTTY_RENDER_STATE_ROW_CELLS_DATA_GRAPHEMES_UTF8,
            (&raw mut buf).cast::<c_void>(),
        );
        (rc_fg == GHOSTTY_SUCCESS, rc_bg == GHOSTTY_SUCCESS)
    };

    // Ghostty stores one cell per grid column. Wide CJK occupies two columns:
    // primary cell (WIDE) + spacer tail (do not paint). See GhosttyCellWide.
    let mut wide = GhosttyCellWide::GHOSTTY_CELL_WIDE_NARROW;
    unsafe {
        let _ = ghostty_cell_get(
            raw,
            GhosttyCellData::GHOSTTY_CELL_DATA_WIDE,
            (&raw mut wide).cast::<c_void>(),
        );
    }

    let grapheme = if buf.len > 0 && buf.len <= utf8_storage.len() {
        String::from_utf8_lossy(&utf8_storage[..buf.len]).into_owned()
    } else {
        String::new()
    };

    let (width, is_spacer) = match wide {
        GhosttyCellWide::GHOSTTY_CELL_WIDE_WIDE => (2u8, false),
        GhosttyCellWide::GHOSTTY_CELL_WIDE_SPACER_TAIL
        | GhosttyCellWide::GHOSTTY_CELL_WIDE_SPACER_HEAD => (0u8, true),
        _ => {
            // Fallback: measure first codepoint when RAW wide is unavailable.
            let w = grapheme
                .chars()
                .next()
                .map(|c| {
                    let w = unsafe { ffi::ghostty_unicode_codepoint_width(c as u32) };
                    if w >= 2 {
                        2
                    } else if w == 0 && !grapheme.is_empty() {
                        1
                    } else {
                        w.max(1)
                    }
                })
                .unwrap_or(1);
            (w, false)
        }
    };

    let fg = if has_fg {
        rgb_to_argb(fg_rgb.r, fg_rgb.g, fg_rgb.b)
    } else {
        default_fg
    };
    let bg = if has_bg {
        rgb_to_argb(bg_rgb.r, bg_rgb.g, bg_rgb.b)
    } else {
        default_bg
    };

    TerminalCell {
        // Spacers never paint text (even if Ghostty left a leftover).
        grapheme: if is_spacer { String::new() } else { grapheme },
        fg,
        bg,
        bold: style.bold,
        italic: style.italic,
        faint: style.faint,
        underline: style.underline != 0,
        strikethrough: style.strikethrough,
        inverse: style.inverse,
        selected,
        width,
        is_spacer,
    }
}

/// Stub capture for host builds without libghostty-vt.
#[cfg(ghostty_vt_stub)]
pub fn capture_frame_stub(cols: u16, rows: u16, banner: &str) -> TerminalFrame {
    let default_fg = 0xFFE2_E8F0;
    let default_bg = 0xFF0B_1220;
    let mut cells = vec![
        TerminalCell {
            fg: default_fg,
            bg: default_bg,
            ..TerminalCell::default()
        };
        cols as usize * rows as usize
    ];
    // Paint first line; CJK takes two columns like Ghostty.
    let mut col = 0usize;
    for ch in banner.chars() {
        if ch == '\n' || ch == '\r' || col >= cols as usize {
            break;
        }
        let wide = crate::frame::east_asian_width(ch) >= 2;
        let w = if wide { 2 } else { 1 };
        if col + w > cols as usize {
            break;
        }
        cells[col].grapheme = ch.to_string();
        cells[col].width = w as u8;
        cells[col].is_spacer = false;
        if wide && col + 1 < cols as usize {
            cells[col + 1].grapheme.clear();
            cells[col + 1].width = 0;
            cells[col + 1].is_spacer = true;
        }
        col += w;
    }
    // Green "styles" marker so demos show color under stub.
    if cols > 10 {
        for (offset, ch) in "styles".chars().enumerate() {
            let idx = 8 + offset;
            if idx < cols as usize {
                cells[idx].grapheme = ch.to_string();
                cells[idx].fg = 0xFF0D_BC79;
                cells[idx].bold = true;
            }
        }
    }
    TerminalFrame {
        cols,
        rows,
        default_fg,
        default_bg,
        cells,
        cursor: TerminalCursor {
            col: 0,
            row: 0,
            visible: true,
            blinking: false,
            style: CursorVisualStyle::Block,
            color: None,
        },
        scrollbar: TerminalScrollbar {
            total: rows as u64,
            offset: 0,
            len: rows as u64,
            viewport_active: true,
        },
    }
}
