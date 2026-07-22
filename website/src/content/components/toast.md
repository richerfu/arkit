---
title: Toast
description: "单条操作反馈。"
---

# Toast

Toast 展示一个短时结果卡片；它本身不管理队列和计时。需要全局堆栈时使用 Sonner。

完整通知：

```rust
Toast {
    message: "保存成功",
    description: "资料已同步",
    variant: ToastVariant::Success,
    appearance: ToastAppearance::Notification,
    action_label: "撤销",
    dismissible: true,
    on_action: move |_| undo(),
    on_dismiss: move |_| hide(),
}
```

极简芯片（shadcn popover 表面，内容 hug 宽度，无 description / action / 关闭按钮）：

```rust
Toast {
    message: "Copied",
    appearance: ToastAppearance::Minimal,
    dismissible: false,
}
```

| 属性                         | 说明                                                        |
| ---------------------------- | ----------------------------------------------------------- |
| `message`、`description`     | 主文案与可选说明（Minimal 忽略 description）                |
| `variant`                    | `Default`、`Success`、`Info`、`Warning`、`Error`、`Loading` |
| `appearance`                 | `Notification`（默认）或 `Minimal`                          |
| `action_label` / `on_action` | 可选操作（Minimal 不展示）                                  |
| `icon`                       | 覆盖变体默认图标                                            |
| `dismissible`                | 是否显示关闭入口，默认 true；Minimal 通常为 false           |
| `rich_colors`                | 使用更强的语义配色                                          |
| `swipe_direction`            | `Up` 或 `Down`                                              |
| `stack_count` / `on_cycle`   | 重叠栈数量与滚动切换回调（Sonner 内部使用）                 |
| `style`                      | `ToastStyle` 颜色、圆角、高度和阴影覆盖                     |
| `on_dismiss`                 | 关闭按钮或滑动回调                                          |

`ToastDestructive { message }` 是 Error 变体快捷组件。字段错误用 FieldError，不要把每次输入错误升级为全局 Toast。
