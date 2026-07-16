---
title: Popover
description: "锚点交互浮层。"
---

# Popover

Popover 把可交互的短内容锚定到 trigger，位置根据 trigger frame 与安全视口计算。

```rust
Popover {
    trigger: rsx! { Button { "打开设置" } },
    width: 280.0,
    on_open_change: move |open| tracing::debug!(open),
    QuickSettings {}
}
```

| 属性                    | 类型                         | 说明               |
| ----------------------- | ---------------------------- | ------------------ |
| `trigger`               | `Element`                    | 锚点与点击入口     |
| `open` / `default_open` | `Option<bool>`               | 受控状态与初始状态 |
| `on_close`              | `Option<EventHandler<()>>`   | 关闭通知           |
| `on_open_change`        | `Option<EventHandler<bool>>` | 完整打开状态通知   |
| `width`                 | `Option<f32>`                | 面板宽度           |
| `children`              | `Element`                    | 面板内容           |

短设置与少量操作适合 Popover；长表单使用 Dialog/Sheet。视口旋转或 resize 后由组件重新定位，业务不要保存绝对坐标。
