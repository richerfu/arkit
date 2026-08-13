//! Table — official bordered card + 40px header + hairline rows.

use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

#[component]
pub fn Table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Element {
    let theme = use_theme();
    let total = rows.len();
    let header_cells: Vec<Element> = headers
        .iter()
        .map(|header| {
            rsx! {
                row {
                    layout_weight: 1.0,
                    height: 40.0,
                    align_items: "center",
                    padding_left: 8.0,
                    padding_right: 8.0,
                    text {
                        content: header.clone(),
                        font_size: spec::TEXT_SM,
                        font_weight: spec::FONT_MEDIUM,
                        font_color: theme.colors.muted_foreground,
                    }
                }
            }
        })
        .collect();
    let body: Vec<Element> = rows
        .iter()
        .enumerate()
        .map(|(_index, row)| {
            let cells: Vec<Element> = row
                .iter()
                .map(|cell| {
                    rsx! {
                        row {
                            layout_weight: 1.0,
                            align_items: "center",
                            padding: 8.0,
                            text {
                                content: cell.clone(),
                                font_size: spec::TEXT_SM,
                                font_color: theme.colors.foreground,
                            }
                        }
                    }
                })
                .collect();
            let _ = total;
            rsx! {
                row {
                    width: "100%",
                    {cells.into_iter()}
                }
            }
        })
        .collect();
    rsx! {
        column {
            width: "100%",
            background_color: theme.colors.card,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: spec::RADIUS_MD,
            clip: true,
            row { width: "100%", {header_cells.into_iter()} }
            {body.into_iter()}
        }
    }
}
