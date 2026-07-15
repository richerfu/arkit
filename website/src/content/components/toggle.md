---
title: Toggle
---

# Toggle

Toggle 表达一个按钮的 pressed 状态，适合粗体、收藏或工具开关。

```rust
let mut bold = use_signal(|| false);

Toggle {
    label: "粗体",
    icon: "bold",
    variant: ToggleVariant::Outline,
    checked: bold(),
    on_change: move |next| bold.set(next),
}
```

| 属性              | 类型                 | 默认值     | 说明              |
| ----------------- | -------------------- | ---------- | ----------------- |
| `label`           | `String`             | 必填       | 按钮文案          |
| `icon`            | `Option<String>`     | `None`     | 可选图标名        |
| `variant`         | `ToggleVariant`      | 默认样式   | 视觉变体          |
| `checked`         | `Option<bool>`       | `None`     | 受控 pressed 状态 |
| `default_checked` | `bool`               | `false`    | 非受控初始状态    |
| `on_change`       | `EventHandler<bool>` | 默认空处理 | 状态变化回调      |

Toggle 是状态控件，不应拿来代替一次性 Button。纯图标场景仍需提供可理解的 label。
