//! Complex cases — List / Grid / WaterFlow virtualized via ArkUI `NodeAdapter`.
//!
//! `use_virtual_items` binds stable identity/revision snapshots through the
//! host's `virtual_source` attribute so only visible items are created on
//! demand. List/Grid/WaterFlow exercise identity-preserving moves and targeted
//! reloads; Native exercises the manually controlled low-level source.

use arkit::dioxus_signals::WritableExt;
use arkit::native::NodeBuilder;
use arkit::prelude::*;
use dioxus_core_macro::{component, Props};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    List,
    Grid,
    WaterFlow,
    Native,
}

const TOTAL: u32 = 10_000;

#[derive(Clone, Copy)]
struct DemoItem {
    id: u32,
    revision: u32,
}

#[component]
pub fn ComplexCasesPage() -> Element {
    let mut active = use_signal(|| Case::List);
    let cur = active();

    rsx! {
        column {
            width: "100%",
            height: "100%",
            background_color: "#fff8fafc",

            row {
                width: "100%",
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
                button {
                    margin_left: 8.0,
                    font_size: 14.0,
                    background_color: if cur == Case::Native { "#ff111827" } else { "#ffffffff" },
                    font_color: if cur == Case::Native { "#ffffffff" } else { "#ff111827" },
                    onclick: move |_| active.set(Case::Native),
                    "Native"
                }
            }

            text {
                padding: 16.0,
                font_size: 13.0,
                font_color: "#ff475569",
                "total {TOTAL} (NodeAdapter + RSX / NodeBuilder)"
            }

            VirtualCaseView { kind: VirtualKind::List, active: cur == Case::List }
            VirtualCaseView { kind: VirtualKind::Grid, active: cur == Case::Grid }
            VirtualCaseView { kind: VirtualKind::WaterFlow, active: cur == Case::WaterFlow }
            NativeVirtualCaseView { active: cur == Case::Native }
        }
    }
}

#[component]
fn VirtualCaseView(kind: VirtualKind, active: bool) -> Element {
    let mut items = use_signal(|| {
        (0..TOTAL)
            .map(|id| DemoItem { id, revision: 0 })
            .collect::<Vec<_>>()
    });
    let mut target = use_signal(|| 2_u32);
    let mut next_id = use_signal(|| TOTAL);
    let mut status = use_signal(|| "先点目标行累计 taps，再移动+更新；taps 应保留".to_string());
    let snapshot = items();
    let target_id = target();
    let target_index = snapshot
        .iter()
        .position(|item| item.id == target_id)
        .unwrap_or_default();
    let target_revision = snapshot[target_index].revision;
    let stamps = snapshot
        .iter()
        .map(|item| VirtualItemStamp::new(item.id, item.revision))
        .collect();
    let render_items = snapshot.clone();
    let source = use_virtual_items(kind, stamps, move |index| {
        let item = render_items[index as usize];
        render_virtual_item(kind, item.id, index, item.revision)
    });
    let height = if active { "100%" } else { "0%" };
    let visibility = if active { "visible" } else { "hidden" };
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
    let update_target = move |_| {
        let id = *target.peek();
        let mut revision = 0;
        items.with_mut(|items| {
            let item = items
                .iter_mut()
                .find(|item| item.id == id)
                .expect("selected virtual demo item disappeared");
            item.revision = item.revision.wrapping_add(1);
            revision = item.revision;
        });
        status.set(format!(
            "{kind_label} id #{id:05} 原位 Reload 到 rev {revision}"
        ));
    };
    let move_and_update_target = move |_| {
        let id = *target.peek();
        let mut destination = 0;
        let mut revision = 0;
        items.with_mut(|items| {
            let from = items
                .iter()
                .position(|item| item.id == id)
                .expect("selected virtual demo item disappeared");
            let mut item = items.remove(from);
            item.revision = item.revision.wrapping_add(1);
            revision = item.revision;
            destination = (from + 2).min(items.len());
            items.insert(destination, item);
        });
        status.set(format!(
            "{kind_label} id #{id:05} Move 到 {destination} + Reload rev {revision}；taps 应不变"
        ));
    };
    let remove_target = move |_| {
        let id = *target.peek();
        let mut next_target = id;
        let mut count = 0;
        items.with_mut(|items| {
            if items.len() <= 1 {
                return;
            }
            let index = items
                .iter()
                .position(|item| item.id == id)
                .expect("selected virtual demo item disappeared");
            items.remove(index);
            next_target = items[index.min(items.len() - 1)].id;
            count = items.len();
        });
        target.set(next_target);
        status.set(format!(
            "{kind_label} 删除 id #{id:05}，count {count}；其他行 taps 应保留"
        ));
    };
    let insert_fresh = move |_| {
        let id = *next_id.peek();
        next_id.set(id.wrapping_add(1));
        let selected = *target.peek();
        let mut index = 0;
        items.with_mut(|items| {
            index = items
                .iter()
                .position(|item| item.id == selected)
                .map_or(items.len(), |position| position + 1);
            items.insert(index, DemoItem { id, revision: 0 });
        });
        target.set(id);
        status.set(format!(
            "{kind_label} 插入全新 id #{id:05} @ {index}；taps 必须从 0 开始"
        ));
    };

    rsx! {
        column {
            width: "100%",
            height: height,
            visibility: visibility,
            opacity,
            column {
                width: "100%",
                height: 88.0,
                padding: 8.0,
                background_color: "#fff1f5f9",
                text {
                    width: "100%",
                    height: 34.0,
                    font_size: 12.0,
                    font_color: "#ff475569",
                    "{kind_label} id #{target_id:05} · index {target_index} · rev {target_revision}\n{status}"
                }
                row {
                    width: "100%",
                    height: 38.0,
                    alignment: "top",
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
                        background_color: "#ff7c3aed",
                        font_color: "#ffffffff",
                        onclick: move_and_update_target,
                        "移动+更新"
                    }
                    button {
                        margin_left: 6.0,
                        font_size: 12.0,
                        padding: 8.0,
                        background_color: "#ffdc2626",
                        font_color: "#ffffffff",
                        onclick: remove_target,
                        "删除"
                    }
                    button {
                        margin_left: 6.0,
                        font_size: 12.0,
                        padding: 8.0,
                        background_color: "#ff059669",
                        font_color: "#ffffffff",
                        onclick: insert_fresh,
                        "插入新ID"
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
            VirtualItemsHost { kind, source }
        }
    }
}

#[component]
fn NativeVirtualCaseView(active: bool) -> Element {
    let source = use_virtual_source(VirtualKind::List, TOTAL, move |index| {
        Ok(NodeBuilder::new("text")?
            .percent_width(1.0)?
            .height(44.0)?
            .padding([12.0, 12.0, 12.0, 12.0])?
            .background_color(if index % 2 == 0 {
                "#ffffffff"
            } else {
                "#fff8fafc"
            })?
            .font_size(14.0)?
            .font_color("#ff334155")?
            .text_content(format!("Native #{index:05}"))?
            .build())
    });
    let height = if active { "100%" } else { "0%" };
    let visibility = if active { "visible" } else { "hidden" };
    let opacity = if active { 1.0 } else { 0.0 };

    rsx! {
        column {
            width: "100%",
            height,
            visibility,
            opacity,
            VirtualHost {
                source,
            }
        }
    }
}

#[component]
fn VirtualHost(source: VirtualSource) -> Element {
    rsx! {
        list {
            virtual_source: source,
            width: "100%",
            layout_weight: 1.0,
            scroll_bar: "off",
        }
    }
}

#[component]
fn VirtualItemsHost(kind: VirtualKind, source: VirtualItems) -> Element {
    match kind {
        VirtualKind::List => rsx! {
            list {
                virtual_source: source,
                width: "100%",
                layout_weight: 1.0,
                scroll_bar: "off",
            }
        },
        VirtualKind::Grid => rsx! {
            grid {
                virtual_source: source,
                width: "100%",
                layout_weight: 1.0,
                grid_column_template: "1fr 1fr",
                scroll_bar: "off",
            }
        },
        VirtualKind::WaterFlow => rsx! {
            waterflow {
                virtual_source: source,
                width: "100%",
                layout_weight: 1.0,
                scroll_bar: "off",
                padding: 12.0,
                water_flow_column_template: "repeat(auto-fill, 104vp)",
                water_flow_column_gap: 12.0,
                water_flow_row_gap: 12.0,
                water_flow_cached_count: 6_i32,
            }
        },
    }
}

fn render_virtual_item(kind: VirtualKind, id: u32, index: u32, revision: u32) -> Element {
    let mut taps = use_signal(|| 0_u32);
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
        format!(
            "id #{id:05} @ {index}\nrev {revision} · {height:.0}vp · taps {}",
            taps()
        )
    } else {
        format!("id #{id:05} @ {index} · rev {revision} · taps {}", taps())
    };

    let width = if kind == VirtualKind::WaterFlow {
        // Native WaterFlow measures percentage widths against the host rather
        // than the generated FlowItem. Match the explicit auto-fill track.
        "104vp"
    } else {
        "100%"
    };
    rsx! {
        row {
            width: width,
            height,
            background_color,
            padding: 12.0,
            onclick: move |_| taps += 1,
            text {
                font_size: 14.0,
                font_color: "#ff334155",
                "{label}"
            }
        }
    }
}

fn virtual_kind_label(kind: VirtualKind) -> &'static str {
    match kind {
        VirtualKind::List => "List",
        VirtualKind::Grid => "Grid",
        VirtualKind::WaterFlow => "WaterFlow",
    }
}
