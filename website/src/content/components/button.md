---
title: Button
description: "主题化按钮，覆盖常见变体、尺寸和禁用态。"
---

# Button

主题化按钮：变体、尺寸、禁用和点击回调都按常见 shadcn 习惯暴露。

## 用法

```rust
Button {
    variant: ButtonVariant::Outline,
    size: ButtonSize::Sm,
    disabled: Some(false),
    width: "100%",
    onclick: move |_| save(),
    "保存"
}
```

## Props

| Prop       | 类型               | 说明                                                  |
| ---------- | ------------------ | ----------------------------------------------------- |
| `variant`  | `ButtonVariant`    | Default、Secondary、Outline、Ghost、Destructive、Link |
| `size`     | `ButtonSize`       | Default 48vp、Sm 36vp、Lg 56vp、Icon 40×40vp          |
| `disabled` | `Option<bool>`     | 禁止 native event，并降低透明度                       |
| `width`    | `Option<String>`   | CSS 宽度（`"100%"`、`"120"`）；未设时走 size 默认     |
| `onclick`  | `EventHandler<()>` | 点击回调                                              |
| `children` | `Element`          | 文本、图标或自定义行                                  |

## 行为

Button 不持有提交状态。异步任务期间由调用方设置 disabled，并在 children 中切换 Spinner/文案。Destructive 只提供视觉语义，不自动弹确认框。

Icon size 只控制外框；children 中的图标自行选择 16–20vp。Link variant 是按钮视觉，不替代 Router Link。
