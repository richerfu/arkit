---
title: 虚拟列表与可见范围
description: "List、Grid、WaterFlow 的 RSX 真虚拟化与 NodeAdapter。"
---

# 虚拟列表与可见范围

`use_virtual_node_adapter_rsx` 把 ArkUI NodeAdapter 挂到 `list`、`grid` 或 `waterflow`。只有进入可见区域的 item 才创建对应的 Dioxus subtree；item 可以直接写 RSX、组件、hook、signal 和事件。这是真虚拟化，不是先构建完整列表再隐藏。

原有 `use_virtual_node_adapter` 继续保留，作为需要 `NodeBuilder` 的底层原生节点路径。

## 支持的容器

| `VirtualKind` | Host tag    | 自动 wrapper |
| ------------- | ----------- | ------------ |
| `List`        | `list`      | ListItem     |
| `Grid`        | `grid`      | GridItem     |
| `WaterFlow`   | `waterflow` | FlowItem     |

三种容器使用同一套 hook 和更新策略，容器差异只由 `VirtualKind` 表达。

## 基本用法

```rust
const TOTAL: u32 = 10_000;

#[component]
fn VirtualList() -> Element {
    let adapter = use_virtual_node_adapter_rsx(
        VirtualKind::List,
        TOTAL,
        move |index| {
            let mut selected = use_signal(|| false);
            rsx! {
                row {
                    width: "100%",
                    height: 44.0,
                    padding: 12.0,
                    background_color: if selected() { "#ffdbeafe" } else { "#ffffffff" },
                    onclick: move |_| selected.toggle(),
                    text {
                        font_size: 14.0,
                        "#{index:05}"
                    }
                }
            }
        },
    );

    let attach_adapter = adapter.clone();
    use_layout_frame_node(move |host, _frame| {
        let _ = attach_adapter.attach(&host);
    });

    rsx! {
        list {
            width: "100%",
            height: "100%",
        }
    }
}
```

`attach` 是幂等的。通过 layout hook attach 时，host node 与可用尺寸已经建立，也能处理 Dioxus 替换 native wrapper 的情况。

每个可见 item 都有独立的 component scope。上例中的 `selected` 只更新被点击的 item；item 离开 ArkUI 缓存并被回收时，对应的 effect、event listener、task、hook 状态和 native wrapper 会一起清理。

## Grid 与 WaterFlow

adapter 负责 item 生命周期；列模板、间距和缓存数量仍在声明式 host 上配置：

```rust
grid {
    width: "100%",
    height: "100%",
    grid_column_template: "1fr 1fr",
}

waterflow {
    width: "100%",
    height: "100%",
    padding: 12.0,
    water_flow_column_template: "repeat(auto-fill, 104vp)",
    water_flow_column_gap: 12.0,
    water_flow_row_gap: 12.0,
    water_flow_cached_count: 6_i32,
}
```

WaterFlow item 可以返回不同高度。列宽由 host template 决定；如果百分比宽度会按 host 而非 FlowItem 计算，可像 `examples/complex_cases` 一样给 item root 显式设置 track width。

## 更新数据

item 内读取的 signal 会按正常 Dioxus 规则局部重渲染。不经过响应式状态的外部数据变化，可使用 keyed hook：

```rust
let rows = use_signal(load_rows);
let keys = rows().iter().map(|row| row.revision).collect();
let adapter = use_virtual_node_adapter_rsx_items_keyed(
    VirtualKind::List,
    keys,
    move |index| {
        let row = &rows()[index as usize];
        rsx! { RowView { row: row.clone() } }
    },
);
```

`item_keys[index]` 应覆盖该 item 的全部非响应式视觉输入。长度不变时，hook 只 reload 发生变化的连续区间。

`VirtualNodeAdapter` 本身也提供命令式结构更新：

| 方法                         | 语义                                               |
| ---------------------------- | -------------------------------------------------- |
| `reload_items(start, count)` | 局部重建指定范围，保留其他可见节点                 |
| `reload_all_items()`         | 数据等长变化时重建全部可见节点                     |
| `insert_items(start, count)` | 插入数据并保留未受影响的节点和滚动位置             |
| `remove_items(start, count)` | 删除数据并保留未受影响的节点和滚动位置             |
| `move_item(from, to)`        | 移动单项，尽量复用现有原生节点                     |
| `set_total_count(total)`     | 总数变化但无法描述具体结构时同步数量并重建可见节点 |

`insert_items`、`remove_items` 和 `move_item` 调用前必须先更新业务数据，因为 ArkUI 可能在方法内部同步请求新位置的 item。被 reload 的可见 item 会卸载旧 subtree 并创建新 scope，因此 item-local hook 状态也会重置。

`render_item` 捕获的数据必须在 adapter 生命周期内有效。不要捕获临时 borrow；优先捕获 `Rc`、signal handle 或不可变数据 owner。框架会自动把窗口指标、应用生命周期、安全区策略和 item-local `ArkHost` 提供给 RSX item。业务自定义 context 不会跨独立的虚拟 item scope 自动继承；需要时先在父组件取得 cloneable context handle，再在 `render_item` 中用 `use_context_provider` 提供。

## RSX render_item 约束

- callback 本身就是一个 item component scope，可以使用 Dioxus hooks。
- 不假定 callback 按 index 顺序执行，ArkUI 可按 viewport 需求请求、回收或重建。
- item 构建应短小确定；图片解码、网络请求等工作仍应使用 resource 或外部预取。
- `reload_items` 表示重新挂载，不用于本可由 item-local signal 完成的普通状态更新。

## NodeBuilder 底层路径

只有直接集成原生 node 或追求最小 Dioxus 开销时，才使用原有 hook：

```rust
let adapter = use_virtual_node_adapter(
    VirtualKind::List,
    TOTAL,
    move |index| {
        Ok(NodeBuilder::new("text")?
            .text_content(format!("#{index:05}"))?
            .build())
    },
);
```

这个 callback 在 Dioxus render cycle 外执行，必须返回全新的 `ArkUINode`，不能调用 hook。adapter 仍会自动创建 ListItem/GridItem/FlowItem wrapper，并负责原生节点清理。

## 选择普通 keyed 列表还是虚拟 RSX

小列表直接在 `rsx!` 中 keyed render，结构和状态最简单。item 数量上千、首屏 native 创建成本明显或需要 WaterFlow 回收时，使用 `use_virtual_node_adapter_rsx`。它只为 ArkUI 当前保留的 item 建立独立 Dioxus scope，保留声明式能力的同时限制节点与 hook 数量。

## 可见范围

`use_virtual_range()` 返回同一个 `Signal<VirtualVisibleRange>` 的 read/write handle：

```rust
let (range, mut set_range) = use_virtual_range();
set_range.set(VirtualVisibleRange::new(first, last));
```

index 是 inclusive。默认 `first_index = 0`、`last_index = -1` 表示尚无可见项，`is_empty()` 返回 true。NodeAdapter 本身不依赖这个 signal；它适用于预取、埋点、可见曝光或与外部分页数据同步。

## 验证

`examples/complex_cases` 同时覆盖 10,000 条 List、两列 Grid、变高 WaterFlow、item-local signal 和三种容器的单项动态更新：

```sh
cd examples/complex_cases
ohrs build --arch aarch
```

真机检查：

- 点击任一可见 item，其 `taps` signal 独立更新，不影响相邻 item。
- 初始目标 `#00002` 可见时，连续点击“更新单项”，只有该 item 的 revision、颜色和本地状态变化。
- WaterFlow 每次更新目标 item 时高度随 revision 改变，其他 item 不应重建或丢失滚动锚点。
- 快速滚动、切换容器、item 回收以及返回页面后的 native 资源释放保持正常。
