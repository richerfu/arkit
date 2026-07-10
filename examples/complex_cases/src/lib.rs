//! Complex cases — List / Grid virtualized via ArkUI `NodeAdapter`.
//!
//! `use_virtual_list` attaches a `NodeAdapter` to the host `list`/`grid` node
//! so only visible items are created on demand (true virtualization). 10 000
//! items scroll smoothly; `render_item` builds each item's content node lazily
//! via `NodeBuilder` (no raw binding calls).

use arkit::dioxus_signals::WritableExt;
use arkit::entry;
use arkit::prelude::*;
use dioxus_core_macro::{component, Props};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    List,
    Grid,
}

const TOTAL: u32 = 10_000;

#[entry]
fn app() -> Element {
    let mut active = use_signal(|| Case::List);
    let cur = active();

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            background_color: "#fff8fafc",

            row {
                percent_width: 1.0,
                padding: 16.0,
                button {
                    font_size: 14.0,
                    background_color: if cur == Case::List { "#ff111827" } else { "#ffffffff" },
                    font_color: if cur == Case::List { "#ffffffff" } else { "#ff111827" },
                    onclick: move |_| active.set(Case::List),
                    "List"
                }
                button {
                    margin_left: 8.0,
                    font_size: 14.0,
                    background_color: if cur == Case::Grid { "#ff111827" } else { "#ffffffff" },
                    font_color: if cur == Case::Grid { "#ffffffff" } else { "#ff111827" },
                    onclick: move |_| active.set(Case::Grid),
                    "Grid"
                }
            }

            text {
                padding: 16.0,
                font_size: 13.0,
                font_color: "#ff475569",
                "total {TOTAL} (NodeAdapter virtualized)"
            }

            VirtualListView { active: cur == Case::List }
            VirtualGridView { active: cur == Case::Grid }
        }
    }
}

#[component]
fn VirtualListView(active: bool) -> Element {
    use_virtual_adapter(VirtualKind::List);
    let height = if active { 1.0 } else { 0.0 };
    let visibility = if active { 0 } else { 1 };
    let opacity = if active { 1.0 } else { 0.0 };

    rsx! {
        list {
            percent_width: 1.0,
            percent_height: height,
            visibility,
            opacity,
        }
    }
}

#[component]
fn VirtualGridView(active: bool) -> Element {
    use_virtual_adapter(VirtualKind::Grid);
    let height = if active { 1.0 } else { 0.0 };
    let visibility = if active { 0 } else { 1 };
    let opacity = if active { 1.0 } else { 0.0 };

    rsx! {
        grid {
            percent_width: 1.0,
            percent_height: height,
            visibility,
            opacity,
            grid_column_template: "1fr 1fr",
        }
    }
}

fn use_virtual_adapter(kind: VirtualKind) {
    let handle = use_virtual_list(kind, TOTAL, move |index| {
        let label = format!("#{index:05}");
        let text = NodeBuilder::new("text")?
            .font_size(14.0)?
            .font_color("#ff334155")?
            .text_content(label)?
            .build();
        NodeBuilder::new("row")?
            .percent_width(1.0)?
            .height(44.0)?
            .padding([4.0, 12.0, 4.0, 12.0])?
            .child(text)
            .map(NodeBuilder::build)
    });

    let attach_handle = handle.clone();
    use_layout_frame_node(move |host_node, _frame| {
        let _ = attach_handle.attach(&host_node);
    });
}
