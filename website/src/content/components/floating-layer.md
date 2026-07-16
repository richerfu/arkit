---
title: Floating Layer
description: "底层浮层定位 primitive。"
---

# Floating Layer

FloatingLayer 是 Popover、Tooltip 和 HoverCard 之下的定位 primitive。仅在现有浮层组件无法满足时直接使用。

```rust
FloatingLayer {
    trigger: rsx! { Button { "更多" } },
    side: FloatingSide::Bottom,
    hover: false,
    on_close: move |_| tracing::debug!("closed"),
    CustomPanel {}
}
```

| 属性                    | 类型                       | 说明                              |
| ----------------------- | -------------------------- | --------------------------------- |
| `trigger`               | `Element`                  | 锚点内容                          |
| `open` / `default_open` | `Option<bool>`             | 打开状态                          |
| `on_close`              | `Option<EventHandler<()>>` | 关闭通知                          |
| `side`                  | `Option<FloatingSide>`     | `Top`、`Bottom`、`Left`、`Right`  |
| `hover`                 | `Option<bool>`             | true 使用 hover，false 使用 click |
| `children`              | `Element`                  | 浮层内容                          |

`FloatingAlign` 是公共定位类型，但当前 FloatingLayer API 不暴露 align 参数。组件使用捕获层处理外部点击；scope 卸载时自动清理 overlay。
