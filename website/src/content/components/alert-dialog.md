---
title: Alert Dialog
description: "强确认模态框。"
---

# Alert Dialog

AlertDialog 用于破坏性操作或必须由用户明确确认的选择，并为 cancel/action 保留清晰的语义位置。

```rust
AlertDialog {
    title: "删除项目？",
    description: "该操作无法撤销。",
    open: deleting(),
    on_close: move |_| deleting.set(false),
    cancel: rsx! {
        Button { variant: ButtonVariant::Outline, "取消" }
    },
    action: rsx! {
        Button { variant: ButtonVariant::Destructive, "删除" }
    },
}
```

| 属性                    | 类型                       | 说明                     |
| ----------------------- | -------------------------- | ------------------------ |
| `title` / `description` | `String`                   | 明确说明操作与结果       |
| `open` / `default_open` | `Option<bool>`             | 受控状态与非受控初始状态 |
| `on_close`              | `Option<EventHandler<()>>` | 遮罩、取消或关闭回调     |
| `cancel` / `action`     | `Option<Element>`          | 取消和确认操作           |
| `children`              | `Element`                  | 可选补充内容             |

普通信息使用 Dialog 或 Toast。可撤销操作优先直接执行并提供 Toast Undo，减少无意义确认。
