---
title: Tooltip
---

# Tooltip

Tooltip 为 trigger 提供一句简短说明，使用 hover 打开并通过 OverlayRoot 展示。

```rust
Tooltip {
    trigger: rsx! {
        Button { size: ButtonSize::Icon, icon: "copy" }
    },
    content: "复制链接",
}
```

| 属性                    | 类型                         | 说明               |
| ----------------------- | ---------------------------- | ------------------ |
| `trigger`               | `Element`                    | 被说明的控件       |
| `content`               | `String`                     | 简短文本           |
| `open` / `default_open` | `Option<bool>`               | 受控状态与初始状态 |
| `on_close`              | `Option<EventHandler<()>>`   | 关闭回调           |
| `on_open_change`        | `Option<EventHandler<bool>>` | 状态变化回调       |

Tooltip 不承载交互、长文或表单错误。纯图标按钮仍应有自身可访问名称，Tooltip 只是补充。
