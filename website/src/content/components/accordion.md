---
title: Accordion
---

# Accordion

Accordion 在一组条目中展开一个内容区，适合 FAQ 和次要设置。每项由 `AccordionItemSpec` 描述。

```rust
let items = vec![
    AccordionItemSpec::new("如何安装？", "install", rsx! {
        Text { "在 Cargo.toml 中添加 arkit。" }
    }),
    AccordionItemSpec::new("支持哪些平台？", "platforms", rsx! {
        Text { "当前面向 HarmonyOS。" }
    }),
];

Accordion {
    items,
    default_value: Some("install".into()),
    collapsible: true,
    on_value_change: move |value| tracing::debug!(?value),
}
```

| 属性              | 类型                                   | 说明                                   |
| ----------------- | -------------------------------------- | -------------------------------------- |
| `items`           | `Vec<AccordionItemSpec>`               | 标题、稳定 value、内容和 disabled 状态 |
| `value`           | `Option<Option<String>>`               | 受控展开值；外层 `None` 表示非受控     |
| `default_value`   | `Option<String>`                       | 非受控初始展开值                       |
| `collapsible`     | `bool`                                 | 已展开项能否再次关闭                   |
| `on_value_change` | `Option<EventHandler<Option<String>>>` | 展开值变化                             |

动态列表必须使用稳定业务 value。关键错误、唯一提交入口或高频内容不应藏在默认关闭的条目中。
