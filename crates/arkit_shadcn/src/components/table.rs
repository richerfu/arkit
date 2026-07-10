//! Table — shadcn-style data table.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original rounded card surface (`card` background,
//! `border`, `sm` radius, clipped), the `40.0`-tall header row with a bottom
//! separator, and the `8.0`-padded body cells with `SM` text.
//!
//! The row separators use ArkUI's four-side border vector so only the bottom
//! edge is drawn, matching the original reusable implementation.

use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Table`].
#[derive(Props, Clone, PartialEq)]
pub struct TableProps {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// A rounded, bordered data table with a header row and body rows.
#[component]
pub fn Table(props: TableProps) -> Element {
    let theme = use_theme();
    let total_rows = props.rows.len();

    let header_cells: Vec<Element> = props
        .headers
        .iter()
        .map(|header| {
            rsx! {
                row {
                    layout_weight: 1.0,
                    height: 40.0,
                    align_items: "center",
                    padding_top: 0.0,
                    padding_right: 8.0,
                    padding_bottom: 0.0,
                    padding_left: 8.0,
                    text {
                        content: header.clone(),
                        font_size: typography::SM,
                        font_weight: 500,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                    }
                }
            }
        })
        .collect();

    let body_rows: Vec<Element> = props
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let is_last = index + 1 == total_rows;
            let border_width = if is_last { "0,0,0,0" } else { "0,0,1,0" }.to_string();
            let cells: Vec<Element> = row
                .iter()
                .map(|cell| {
                    rsx! {
                        row {
                            layout_weight: 1.0,
                            align_items: "center",
                            padding_top: 8.0,
                            padding_right: 8.0,
                            padding_bottom: 8.0,
                            padding_left: 8.0,
                            text {
                                content: cell.clone(),
                                font_size: typography::SM,
                                font_color: theme.colors.foreground,
                                line_height: 20.0,
                            }
                        }
                    }
                })
                .collect();
            rsx! {
                row {
                    key: "{index}",
                    percent_width: 1.0,
                    align_items: "center",
                    border_width: border_width,
                    border_color: theme.colors.border,
                    {cells.into_iter()}
                }
            }
        })
        .collect();

    rsx! {
        column {
            percent_width: 1.0,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.sm,
            background_color: theme.colors.card,
            clip: true,
            row {
                percent_width: 1.0,
                border_width: "0,0,1,0",
                border_color: theme.colors.border,
                {header_cells.into_iter()}
            }
            {body_rows.into_iter()}
        }
    }
}
