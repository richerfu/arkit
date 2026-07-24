---
title: Label
description: "给输入控件配可读标签。"
---

# Label

给控件配可读标签，点标签也能聚焦到对应输入。

## Props

| Prop      | 类型     | 说明     |
| --------- | -------- | -------- |
| `content` | `String` | 标签文字 |

Label 只负责视觉，不提供 HTML `for` 绑定。复杂表单优先使用 Form 页中的 `FieldLabel`，它支持 required 与 invalid 状态，并与 FieldDescription/FieldError 形成完整结构。
