---
title: Sidebar
---

# Sidebar

Sidebar 把固定宽度的侧边导航与主内容并排展示，SidebarItem 提供选中视觉和点击事件。

```rust
Sidebar {
    sidebar: rsx! {
        column {
            SidebarItem { title: "概览", active: true }
            SidebarItem { title: "成员", active: false }
        }
    },
    MainContent {}
}
```

| 组件          | 属性                                                                         |
| ------------- | ---------------------------------------------------------------------------- |
| `Sidebar`     | `sidebar: Element`、`children: Element`                                      |
| `SidebarItem` | `title: String`、`active: Option<bool>`、`onclick: Option<EventHandler<()>>` |

Sidebar 适合宽屏主导航。小屏通过 `use_window_metrics` 切换到 Sheet、Drawer 或 BottomNavigation；active 状态从 Router 派生。
