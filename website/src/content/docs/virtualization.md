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

## 稳定身份与精确更新

高层 `use_virtual_items` 明确分开两件事：`id` 只表示稳定身份，`revision` 覆盖 item 的全部非响应式视觉输入。同一 ID 改 revision 会原位 reload；移动同一 ID 会保留仍被 adapter 持有的 item-local 状态。一个 snapshot 内的 ID 必须唯一，重复会被明确拒绝：

```rust
let rows = use_signal(load_rows);
let snapshot = rows();
let stamps = snapshot
    .iter()
    .map(|row| VirtualItemStamp::new(row.id, row.revision))
    .collect();
let render_rows = snapshot.clone();
let source = use_virtual_items(
    VirtualKind::List,
    stamps,
    move |index| {
        let row = render_rows[index as usize].clone();
        rsx! { RowView { row } }
    },
);

rsx! { list { virtual_source: source } }
```

返回的 `VirtualItems` 只能绑定到 `virtual_source` 属性，不暴露手动数据 mutation，避免 snapshot 和 adapter 各自修改数量/顺序。

`use_virtual_items` 也接受 full reverse、rotate 和 shuffle 这类全量顺序变化；身份仍由 ID 而不是 index 决定。不过 item-local hook 状态只保证在该 item 仍被 adapter 保留时随移动保存。把 item 移到远离 viewport 的位置后，ArkUI 可以回收其 subtree；再次滚回时重新得到初始 local state 属于正常的有界虚拟化行为，不应当被判定为 identity diff 错误。`complex_cases` 示例因此用保持目标仍可见的单步 rotate 检查 taps，用 reverse/shuffle 检查 native 顺序、数量和响应，再用 Restore 回到基线。

性能记录也需要区分边界：业务 signal 更新只代表逻辑操作已触发，`use_virtual_items` effect 完成只代表 adapter mutation 已提交，真正 native layout 要到后续 ArkUI frame 才发生。当前高层 hook 没有暴露 native layout-complete 回调；没有该信号时不要把 render/effect 的时间标成“布局耗时”。

需要手工控制时使用低层 `use_virtual_source` / `VirtualSource`。传入的 count 只是初始值；之后由调用方在更新业务数据后显式调用 `reload_items`、`reload_all_items`、`insert_items`、`remove_items`、`move_item` 或 `set_total_count`。ArkUI 可能在 mutation 中同步请求新位置的 item。

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

小列表直接 keyed RSX 最简单。item 数量上千、首屏 native 创建成本明显，或需要 WaterFlow 回收时使用 `use_virtual_items`；需要直接操作 adapter 时才用 `use_virtual_source`。不要捕获临时 borrow；callback 应捕获 `Rc`、signal handle 或 owned snapshot。

## 可见范围

`use_virtual_range()` 返回 `Signal<VirtualVisibleRange>` 的 read/write handle，适合预取、曝光和分页：

```rust
let (range, mut set_range) = use_virtual_range();
set_range.set(VirtualVisibleRange::new(first, last));
```

index 是 inclusive。默认 `first_index = 0`、`last_index = -1` 表示尚无可见项。`VirtualSource` 本身不依赖这个 signal。
