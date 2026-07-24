---
title: Combobox
description: "可检索选项选择。"
---

# Combobox

Combobox 在选择列表上增加本地查询过滤，适合候选项较多但仍可一次加载的场景。

```rust
let mut city = use_signal(String::new);

Combobox {
    options: cities(),
    selected: city(),
    on_select: move |next| city.set(next),
}
```

| 属性             | 类型                           | 默认值           | 说明                           |
| ---------------- | ------------------------------ | ---------------- | ------------------------------ |
| `options`        | `Vec<String>`                  | 必填             | 可过滤候选项                   |
| `placeholder`    | `Option<String>`               | 当前 i18n locale | 未选中时的文案                 |
| `label`          | `Option<String>`               | 当前 i18n locale | 候选列表分组标题；空字符串隐藏 |
| `selected`       | `String`                       | 空字符串         | 当前选中值                     |
| `open`           | `Option<bool>`                 | `None`           | 受控浮层状态                   |
| `default_open`   | `bool`                         | `false`          | 非受控初始状态                 |
| `on_open_change` | `Option<EventHandler<bool>>`   | `None`           | 打开状态变化                   |
| `on_select`      | `Option<EventHandler<String>>` | `None`           | 选择回调                       |

查询文本是组件的临时交互状态，`selected` 才是表单值。远程搜索、分页和防抖应由业务封装，不要把海量列表一次传入组件。

默认 placeholder 与分组标题支持 `en-US`/`zh-CN` 响应式切换；显式传值后固定使用业务文案。
