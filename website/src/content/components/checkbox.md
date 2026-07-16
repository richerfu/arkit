---
title: Checkbox
description: "受控和非受控复选。"
---

# Checkbox

Checkbox 表达一个可独立选择的布尔值，适合协议确认或多选列表。`checked` 与 `default_checked` 分别对应受控和非受控模式。

```rust
let mut accepted = use_signal(|| false);

Checkbox {
    label: "我已阅读并同意协议",
    checked: accepted(),
    on_change: move |next| accepted.set(next),
}
```

## Props

| 属性              | 类型                         | 默认值 | 说明               |
| ----------------- | ---------------------------- | ------ | ------------------ |
| `label`           | `Option<String>`             | `None` | 复选框旁的可读标签 |
| `checked`         | `Option<bool>`               | `None` | 受控选中状态       |
| `default_checked` | `Option<bool>`               | `None` | 非受控初始状态     |
| `checked_color`   | `Option<u32>`                | 主题色 | 选中背景颜色       |
| `disabled`        | `Option<bool>`               | `None` | 禁用交互           |
| `on_change`       | `Option<EventHandler<bool>>` | `None` | 下一状态回调       |

传入 `checked` 后，回调不会替调用方修改外部状态。协议类复选框应让标签与控件形成同一点击区域。
