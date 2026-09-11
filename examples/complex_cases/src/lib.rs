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
    let mut operation = use_signal(|| 0_u32);
    let mut last_tap = use_signal(|| None::<(u32, u32)>);
    let mut status = use_signal(|| "Tap id #00002, then Rot L/Rot R or Move+Rev".to_string());
    let snapshot = items();
    let count = snapshot.len();
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
        render_virtual_item(kind, item.id, index, item.revision, last_tap)
    });
    let height = if active { "100%" } else { "0%" };
    let visibility = if active { "visible" } else { "hidden" };
    let opacity = if active { 1.0 } else { 0.0 };
    let kind_label = virtual_kind_label(kind);
    let taps_status = last_tap().map_or_else(
        || "last taps none".to_string(),
        |(id, taps)| format!("last taps #{id:05}={taps}"),
    );

    let previous_target = move |_| {
        let current = *target.peek();
        let items = items.peek();
        let index = items
            .iter()
            .position(|item| item.id == current)
            .expect("selected virtual demo item disappeared");
        let next = items[(index + items.len() - 1) % items.len()].id;
        target.set(next);
        operation += 1;
        status.set(format!("Select previous id #{next:05}"));
    };
    let next_target = move |_| {
        let current = *target.peek();
        let items = items.peek();
        let index = items
            .iter()
            .position(|item| item.id == current)
            .expect("selected virtual demo item disappeared");
        let next = items[(index + 1) % items.len()].id;
        target.set(next);
        operation += 1;
        status.set(format!("Select next id #{next:05}"));
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
            "{kind_label} Reload #{id:05} to rev {revision}; taps must stay"
        ));
        operation += 1;
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
            "{kind_label} Move+Reload #{id:05} -> {destination}, rev {revision}; taps must stay"
        ));
        operation += 1;
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
            "{kind_label} Delete #{id:05}; count {count}; retained taps must stay"
        ));
        operation += 1;
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
            "{kind_label} Insert new #{id:05} @ {index}; taps must start at 0"
        ));
        operation += 1;
    };
    let reverse_all = move |_| {
        items.with_mut(|items| items.reverse());
        operation += 1;
        status.set(format!(
            "{kind_label} Reverse all; verify order/count, then Restore"
        ));
    };
    let rotate_left = move |_| {
        items.with_mut(|items| {
            if items.len() > 1 {
                items.rotate_left(1);
            }
        });
        operation += 1;
        status.set(format!(
            "{kind_label} Rot L by 1; visible #00002 taps must stay"
        ));
    };
    let rotate_right = move |_| {
        items.with_mut(|items| {
            if items.len() > 1 {
                items.rotate_right(1);
            }
        });
        operation += 1;
        status.set(format!(
            "{kind_label} Rot R by 1; visible #00002 taps must stay"
        ));
    };
    let shuffle_all = move |_| {
        items.with_mut(|items| deterministic_shuffle(items));
        operation += 1;
        status.set(format!(
            "{kind_label} Shuffle seed 0x5eed; verify order/count, then Restore"
        ));
    };
    let restore = move |_| {
        items.set((0..TOTAL).map(|id| DemoItem { id, revision: 0 }).collect());
        target.set(2);
        next_id.set(TOTAL);
        last_tap.set(None);
        operation += 1;
        status.set(format!(
            "{kind_label} Restore baseline; offscreen taps are not persistent storage"
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
                height: 146.0,
                padding: 8.0,
                background_color: "#fff1f5f9",
                text {
                    width: "100%",
                    height: 58.0,
                    font_size: 11.0,
                    font_color: "#ff475569",
                    "{kind_label} logical op#{operation} · count {count} · target #{target_id:05} @ {target_index} rev {target_revision}\n{taps_status} · {status}\nnative-layout: verify viewport; no completion callback"
                }
                row {
                    width: "100%",
                    height: 34.0,
                    alignment: "top",
                    FixtureButton { label: "Prev", width: "14%", color: "#ffffffff", on_press: previous_target }
                    FixtureButton { label: "Reload", width: "14%", color: "#ff2563eb", on_press: update_target }
                    FixtureButton { label: "Move+Rev", width: "18%", color: "#ff7c3aed", on_press: move_and_update_target }
                    FixtureButton { label: "Delete", width: "14%", color: "#ffdc2626", on_press: remove_target }
                    FixtureButton { label: "New ID", width: "14%", color: "#ff059669", on_press: insert_fresh }
                    FixtureButton { label: "Next", width: "14%", color: "#ffffffff", on_press: next_target }
                }
                row {
                    margin_top: 4.0,
                    width: "100%",
                    height: 34.0,
                    alignment: "top",
                    FixtureButton { label: "Reverse", width: "18%", color: "#ff0f766e", on_press: reverse_all }
                    FixtureButton { label: "Rot L", width: "18%", color: "#ff0369a1", on_press: rotate_left }
                    FixtureButton { label: "Rot R", width: "18%", color: "#ff0369a1", on_press: rotate_right }
                    FixtureButton { label: "Shuffle", width: "18%", color: "#ffc2410c", on_press: shuffle_all }
                    FixtureButton { label: "Restore", width: "18%", color: "#ff334155", on_press: restore }
                }
            }
            VirtualItemsHost { kind, source }
        }
    }
}

#[component]
fn FixtureButton(
    label: &'static str,
    width: &'static str,
    color: &'static str,
    on_press: EventHandler<()>,
) -> Element {
    let light_background = color == "#ffffffff";
    rsx! {
        button {
            width,
            height: 32.0,
            margin_right: 4.0,
            padding: 2.0,
            font_size: 10.0,
            background_color: color,
            font_color: if light_background { "#ff334155" } else { "#ffffffff" },
            onclick: move |_| on_press.call(()),
            "{label}"
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

fn render_virtual_item(
    kind: VirtualKind,
    id: u32,
    index: u32,
    revision: u32,
    mut last_tap: Signal<Option<(u32, u32)>>,
) -> Element {
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
            onclick: move |_| {
                taps += 1;
                last_tap.set(Some((id, taps())));
            },
            text {
                font_size: 14.0,
                font_color: "#ff334155",
                "{label}"
            }
        }
    }
}

fn deterministic_shuffle(items: &mut [DemoItem]) {
    let mut state = 0x5eed_5eed_d15c_a11e_u64;
    for upper in (1..items.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let index = (state as usize) % (upper + 1);
        items.swap(upper, index);
    }
}

fn virtual_kind_label(kind: VirtualKind) -> &'static str {
    match kind {
        VirtualKind::List => "List",
        VirtualKind::Grid => "Grid",
        VirtualKind::WaterFlow => "WaterFlow",
    }
}

#[cfg(test)]
mod tests {
    use super::{deterministic_shuffle, DemoItem};

    #[test]
    fn shuffle_is_deterministic_and_preserves_every_identity() {
        let original = (0..32)
            .map(|id| DemoItem { id, revision: id })
            .collect::<Vec<_>>();
        let mut first = original.clone();
        let mut second = original.clone();

        deterministic_shuffle(&mut first);
        deterministic_shuffle(&mut second);

        assert!(first
            .iter()
            .zip(&second)
            .all(|(left, right)| { left.id == right.id && left.revision == right.revision }));
        assert!(first
            .iter()
            .zip(&original)
            .any(|(left, right)| left.id != right.id));
        let mut ids = first.iter().map(|item| item.id).collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(ids, (0..32).collect::<Vec<_>>());
    }
}
