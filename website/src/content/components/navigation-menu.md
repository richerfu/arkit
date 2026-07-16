---
title: Navigation Menu
description: "主导航菜单。"
---

# Navigation Menu

NavigationMenu 是横向导航容器，NavigationItem 使用 Button 的 active/ghost 外观。

```rust
NavigationMenu {
    NavigationItem {
        title: "首页",
        active: matches!(route(), Route::Home),
        onclick: move |_| router.push(Route::Home),
    }
    NavigationItem {
        title: "设置",
        active: matches!(route(), Route::Settings),
        onclick: move |_| router.push(Route::Settings),
    }
}
```

`NavigationMenu` 只有 `children`；`NavigationItem` 提供 `title`、`active: Option<bool>` 与 `onclick: Option<EventHandler<()>>`。

active 状态应从当前 typed Route 派生。不要在点击时先维护另一份本地选中状态，否则路由失败或系统返回后容易出现两个真相源。
