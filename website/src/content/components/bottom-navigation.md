---
title: Bottom Navigation
description: "移动端底部主导航。"
---

# Bottom Navigation

BottomNavigation 用于移动端 3–5 个顶级目的地，每项包含标签和 Lucide 图标名。

```rust
BottomNavigation {
    items: vec![
        BottomNavigationItem::new("首页", "house"),
        BottomNavigationItem::new("搜索", "search"),
        BottomNavigationItem::new("我的", "user"),
    ],
    selected: selected(),
    on_select: move |index| selected.set(index),
}
```

| 属性               | 类型                        | 说明           |
| ------------------ | --------------------------- | -------------- |
| `items`            | `Vec<BottomNavigationItem>` | 目的地列表     |
| `selected`         | `Option<usize>`             | 受控选中索引   |
| `default_selected` | `usize`                     | 非受控初始索引 |
| `on_select`        | `EventHandler<usize>`       | 选择回调       |

组件会把越界索引限制到有效范围。每个 tab 独立的返回栈、深链与状态恢复属于 Router 架构，不由导航栏保存。
