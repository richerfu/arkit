---
title: Toast
description: "单条操作反馈。"
---

# Toast

Toast 展示一个短时结果卡片；它本身不管理队列和计时。需要全局堆栈时使用 Sonner。

```rust
Toast {
    message: "保存成功",
    description: "资料已同步",
    variant: ToastVariant::Success,
    action_label: "撤销",
    dismissible: true,
    on_action: move |_| undo(),
    on_dismiss: move |_| hide(),
}
```

| 属性                         | 说明                                                        |
| ---------------------------- | ----------------------------------------------------------- |
| `message`、`description`     | 主文案与可选说明                                            |
| `variant`                    | `Default`、`Success`、`Info`、`Warning`、`Error`、`Loading` |
| `action_label` / `on_action` | 可选操作                                                    |
| `icon`                       | 覆盖变体默认图标                                            |
| `dismissible`                | 是否显示关闭入口，默认 true                                 |
| `rich_colors`                | 使用更强的语义配色                                          |
| `swipe_direction`            | `Up` 或 `Down`                                              |
| `style`                      | `ToastStyle` 颜色、圆角、高度和阴影覆盖                     |
| `on_dismiss`                 | 关闭按钮或滑动回调                                          |

`ToastDestructive { message }` 是 Error 变体快捷组件。字段错误用 FieldError，不要把每次输入错误升级为全局 Toast。
