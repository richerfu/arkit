---
title: Input
description: "单行受控文本输入。"
---

# Input

Input 是单行受控文本输入。组件负责主题样式、禁用态和错误态，字段值与校验状态由页面持有。

```rust
let mut name = use_signal(String::new);

Input {
    value: name(),
    placeholder: "用户名",
    percent_width: 1.0,
    invalid: name().chars().count() > 20,
    on_change: move |next| name.set(next),
}
```

## Props

| 属性            | 类型                           | 默认值  | 说明                           |
| --------------- | ------------------------------ | ------- | ------------------------------ |
| `value`         | `Option<String>`               | `None`  | 当前值；传入后按受控方式使用   |
| `placeholder`   | `Option<String>`               | `None`  | 空值提示                       |
| `height`        | `Option<f32>`                  | 48vp    | 输入框高度                     |
| `percent_width` | `Option<f32>`                  | `None`  | 相对父容器宽度，`1.0` 表示占满 |
| `invalid`       | `bool`                         | `false` | 使用 destructive 边框          |
| `disabled`      | `bool`                         | `false` | 禁止编辑并显示禁用态           |
| `on_change`     | `Option<EventHandler<String>>` | `None`  | 文本变化回调                   |

错误不能只依赖边框颜色表达，应配合 `FieldError`。提交期间可禁用输入，但失败后应保留用户已经输入的值。
