---
title: Dropdown Menu
---

# Dropdown Menu

DropdownMenu 点击 trigger 后打开锚定菜单，支持 action、submenu、checkbox、radio、label 与 separator 条目。

```rust
let items = vec![
    MenuEntry::action("重命名")
        .icon("pencil")
        .on_select(EventHandler::new(move |_| rename())),
    MenuEntry::separator(),
    MenuEntry::action("删除")
        .destructive()
        .on_select(EventHandler::new(move |_| remove())),
];

DropdownMenu {
    items,
    width: 224.0,
    Button { "更多" }
}
```

| 属性                            | 说明                                      |
| ------------------------------- | ----------------------------------------- |
| `items: Vec<DropdownMenuEntry>` | `DropdownMenuEntry` 是 `MenuEntry` 的别名 |
| `children: Element`             | 点击 trigger                              |
| `open` / `default_open`         | 受控状态与非受控初始值                    |
| `on_open_change`                | 状态通知                                  |
| `width`                         | 面板宽度                                  |
| `trigger_capture`               | 兼容配置；当前实现不改变行为              |

Builder 还支持 `disabled`、`inset`、`shortcut` 和 `icon`。选择 action 后通常关闭；破坏性操作应进入 AlertDialog。
