---
title: Context Menu
---

# Context Menu

ContextMenu 通过长按 trigger 打开菜单，普通点击仍由 child 自己处理。菜单条目能力与 DropdownMenu 相同。

```rust
ContextMenu {
    items: vec![
        MenuEntry::action("复制"),
        MenuEntry::action("删除").destructive(),
    ],
    on_open_change: move |open| tracing::debug!(open),
    FileRow { file }
}
```

| 属性             | 类型                         | 说明             |
| ---------------- | ---------------------------- | ---------------- |
| `items`          | `Vec<ContextMenuEntry>`      | `MenuEntry` 条目 |
| `children`       | `Element`                    | 长按触发区域     |
| `open`           | `Option<bool>`               | 受控状态         |
| `default_open`   | `bool`                       | 非受控初始状态   |
| `on_open_change` | `Option<EventHandler<bool>>` | 状态通知         |
| `width`          | `Option<f32>`                | 面板宽度         |

重要动作不能只藏在长按手势中，应提供可发现的更多按钮。动态条目保持稳定顺序和业务 identity。
