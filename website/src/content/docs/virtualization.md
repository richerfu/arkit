---
title: 虚拟列表与可见范围
description: "List、Grid、WaterFlow 的 NodeAdapter。"
---

# 虚拟列表与可见范围

`use_virtual_node_adapter` 把 ArkUI NodeAdapter 挂到 `list`、`grid` 或 `waterflow`。只有进入可见区域的 item 才调用 `render_item` 创建原生节点；这是真虚拟化，不是先构建完整 Dioxus subtree 再隐藏。

## 支持的容器

| `VirtualKind` | Host tag    | 自动 wrapper |
| ------------- | ----------- | ------------ |
| `List`        | `list`      | ListItem     |
| `Grid`        | `grid`      | GridItem     |
| `WaterFlow`   | `waterflow` | FlowItem     |

`use_virtual_water_flow` 是 WaterFlow 的便捷入口。旧名称 `use_virtual_list` 仍接受三种 kind，但新代码使用 `use_virtual_node_adapter`。

## 基本用法

```rust
const TOTAL: u32 = 10_000;

#[component]
fn VirtualList() -> Element {
    let handle = use_virtual_node_adapter(
        VirtualKind::List,
        TOTAL,
        move |index| {
            let text = NodeBuilder::new("text")?
                .text_content(format!("#{index:05}"))?
                .font_size(14.0)?
                .build();
            Ok(NodeBuilder::new("row")?
                .percent_width(1.0)?
                .height(44.0)?
                .padding([12.0; 4])?
                .child(text)?
                .build())
        },
    );

    use_layout_frame_node(move |host, _frame| {
        let _ = handle.attach(&host);
    });

    rsx! {
        list {
            percent_width: 1.0,
            percent_height: 1.0,
        }
    }
}
```

`attach` 是幂等的。通过 layout hook attach 的好处是 host node 与可用尺寸已经建立，也能处理 Dioxus 替换 native wrapper 的情况。

## Grid

adapter 只负责 item 生命周期；列模板和间距仍在声明式 host 上配置：

```rust
grid {
    percent_width: 1.0,
    percent_height: 1.0,
    grid_column_template: "1fr 1fr",
}
```

`render_item` 返回 item content，adapter 自动创建 GridItem wrapper。

## WaterFlow

```rust
let handle = use_virtual_water_flow(TOTAL, render_waterfall_item);

rsx! {
    waterflow {
        percent_width: 1.0,
        percent_height: 1.0,
        padding: 12.0,
        water_flow_column_template: "repeat(auto-fill, 104vp)",
        water_flow_column_gap: 12.0,
        water_flow_row_gap: 12.0,
        water_flow_cached_count: 6_i32,
    }
}
```

WaterFlow item 可以返回不同高度。列宽由 host template 决定；如果百分比宽度会按 host 而非 FlowItem 计算，可像 `examples/complex_cases` 一样给 content 显式设置 track width。

## 更新 item 数量

`VirtualNodeAdapterHandle::set_total_count(total)` 更新总数并触发 reload。handle 是 cloneable 的轻量共享句柄，业务数据仍应由稳定 owner 持有。

`render_item` 捕获的数据必须在 adapter 生命周期内有效。不要捕获临时 borrow；优先捕获 `Rc`/signal snapshot 或不可变数据 owner。

## 可见范围

`use_virtual_range()` 返回同一个 `Signal<VirtualVisibleRange>` 的 read/write handle：

```rust
let (range, mut set_range) = use_virtual_range();

// 在容器的 scroll-index 回调中：
set_range.set(VirtualVisibleRange::new(first, last));
```

index 是 inclusive。默认 `first_index = 0`、`last_index = -1` 表示尚无可见项，`is_empty()` 返回 true。

NodeAdapter 本身不依赖这个 signal；它适用于预取、埋点、可见曝光或与外部分页数据同步。

## render_item 约束

- 每次调用返回一个全新的 content node。
- 不在 callback 中调用 Dioxus hooks。
- 不持有 renderer tree 中 node 的可变 borrow。
- 所有 early-error path 由 `NodeBuilder` 或显式 cleanup 处理。
- item 构建应短小确定；图片解码、网络请求等工作在外部预取。
- 不假定 callback 按 index 顺序执行，ArkUI 可按 viewport 需求请求/回收。

## 选择声明式列表还是 NodeAdapter

小列表、item 需要完整 Dioxus component/hooks 时，直接在 `rsx!` 中 keyed render 最简单。大列表、item 数量上千或 native 创建成本明显时使用 NodeAdapter。NodeAdapter item 是原生节点，不具有独立 Dioxus component scope；复杂响应式 item 需要在数据更新时 reload 或设计专门的 adapter update path。

## 验证

`examples/complex_cases` 同时覆盖 10,000 条 List、两列 Grid 与变高 WaterFlow：

```sh
cd examples/complex_cases
ohrs build --arch aarch
```

真机检查快速滚动、切换容器、item 回收、变高布局和返回页面后的 native 资源释放。
