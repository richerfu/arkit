---
title: Text
---

# Text

Text 把项目排版层级映射为主题化 ArkUI text。

## 用法

```rust
Text { content: "页面标题", variant: TextVariant::H1 }
Text { content: "辅助说明", variant: TextVariant::Muted }
```

## Props

| Prop      | 类型          | 说明     |
| --------- | ------------- | -------- |
| `content` | `String`      | 文本内容 |
| `variant` | `TextVariant` | 排版层级 |

## Variant

Default、H1、H2、H3、P、Blockquote、Code、Lead、Large、Small、Muted。它们分别定义字号、字重、行高、颜色和必要的边框/背景。

Text 适合稳定设计层级；需要 max_lines、overflow、动态 alignment 等底层属性时使用原生 `text`，或在业务组件中封装新的语义 variant。

不要用 H1/H2 仅为了“看起来更大”；标题层级应与页面结构一致。
