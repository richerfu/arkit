---
title: Menubar
description: "多菜单命令栏。"
---

# Menubar

Menubar 组合多个带标题的菜单，适合平板或桌面式命令栏。所有菜单复用 `MenuEntry` 模型。

```rust
Menubar {
    menus: vec![
        MenubarMenuSpec::new("文件", vec![
            MenuEntry::action("新建").shortcut("Ctrl+N"),
            MenuEntry::separator(),
            MenuEntry::action("关闭"),
        ]),
        MenubarMenuSpec::new("编辑", vec![
            MenuEntry::action("复制").shortcut("Ctrl+C"),
        ]),
    ],
    on_active_change: move |index| tracing::debug!(?index),
}
```

| 属性               | 类型                                  | 说明                               |
| ------------------ | ------------------------------------- | ---------------------------------- |
| `menus`            | `Vec<MenubarMenuSpec>`                | 菜单标题与条目                     |
| `active`           | `Option<Option<usize>>`               | 受控打开菜单；外层 None 表示非受控 |
| `default_active`   | `Option<usize>`                       | 初始打开索引                       |
| `on_active_change` | `Option<EventHandler<Option<usize>>>` | 当前菜单变化                       |

移动页面上的单组操作优先 DropdownMenu。快捷键文案只是视觉提示，真正的键盘监听仍由应用处理。
