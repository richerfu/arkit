---
title: 安全区与浮层
description: "安全区怎么避让，以及用声明式 Portal 投影浮层。"
---

# 安全区与浮层

刘海、手势条和键盘会吃掉可视区域。业务内容按入口 policy 消费安全区；需要投影到应用根层的内容使用声明式 `Portal`。

## SafeArea

```rust
let safe = use_safe_area();
let policy = use_safe_area_policy();

rsx! {
    SafeArea {
        edges: SafeAreaEdges::BOTTOM | SafeAreaEdges::LEFT,
        PageContent {}
    }
}
```

可组合边：`NONE`、`TOP`、`RIGHT`、`BOTTOM`、`LEFT`、`HORIZONTAL`、`VERTICAL`、`ALL`。嵌套区域只消费声明的边。

## 声明式 Portal

```rust
rsx! {
    if open() {
        Portal {
            layer: OverlayLayer::Floating,
            column {
                position: "24,80",
                padding: 12.0,
                "浮层内容"
            }
        }
    }
}
```

Portal 保留声明位置的 component ownership、Signal、Context 与 Hook，但 renderer 把 native subtree 投影到 root。固定层级顺序是 `Modal < Floating < Transient`。它不是第二个 VirtualDom，也不是 ArkTS overlay window。

全屏 Portal root 默认 pass-through；需要点击的 panel 显式使用默认 hit-test，需要 outside-click 或 modal 行为时声明实际 backdrop。

## ModalPortal

```rust
rsx! {
    ModalPortal {
        open: open(),
        presentation: ModalPresentation::CenteredDialog,
        dismiss_on_backdrop: true,
        on_dismiss: move |_| open.set(false),
        DialogPanel {}
    }
}
```

`ModalPresentation` 支持 `CenteredDialog`、`RightSheet`、`BottomDrawer`。`ModalPortal` 声明 backdrop、safe viewport inset 和 dismissal；open state 始终由业务 Signal / Props 控制。

## 定位

浮层需要把 window-relative trigger frame 换算到 root viewport 时，使用 `use_overlay_viewport()`。返回值包含 content frame、safe-area inset 和 pixel-to-vp scale。

Portal 随声明 scope 一起卸载，不存在独立 token、命令式 publish 或缓存第一次 `Element` 的问题。
