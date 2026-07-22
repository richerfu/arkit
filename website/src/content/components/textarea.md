---
title: Textarea
description: "多行文本输入。"
---

# Textarea

shadcn 风格的多行输入框。

## 用法

```rust
Textarea {
    placeholder: Some("备注".into()),
    value: Some(notes()),
    width: "100%",
    on_change: move |v| notes.set(v),
}
```

## Props

| Prop          | 类型                           | 默认    | 说明                            |
| ------------- | ------------------------------ | ------- | ------------------------------- |
| `placeholder` | `Option<String>`               | `None`  | 占位文案                        |
| `value`       | `Option<String>`               | `None`  | 受控值                          |
| `height`      | `Option<f32>`                  | `64`    | 固定高度 vp                     |
| `width`       | `Option<String>`               | `None`  | CSS 宽度（`"100%"` 表示占满）   |
| `invalid`     | `bool`                         | `false` | 校验失败时使用 destructive 边框 |
| `disabled`    | `bool`                         | `false` | 禁止编辑，保留尺寸              |
| `on_change`   | `Option<EventHandler<String>>` | `None`  | 文本变更                        |
