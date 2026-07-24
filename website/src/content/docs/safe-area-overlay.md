---
title: 安全区与浮层
description: "安全区怎么避让，浮层又该挂在哪一层。"
---

# 安全区与浮层

刘海、手势条和键盘会吃掉可视区域。业务内容默认待在安全区里；需要贴边或盖在上面的浮层，走 OverlayRoot。

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

edge-to-edge 入口让业务内容填满 XComponent，但 modal panel 和框架 floating content 仍可使用 safe viewport。Backdrop 可以覆盖系统栏附近。

## 发布浮层

```rust
let overlay = use_overlay();
let trigger = overlay.clone();

rsx! {
    button {
        onclick: move |_| trigger.show_floating(|| rsx! {
            column { padding: 12.0, "浮层内容" }
        }),
        "打开"
    }
}
```

浮层内容仍属于同一个 VirtualDom，Signal、Context 和 Hook 正常工作。它不是第二个 renderer，也不是 ArkTS overlay window。

## Modal 类型

`ModalPresentation` 支持 `CenteredDialog`、`RightSheet`、`BottomDrawer`。`ModalOverlaySpec` 控制 backdrop、safe viewport inset 和 `dismiss_on_backdrop`。

| Overlay API                  | 作用                           |
| ---------------------------- | ------------------------------ |
| `show_floating`              | 发布自定位的全屏浮层 subtree   |
| `show_modal`                 | 发布标准 modal chrome          |
| `show_modal_with_dismiss`    | 关闭前同步受控状态             |
| `dismiss`                    | 移除当前 token                 |
| `is_open`                    | 查询 token 是否打开            |
| `overlay_frame` / `viewport` | 读取浮层 frame 与安全 viewport |

## 生命周期与状态

`use_overlay` 为当前 scope 创建 token。scope 卸载会移除自己的 overlay，并断开 content closure；旧 handle 不能重新发布。

Dialog、Popover、Menu 的 open state 应由 Signal/Props 控制。保持打开时要重新发布最新 subtree，不能永久缓存第一次生成的 `Element`，否则 checkbox、submenu 等会显示陈旧状态。
