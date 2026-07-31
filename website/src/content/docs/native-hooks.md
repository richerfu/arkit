---
title: 原生节点与布局 Hooks
description: "用精确元素引用观测布局，并在受控边界访问原生节点。"
---

# 原生节点与布局 Hooks

大多数页面只需要 RSX。布局观测、动画 target、XComponent 和 WebView 等集成需要原生节点时，使用绑定到具体元素的 `NativeElementRef`，不要按 component scope 猜测某个“根节点”。

## 精确元素引用

```rust
let reference = use_native_element_ref();

use_layout_frame(reference.clone(), move |frame| {
    if frame.is_measured() {
        // frame 是 window-relative 物理像素
    }
});

rsx! {
    row {
        native_ref: reference,
        width: "100%",
        height: 48.0,
    }
}
```

一个 ref 只描述携带 `native_ref` 属性的那个元素。renderer 挂载节点时生成 `MountedNodeLease`；节点重建或卸载后，旧 lease 会因 generation 不匹配而自动失效。

`NativeElementRef` 不会自动猜测 component 的根节点。ref 没有实际挂到
`native_ref` 时，布局、生命周期和动画 target 都不会收到事件；可复用组件若要支持这些能力，应把可选 ref 显式转发到自己的原生根元素。

## 布局与生命周期

| API                               | 回调参数                   | 用途                       |
| --------------------------------- | -------------------------- | -------------------------- |
| `use_layout_size(ref, callback)`  | `LayoutSize`               | width / height             |
| `use_layout_frame(ref, callback)` | `LayoutFrame`              | window-relative 完整 frame |
| `use_component_lifecycle(ref)`    | `ComponentLifecycleState`  | 精确节点的挂载与可见状态   |
| `use_component_visibility(ref)`   | `bool`                     | 精确节点是否可见           |
| `use_mounted_node(ref, callback)` | `Option<MountedNodeLease>` | 框架级原生集成             |

`LayoutSize` 与 `LayoutFrame` 使用物理像素；`is_measured()` 可过滤零尺寸首帧。ref 的多个消费者复用 renderer 的统一 native event route，不会互相覆盖 ArkUI callback。

`MountedNodeLease` 是非 owning、generation-checked 的借用。普通业务不需要访问它；框架组件只有在 binding API 无法声明式表达时，才通过带安全约束的 `with_native` / `with_native_mut` 调用原生能力。

## NodeBuilder 与 OwnedNativeNode

虚拟列表的 native item 在 Dioxus render cycle 外创建。该边界使用 `arkit::native::NodeBuilder`，并返回唯一 owner `OwnedNativeNode`：

```rust
use arkit::native::{NodeBuilder, OwnedNativeNode};
use ohos_arkui_binding::common::error::ArkUIResult;

fn render_item(index: u32) -> ArkUIResult<OwnedNativeNode> {
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

builder 或未转移的 `OwnedNativeNode` 被 drop 时会 dispose；`.child(...)` 成功后所有权转给父节点；virtual source 接收根 owner 后再接管生命周期。facade 不再导出裸 `create_node*` 和 `NodeKind`，避免出现无 owner 的原生句柄。

## 回到当前 root 的 UI loop

```rust
let runtime = use_runtime_handle();
let mut value = use_signal(String::new);

register_native_callback(move |payload| {
    runtime.queue_ui(move || value.set(payload));
});
```

`RuntimeHandle` 属于当前 root。它同时提供 `queue_ui`、`tokio()` 和 RAII back handler；root 卸载后 pending UI work 会被清理，也不会误唤醒另一个应用 root。

`use_runtime_handle()` 只能在 Arkit 挂载的 Dioxus root 内调用。缺少该上下文属于宿主接入错误，API 会直接失败，而不是回退到某个进程全局 runtime。

## 从旧版 host API 迁移

| 旧 API                                               | 新 API / 做法                                                      |
| ---------------------------------------------------- | ------------------------------------------------------------------ |
| `use_ark_node()` / `ArkNodeRef`                      | `use_native_element_ref()`，把同一 ref 挂到目标元素的 `native_ref` |
| `use_layout_frame(callback)`                         | `use_layout_frame(ref, callback)`                                  |
| `use_ark_host_provider()` / `ArkHost`                | 由 `mount_entry` 自动安装 root-local 上下文                        |
| `OverlayRoot` / `use_overlay()`                      | 在声明位置使用 `Portal` / `ModalPortal`                            |
| `use_virtual_node_adapter*()` / `VirtualNodeAdapter` | `use_virtual_source*()` / `VirtualSource`                          |
| `queue_ui_loop(...)`                                 | `use_runtime_handle().queue_ui(...)`                               |
| `tokio_handle()`                                     | `use_runtime_handle().tokio()`                                     |
| `register_back_press_handler(...)`                   | `use_runtime_handle().register_back_handler(...)`                  |

Portal 是受声明状态控制的：受控组件收到关闭操作时会调用
`on_close` / `on_open_change(false)`，调用方需要同步更新 `open`；框架不再从组件外部命令式删除仍被声明为打开的浮层。

## 所有权规则

- RSX / renderer 创建的 node：renderer 独占，外部只有 `MountedNodeLease`。
- `NodeBuilder::build` 返回且尚未转移的 node：`OwnedNativeNode` 独占。
- `VirtualSource` item：source 接管 wrapper、content 和 item-local runtime。
- 原生 subscription / registration：用 RAII owner 绑定到 component scope。
- ArkUI node、WebView、Drawing handle 不跨线程移动或析构。
