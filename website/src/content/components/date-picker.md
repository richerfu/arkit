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
    placeholder: "选择出生日期".into(),
    close_label: "关闭".into(),
    calendar_labels: CalendarLabels::new(
        ["日", "一", "二", "三", "四", "五", "六"].map(str::to_owned),
        ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"].map(str::to_owned),
        "{year}年{month}",
    ),
    on_change: move |next| birthday.set(next),
}
```

| 属性               | 类型                           | 默认值     | 说明                     |
| ------------------ | ------------------------------ | ---------- | ------------------------ |
| `selected`         | `Option<String>`               | `None`     | 受控日期                 |
| `default_selected` | `Option<String>`               | `None`     | 非受控初始日期           |
| `placeholder`      | `String`                       | 必填       | 未选择时文案             |
| `close_label`      | `String`                       | 必填       | 面板关闭按钮文案         |
| `calendar_labels`  | `CalendarLabels`               | 必填       | 日历月份、星期及标题格式 |
| `open`             | `Option<bool>`                 | `None`     | 受控面板状态             |
| `default_open`     | `bool`                         | `false`    | 非受控初始状态           |
| `disabled`         | `bool`                         | `false`    | 禁止打开                 |
| `on_change`        | `EventHandler<Option<String>>` | 默认空处理 | 日期变化；再次选择可清空 |
| `on_open_change`   | `EventHandler<bool>`           | 默认空处理 | 面板状态变化             |

受控 `open` 场景必须在 `on_open_change` 中同步外部状态。DatePicker 选择的是日期，不负责时间或日期范围。
