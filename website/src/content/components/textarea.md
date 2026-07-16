---
title: Textarea
description: "多行受控文本输入。"
---

# Textarea

Textarea 用于多行文本，状态模型与 Input 相同。请给它确定高度或由父布局约束，避免内容增长持续撑开页面。

```rust
let mut description = use_signal(String::new);

Textarea {
    value: description(),
    placeholder: "补充说明",
    height: 120.0,
    percent_width: 1.0,
    on_change: move |next| description.set(next),
}
```

## Props

| 属性            | 类型                           | 默认值  | 说明           |
| --------------- | ------------------------------ | ------- | -------------- |
| `value`         | `Option<String>`               | `None`  | 当前多行文本   |
| `placeholder`   | `Option<String>`               | `None`  | 空值提示       |
| `height`        | `Option<f32>`                  | 64vp    | 输入区域高度   |
| `percent_width` | `Option<f32>`                  | `None`  | 相对父容器宽度 |
| `invalid`       | `bool`                         | `false` | 错误态         |
| `disabled`      | `bool`                         | `false` | 禁用编辑       |
| `on_change`     | `Option<EventHandler<String>>` | `None`  | 文本变化回调   |

长内容要明确最大长度，并在组件外展示字符数。需要富文本、Markdown 编辑或自动增长时，应在基础输入能力上封装专用编辑器。
