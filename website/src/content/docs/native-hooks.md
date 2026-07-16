---
title: 原生节点与布局 Hooks
description: "节点句柄、布局观测与 UI-loop handoff。"
---

# 原生节点与布局 Hooks

核心业务 UI 应使用 `rsx!`。本章 API 是 escape hatch，只在声明式 element registry 无法表达的能力中使用，例如原生 WebView、NodeAdapter、自定义 Drawing 或第三方 ArkUI node。

## ArkHost 与 use_ark_node

`#[entry]` 会自动安装 `ArkHost`。组件调用 `use_ark_node()` 后，runtime 在完成一次 Dioxus render 后把该 scope 对应的 mounted root node 写回 `ArkNodeRef`。

```rust
let node_ref = use_ark_node();

use_effect(move || {
    if let Some(node) = node_ref.peek() {
        // node 是 renderer 当前持有的同一个 Rc<RefCell<ArkUINode>>
        let native = node.borrow();
        // 只做必要的原生操作
    }
});
```

节点可能在首帧尚未 resolve，也可能随 subtree 替换而变化。不要把裸 native handle 存到 process-global 状态。

`use_ark_host_provider()` 只用于手工构造 `VirtualDom` 并直接挂载 runtime 的高级场景；普通 `#[entry]` 应用不得重复调用。

## 布局观测

布局 hook 有三个层次：

| API                     | 回调参数                  | 典型用途                              |
| ----------------------- | ------------------------- | ------------------------------------- |
| `use_layout_size`       | `LayoutSize`              | 只关心 width/height                   |
| `use_layout_frame`      | `LayoutFrame`             | 需要 window-relative x/y/width/height |
| `use_layout_frame_node` | `ArkUINode + LayoutFrame` | 需要把原生 child/adapter 挂到宿主     |

`LayoutSize` 与 `LayoutFrame` 使用物理像素；`is_measured()` 可过滤零尺寸首帧。

```rust
use_layout_frame_node(move |mut node, frame| {
    if !frame.is_measured() {
        return;
    }
    // 同步原生 child 的尺寸或挂载 adapter
});
```

同一 node 的多个订阅共享一个 native area-change listener。hook 卸载会按 token 移除对应订阅；native handle 被复用时 generation 会隔离旧 callback。

## NodeBuilder

NodeAdapter 的 `render_item` 在 Dioxus render cycle 外执行，必须返回原生 `ArkUINode`。`NodeBuilder` 为这个边界提供链式、可清理的构造器：

```rust
fn render_item(index: u32) -> ArkUIResult<ArkUINode> {
    let label = NodeBuilder::new("text")?
        .font_size(14.0)?
        .font_color("#ff334155")?
        .text_content(format!("Item {index}"))?
        .build();

    Ok(NodeBuilder::new("row")?
        .percent_width(1.0)?
        .height(48.0)?
        .padding([12.0, 12.0, 12.0, 12.0])?
        .child(label)?
        .build())
}
```

提供的 convenience methods：

- `percent_width`、`percent_height`、`width`、`height`
- `background_color`、`font_size`、`font_color`、`text_content`
- `padding`、`margin`、`child`
- `attr`：传入 canonical `ArkUINodeAttributeType`

builder 在 `build` 前持有 native cleanup guard。任一步返回错误或 builder 被提前 drop，已创建 node 会被 dispose；`build` 后所有权转移给调用方。

## Tag 与原生创建

facade 还导出：

| API                  | 说明                                  |
| -------------------- | ------------------------------------- |
| `canonical_tag`      | 把 alias 规范化为 canonical ArkUI tag |
| `kind_from_tag`      | tag 到 `NodeKind`                     |
| `create_node`        | 按 `NodeKind` 创建                    |
| `create_node_by_tag` | 按 RSX/ArkUI tag 创建                 |

普通业务不要绕过 `NodeBuilder` 直接拼 early-error path；binding 的 `ArkUINode` 没有隐式 `Drop`，遗漏 dispose 会泄漏原生对象。

## UI-loop handoff

原生 callback 不能在 native patch 期间同步写 Dioxus：

```rust
let mut value = use_signal(String::new);

register_native_callback(move |payload| {
    queue_ui_loop(move || {
        value.set(payload);
    });
});
```

`queue_ui_loop` 把 owned closure 绑定到当前 root，唤醒 OpenHarmony loop；root 卸载后其 pending effects 会被清理。

## ScopeNodeResolver

`ScopeNodeResolver`、`register_scope_resolver` 和 `ScopeResolverRegistration` 是 runtime 与 hooks crate 之间的公开窄接口。它们负责将 `ScopeId` 解析为 renderer 已挂载节点，通常只由 `ArkHost` 实现。应用层只消费 `use_ark_node`，不应自行注册第二个 resolver。

## 所有权规则

- Dioxus/renderer 创建的 node：借用，不 dispose。
- `NodeBuilder::build` 返回且尚未交给 adapter/tree 的 node：调用方拥有。
- NodeAdapter item：adapter 接管 wrapper/item 生命周期。
- 原生 subscription/registration：持有 RAII registration 到对应 component scope。
- 不跨线程移动 ArkUI node、WebView 或 Drawing handle。
