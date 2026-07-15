//! Complex cases — List / Grid / WaterFlow virtualized via ArkUI `NodeAdapter`.
//!
//! `use_virtual_node_adapter` and `use_virtual_water_flow` attach the matching
//! adapter to the native host so only visible items are created on demand
//! (true virtualization). 10 000 items scroll smoothly; `render_item` builds
//! each item's content node lazily via `NodeBuilder` (no raw binding calls).

use arkit::dioxus_signals::WritableExt;
use arkit::entry;
use arkit::prelude::*;
use dioxus_core_macro::{component, Props};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    List,
    Grid,
    WaterFlow,
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
                button {
                    margin_left: 8.0,
                    font_size: 14.0,
                    background_color: if cur == Case::WaterFlow { "#ff111827" } else { "#ffffffff" },
                    font_color: if cur == Case::WaterFlow { "#ffffffff" } else { "#ff111827" },
                    onclick: move |_| active.set(Case::WaterFlow),
                    "WaterFlow"
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
            VirtualWaterFlowView { active: cur == Case::WaterFlow }
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

#[component]
fn VirtualWaterFlowView(active: bool) -> Element {
    let handle = use_virtual_water_flow(TOTAL, move |index| {
        render_virtual_item(VirtualKind::WaterFlow, index)
    });
    use_attach_virtual_adapter(handle);
    let height = if active { 1.0 } else { 0.0 };
    let visibility = if active { 0 } else { 1 };
    let opacity = if active { 1.0 } else { 0.0 };

    rsx! {
        waterflow {
            percent_width: 1.0,
            percent_height: height,
            visibility,
            opacity,
            padding: 12.0,
            water_flow_column_template: "repeat(auto-fill, 104vp)",
            water_flow_column_gap: 12.0,
            water_flow_row_gap: 12.0,
            water_flow_cached_count: 6_i32,
        }
    }
}

fn use_virtual_adapter(kind: VirtualKind) {
    let handle =
        use_virtual_node_adapter(kind, TOTAL, move |index| render_virtual_item(kind, index));
    use_attach_virtual_adapter(handle);
}

fn use_attach_virtual_adapter(handle: VirtualNodeAdapterHandle) {
    let attach_handle = handle.clone();
    use_layout_frame_node(move |host_node, _frame| {
        let _ = attach_handle.attach(&host_node);
    });
}

fn render_virtual_item(
    kind: VirtualKind,
    index: u32,
) -> arkit::ohos_arkui_binding::common::error::ArkUIResult<
    arkit::ohos_arkui_binding::common::node::ArkUINode,
> {
    let (height, background_color) = match kind {
        VirtualKind::WaterFlow => {
            const HEIGHTS: [f32; 5] = [88.0, 120.0, 152.0, 104.0, 136.0];
            const COLORS: [&str; 5] = [
                "#ffe0f2fe",
                "#ffdcfce7",
                "#fffef3c7",
                "#fffce7f3",
                "#ffede9fe",
            ];
            let variant = index as usize % HEIGHTS.len();
            (HEIGHTS[variant], COLORS[variant])
        }
        VirtualKind::List | VirtualKind::Grid => (44.0, "#ffffffff"),
    };
    let label = if kind == VirtualKind::WaterFlow {
        format!("#{index:05}\n{height:.0}vp")
    } else {
        format!("#{index:05}")
    };
    let text = NodeBuilder::new("text")?
        .font_size(14.0)?
        .font_color("#ff334155")?
        .text_content(label)?
        .build();
    let item = NodeBuilder::new("row")?
        .height(height)?
        .background_color(background_color)?
        .padding([12.0, 12.0, 12.0, 12.0])?
        .child(text)?;
    let item = if kind == VirtualKind::WaterFlow {
        // Native WaterFlow measures percentage widths against the host rather
        // than the generated FlowItem. Match the explicit auto-fill track to
        // keep the item content inside its wrapper.
        item.width(104.0)?
    } else {
        item.percent_width(1.0)?
    };
    Ok(item.build())
}
