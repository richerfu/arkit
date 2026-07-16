---
title: Switch
description: "即时二元设置。"
---

# Switch

Switch 用于立即生效的开关设置。需要“确认后才提交”的布尔选择，应使用 Checkbox，而不是 Switch。

```rust
let mut enabled = use_signal(|| true);

Switch {
    checked: enabled(),
    on_change: move |next| enabled.set(next),
}
```

| 属性              | 类型                         | 默认值 | 说明           |
| ----------------- | ---------------------------- | ------ | -------------- |
| `checked`         | `Option<bool>`               | `None` | 受控状态       |
| `default_checked` | `Option<bool>`               | `None` | 非受控初始状态 |
| `on_change`       | `Option<EventHandler<bool>>` | `None` | 状态变化回调   |

Switch 本身不包含标签，页面应在旁边提供明确设置名称。若切换会触发异步副作用，应处理失败回滚或显示保存状态。
