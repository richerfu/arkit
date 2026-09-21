---
title: Input
description: "单行文本输入，适合标题、账号这类短内容。"
---

# Input

单行文本输入。

## 用法

```rust
Input {
    mode: InputMode::Text,
    placeholder: Some("邮箱".into()),
    value: Some(email()),
    on_change: move |v| email.set(v),
}
```

密码输入使用系统密码掩码，并在输入框末尾显示跟随当前主题颜色的显隐图标：

```rust
Input {
    mode: InputMode::Password,
    placeholder: Some("密码".into()),
    value: Some(password()),
    on_change: move |v| password.set(v),
}
```

纯数字输入会启用数字键盘，并同时过滤键入和粘贴内容中的非 ASCII 数字：

```rust
Input {
    mode: InputMode::Number,
    placeholder: Some("验证码".into()),
    value: Some(code()),
    on_change: move |v| code.set(v),
}
```

## Props

| Prop          | 类型                           | 默认    | 说明                            |
| ------------- | ------------------------------ | ------- | ------------------------------- |
| `placeholder` | `Option<String>`               | `None`  | 占位文案                        |
| `value`       | `Option<String>`               | `None`  | 受控值                          |
| `mode`        | `InputMode`                    | `Text`  | `Text`、`Password` 或 `Number`  |
| `height`      | `Option<f32>`                  | `36`    | 固定高度 vp                     |
| `width`       | `Option<String>`               | `None`  | 显式 CSS 宽度，优先于 `full`      |
| `full`        | `bool`                         | `true`  | 默认占满父容器可用宽度            |
| `invalid`     | `bool`                         | `false` | 校验失败时使用 destructive 边框 |
| `disabled`    | `bool`                         | `false` | 禁止编辑，保留尺寸              |
| `read_only`   | `bool`                         | `false` | 保持外观但禁止焦点和输入        |
| `on_change`   | `Option<EventHandler<String>>` | `None`  | 文本变更                        |
| `on_click`    | `Option<EventHandler<()>>`     | `None`  | 点击输入框                      |
