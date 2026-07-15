---
title: Label
---

# Label

Label 是小号、中等字重的表单标签。

```rust
column {
    Label { content: "用户名" }
    Input {
        value: name(),
        on_change: move |next| name.set(next),
    }
}
```

## Props

| Prop      | 类型     | 说明     |
| --------- | -------- | -------- |
| `content` | `String` | 标签文字 |

Label 只负责视觉，不提供 HTML `for` 绑定。复杂表单优先使用 Form 页中的 `FieldLabel`，它支持 required 与 invalid 状态，并与 FieldDescription/FieldError 形成完整结构。
