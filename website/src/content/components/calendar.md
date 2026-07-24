---
title: Calendar
description: "月视图日期选择。"
---

# Calendar

Calendar 按月展示日期，日期值统一使用 `YYYY-MM-DD` 字符串。

```rust
let mut selected = use_signal(|| None::<String>);

Calendar {
    selected: selected(),
    initial_month: "2026-07",
    embedded: true,
    on_day_press: move |day| selected.set(Some(day)),
}
```

| 属性              | 类型                     | 默认值           | 说明                     |
| ----------------- | ------------------------ | ---------------- | ------------------------ |
| `selected`        | `Option<String>`         | `None`           | 单选日期                 |
| `selected_dates`  | `Vec<String>`            | 空               | 多个高亮日期             |
| `initial_month`   | `Option<String>`         | 当前月           | 初始月份，格式 `YYYY-MM` |
| `labels`          | `Option<CalendarLabels>` | 当前 i18n locale | 月份、星期与标题格式     |
| `selection_color` | `Option<u32>`            | 主题色           | 选中颜色                 |
| `today_color`     | `Option<u32>`            | 主题色           | 今天标记颜色             |
| `embedded`        | `bool`                   | `false`          | 嵌入式外观               |
| `on_day_press`    | `EventHandler<String>`   | 默认空处理       | 日期点击回调             |

纯日期不等同于本地午夜时间戳。与后端通信时应显式约定时区和数据格式，避免跨时区后日期偏移。

内置 `en-US` 与 `zh-CN` 文案随应用 `I18nContext` 切换。需要覆盖时传入
`Some(CalendarLabels::new(weekdays, months, month_title_template))`；
标题模板支持 `{year}` 和 `{month}`。`CalendarLabels::english()` 仍可用于
强制固定英文，不受 locale 变化影响。
