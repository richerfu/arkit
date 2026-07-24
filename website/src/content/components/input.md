---
title: Input
description: "单行文本输入，适合标题、账号这类短内容。"
---

# Input

单行文本输入。

## 用法

```rust
Input {
    placeholder: Some("邮箱".into()),
    value: Some(email()),
    width: "100%",
    on_change: move |v| email.set(v),
}
```

## Props

| Prop          | 类型                           | 默认    | 说明                            |
| ------------- | ------------------------------ | ------- | ------------------------------- |
| `placeholder` | `Option<String>`               | `None`  | 占位文案                        |
| `value`       | `Option<String>`               | `None`  | 受控值                          |
| `height`      | `Option<f32>`                  | `48`    | 固定高度 vp                     |
| `width`       | `Option<String>`               | `None`  | CSS 宽度（`"100%"` 表示占满）   |
| `invalid`     | `bool`                         | `false` | 校验失败时使用 destructive 边框 |
| `disabled`    | `bool`                         | `false` | 禁止编辑，保留尺寸              |
| `on_change`   | `Option<EventHandler<String>>` | `None`  | 文本变更                        |
