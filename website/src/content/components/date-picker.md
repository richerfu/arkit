---
title: Date Picker
description: "底部面板日期选择器。"
---

# Date Picker

DatePicker 组合 outline trigger、BottomSheet 与 Calendar，适合表单中的单日期选择。

```rust
let mut birthday = use_signal(|| None::<String>);

DatePicker {
    selected: birthday(),
    on_change: move |next| birthday.set(next),
}
```

| 属性               | 类型                           | 默认值           | 说明                     |
| ------------------ | ------------------------------ | ---------------- | ------------------------ |
| `selected`         | `Option<String>`               | `None`           | 受控日期                 |
| `default_selected` | `Option<String>`               | `None`           | 非受控初始日期           |
| `placeholder`      | `Option<String>`               | 当前 i18n locale | 未选择时文案             |
| `close_label`      | `Option<String>`               | 当前 i18n locale | 面板关闭按钮文案         |
| `calendar_labels`  | `Option<CalendarLabels>`       | 当前 i18n locale | 日历月份、星期及标题格式 |
| `open`             | `Option<bool>`                 | `None`           | 受控面板状态             |
| `default_open`     | `bool`                         | `false`          | 非受控初始状态           |
| `disabled`         | `bool`                         | `false`          | 禁止打开                 |
| `on_change`        | `EventHandler<Option<String>>` | 默认空处理       | 日期变化；再次选择可清空 |
| `on_open_change`   | `EventHandler<bool>`           | 默认空处理       | 面板状态变化             |

受控 `open` 场景必须在 `on_open_change` 中同步外部状态。DatePicker 选择的是日期，不负责时间或日期范围。

placeholder、关闭按钮和内嵌 Calendar 默认共享应用的 `I18nContext`，支持
`en-US`/`zh-CN` 切换；任一字段显式传入后只覆盖对应文案。
