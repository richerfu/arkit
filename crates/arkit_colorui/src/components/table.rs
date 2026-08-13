//! Table — `.cu-list.menu` rows, hairline `#ddd`.

use arkit_prelude::*;

use crate::spec;

#[component]
pub fn Table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Element {
    let total = rows.len();
    let header_cells: Vec<Element> = headers
        .iter()
        .map(|header| {
            rsx! {
                row {
                    layout_weight: 1.0,
                    height: spec::LIST_ITEM,
                    align_items: "center",
                    padding_left: spec::PADDING,
                    padding_right: spec::PADDING,
                    text {
                        content: header.clone(),
                        font_size: spec::TEXT_SM,
                        font_color: spec::TEXT_MUTED,
                    }
                }
            }
        })
        .collect();
    let body: Vec<Element> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let cells: Vec<Element> = row
                .iter()
                .map(|cell| {
                    rsx! {
                        row {
                            layout_weight: 1.0,
                            align_items: "center",
                            padding_left: spec::PADDING,
                            padding_right: spec::PADDING,
                            min_height: spec::LIST_ITEM,
                            text {
                                content: cell.clone(),
                                font_size: spec::TEXT_DF,
                                font_color: spec::TEXT,
                            }
                        }
                    }
                })
                .collect();
            rsx! {
                row {
                    width: "100%",
                    background_color: spec::BG_WHITE,
                    border_width: if index + 1 == total { 0.0 } else { 0.0 },
                    {cells.into_iter()}
                }
            }
        })
        .collect();

    rsx! {
        column {
            width: "100%",
            background_color: spec::BG_WHITE,
            row {
                width: "100%",
                background_color: spec::PAGE_BG,
                {header_cells.into_iter()}
            }
            {body.into_iter()}
        }
    }
}
