---
title: Select
description: "锚点下拉选择。"
---

# Select

Select 从中等规模的字符串列表中选择一个值，支持选择值和打开状态分别受控。

```rust
let mut fruit = use_signal(String::new);

Select {
    options: vec!["Apple".into(), "Pear".into()],
    selected: fruit(),
    on_select: move |next| fruit.set(next),
}
```

| 属性               | 类型                           | 默认值           | 说明                       |
| ------------------ | ------------------------------ | ---------------- | -------------------------- |
| `options`          | `Vec<String>`                  | 必填             | 候选项                     |
| `placeholder`      | `Option<String>`               | 当前 i18n locale | 未选中时的文案             |
| `label`            | `Option<String>`               | 当前 i18n locale | 下拉分组标题；空字符串隐藏 |
| `selected`         | `Option<String>`               | `None`           | 受控选中值                 |
| `default_selected` | `String`                       | 空字符串         | 非受控初始值               |
| `open`             | `Option<bool>`                 | `None`           | 受控打开状态               |
| `default_open`     | `bool`                         | `false`          | 非受控初始状态             |
| `on_open_change`   | `Option<EventHandler<bool>>`   | `None`           | 打开状态变化               |
| `on_select`        | `Option<EventHandler<String>>` | `None`           | 选中回调                   |

当前 option 字符串同时承担值和标签。业务对象应使用稳定 id，并在进入组件前映射显示文案；需要搜索时改用 Combobox。

默认 placeholder 与分组标题支持 `en-US`/`zh-CN` 响应式切换；显式传值后固定使用业务文案。

弹出层相对 **Select 触发器根节点** 锚点（`side=bottom`、`align=start`、宽度≈触发器），与 shadcn 一致；不是相对页面布局对齐。几何以 `use_layout_frame` 测量为准，不使用点击命中子节点的 bounds。
