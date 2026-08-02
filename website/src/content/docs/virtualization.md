---
title: 虚拟列表与可见范围
description: "在 List / Grid / WaterFlow 里用 RSX 或 OwnedNativeNode 做真正的虚拟列表。"
---

# 虚拟列表与可见范围

长列表使用 `VirtualSource`：ArkUI 只请求可见项，renderer 根据声明式 `virtual_source` 属性自动 attach、rebind 和 detach，不需要业务拿宿主节点手工连接 adapter。

## 支持的容器

| `VirtualKind` | Host tag    | 自动 wrapper |
| ------------- | ----------- | ------------ |
| `List`        | `list`      | ListItem     |
| `Grid`        | `grid`      | GridItem     |
| `WaterFlow`   | `waterflow` | FlowItem     |

## RSX item

```rust
const TOTAL: u32 = 10_000;

#[component]
fn VirtualList() -> Element {
    let source = use_virtual_source(VirtualKind::List, TOTAL, move |index| {
        let mut selected = use_signal(|| false);
        rsx! {
            row {
                width: "100%",
                height: 44.0,
                padding: 12.0,
                background_color: if selected() { "#ffdbeafe" } else { "#ffffffff" },
                onclick: move |_| selected.toggle(),
                text { "#{index:05}" }
            }
        }
    });

    rsx! {
        list {
            virtual_source: source,
            width: "100%",
            height: "100%",
        }
    }
}
```

每个可见 item 有独立的 Dioxus subtree。item 被 ArkUI 回收时，其 effect、event listener、task、hook 状态和 native wrapper 会按顺序清理。窗口指标、应用生命周期、安全区策略和当前 `RuntimeHandle` 会自动提供给 item runtime；业务自定义 context 仍需显式传入。

## Grid 与 WaterFlow

source 管 item 生命周期；列模板、间距和缓存数量仍写在容器上：

```rust
let source = use_virtual_source(VirtualKind::WaterFlow, TOTAL, render_item);

rsx! {
    waterflow {
        virtual_source: source,
        width: "100%",
        height: "100%",
        padding: 12.0,
        water_flow_column_template: "repeat(auto-fill, 104vp)",
        water_flow_column_gap: 12.0,
        water_flow_row_gap: 12.0,
        water_flow_cached_count: 6_i32,
    }
}
```

## 精确更新

`item_keys[index]` 应覆盖 item 的全部非响应式视觉输入。长度相同时，keyed hook 只 reload 变化的连续区间：

```rust
let rows = use_signal(load_rows);
let keys = rows().iter().map(|row| row.revision).collect();
let source = use_virtual_source_items_keyed(
    VirtualKind::List,
    keys,
    move |index| {
        let row = rows()[index as usize].clone();
        rsx! { RowView { row } }
    },
);

rsx! { list { virtual_source: source } }
```

`VirtualSource` 也提供 `reload_items`、`reload_all_items`、`insert_items`、`remove_items`、`move_item` 和 `set_total_count`。结构更新前先更新业务数据，因为 ArkUI 可能在调用中同步请求新位置的 item。

## Native item

不需要 item-local Dioxus 状态时，同一个 hook 可返回 `ArkUIResult<OwnedNativeNode>`：

```rust
use arkit::native::NodeBuilder;

let source = use_virtual_source(VirtualKind::List, TOTAL, move |index| {
    Ok(NodeBuilder::new("text")?
        .text_content(format!("#{index:05}"))?
        .build())
});

rsx! { list { virtual_source: source } }
```

callback 在 Dioxus render cycle 外执行，不能调用 hook。source 自动创建 ListItem / GridItem / FlowItem wrapper，并在 attachment 失败、reload 或回收时按唯一 owner 规则释放节点。

## 选择普通 keyed 列表还是虚拟 source

小列表直接 keyed RSX 最简单。item 数量上千、首屏 native 创建成本明显，或需要 WaterFlow 回收时使用 `use_virtual_source`。不要捕获临时 borrow；callback 应捕获 `Rc`、signal handle 或 owned snapshot。

## 可见范围

`use_virtual_range()` 返回 `Signal<VirtualVisibleRange>` 的 read/write handle，适合预取、曝光和分页：

```rust
let (range, mut set_range) = use_virtual_range();
set_range.set(VirtualVisibleRange::new(first, last));
```

index 是 inclusive。默认 `first_index = 0`、`last_index = -1` 表示尚无可见项。`VirtualSource` 本身不依赖这个 signal。
