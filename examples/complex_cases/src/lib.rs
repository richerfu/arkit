//! Complex cases — List / Grid / WaterFlow virtualized via ArkUI `NodeAdapter`.
//!
//! `use_virtual_node_adapter` attaches the matching adapter to the native host
//! so only visible items are created on demand (true virtualization). Each
//! container also demonstrates item-local invalidation: updating one revision
//! calls `reload_items(index, 1)` and rebuilds only that native item.

use std::cell::RefCell;
use std::rc::Rc;

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

            VirtualCaseView { kind: VirtualKind::List, active: cur == Case::List }
            VirtualCaseView { kind: VirtualKind::Grid, active: cur == Case::Grid }
            VirtualCaseView { kind: VirtualKind::WaterFlow, active: cur == Case::WaterFlow }
        }
    }
}

#[component]
fn VirtualCaseView(kind: VirtualKind, active: bool) -> Element {
    let revisions = use_hook(|| Rc::new(RefCell::new(vec![0_u32; TOTAL as usize])));
    let render_revisions = revisions.clone();
    let adapter = use_virtual_node_adapter(kind, TOTAL, move |index| {
        let revision = render_revisions.borrow()[index as usize];
        render_virtual_item(kind, index, revision)
    });
    let mut target = use_signal(|| 2_u32);
    let mut status = use_signal(|| "点击更新只会重建目标 item".to_string());
    let target_index = target();
    let target_revision = revisions.borrow()[target_index as usize];
    let height = if active { 1.0 } else { 0.0 };
    let visibility = if active { 0 } else { 1 };
    let opacity = if active { 1.0 } else { 0.0 };
    let kind_label = virtual_kind_label(kind);

    let previous_target = move |_| {
        let current = *target.peek();
        target.set(if current == 0 { TOTAL - 1 } else { current - 1 });
    };
    let next_target = move |_| {
        let current = *target.peek();
        target.set((current + 1) % TOTAL);
    };
    let update_revisions = revisions.clone();
    let update_adapter = adapter.clone();
    let update_target = move |_| {
        let index = *target.peek();
        let revision = {
            let mut revisions = update_revisions.borrow_mut();
            let revision = &mut revisions[index as usize];
            *revision = revision.wrapping_add(1);
            *revision
        };
        match update_adapter.reload_items(index, 1) {
            Ok(()) => status.set(format!(
                "{kind_label} #{index:05} 已局部更新到 rev {revision}"
            )),
            Err(error) => status.set(format!("局部更新失败: {error}")),
        }
    };

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: height,
            visibility,
            opacity,
            column {
                percent_width: 1.0,
                height: 88.0,
                padding: 8.0,
                background_color: "#fff1f5f9",
                text {
                    percent_width: 1.0,
                    height: 34.0,
                    font_size: 12.0,
                    font_color: "#ff475569",
                    "{kind_label} #{target_index:05} · rev {target_revision}\n{status}"
                }
                row {
                    percent_width: 1.0,
                    height: 38.0,
                    alignment: 1,
                    button {
                        font_size: 12.0,
                        padding: 8.0,
                        background_color: "#ffffffff",
                        font_color: "#ff334155",
                        onclick: previous_target,
                        "上一项"
                    }
                    button {
                        margin_left: 6.0,
                        font_size: 12.0,
                        padding: 8.0,
                        background_color: "#ff2563eb",
                        font_color: "#ffffffff",
                        onclick: update_target,
                        "更新单项"
                    }
                    button {
                        margin_left: 6.0,
                        font_size: 12.0,
                        padding: 8.0,
                        background_color: "#ffffffff",
                        font_color: "#ff334155",
                        onclick: next_target,
                        "下一项"
                    }
                }
            }
            VirtualHost { kind, adapter }
        }
    }
}

#[component]
fn VirtualHost(kind: VirtualKind, adapter: VirtualNodeAdapter) -> Element {
    use_attach_virtual_adapter(adapter);
    match kind {
        VirtualKind::List => rsx! {
            list {
                percent_width: 1.0,
                layout_weight: 1.0,
            }
        },
        VirtualKind::Grid => rsx! {
            grid {
                percent_width: 1.0,
                layout_weight: 1.0,
                grid_column_template: "1fr 1fr",
            }
        },
        VirtualKind::WaterFlow => rsx! {
            waterflow {
                percent_width: 1.0,
                layout_weight: 1.0,
                padding: 12.0,
                water_flow_column_template: "repeat(auto-fill, 104vp)",
                water_flow_column_gap: 12.0,
                water_flow_row_gap: 12.0,
                water_flow_cached_count: 6_i32,
            }
        },
    }
}

fn use_attach_virtual_adapter(adapter: VirtualNodeAdapter) {
    let attach_adapter = adapter.clone();
    use_layout_frame_node(move |host_node, _frame| {
        let _ = attach_adapter.attach(&host_node);
    });
}

fn render_virtual_item(
    kind: VirtualKind,
    index: u32,
    revision: u32,
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
            let variant = (index as usize + revision as usize) % HEIGHTS.len();
            (HEIGHTS[variant], COLORS[variant])
        }
        VirtualKind::List | VirtualKind::Grid => {
            const COLORS: [&str; 5] = [
                "#ffffffff",
                "#ffdbeafe",
                "#ffdcfce7",
                "#fffef3c7",
                "#fffce7f3",
            ];
            let variant = revision as usize % COLORS.len();
            (44.0, COLORS[variant])
        }
    };
    let label = if kind == VirtualKind::WaterFlow {
        format!("#{index:05}\nrev {revision} · {height:.0}vp")
    } else {
        format!("#{index:05} · rev {revision}")
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

fn virtual_kind_label(kind: VirtualKind) -> &'static str {
    match kind {
        VirtualKind::List => "List",
        VirtualKind::Grid => "Grid",
        VirtualKind::WaterFlow => "WaterFlow",
    }
}
