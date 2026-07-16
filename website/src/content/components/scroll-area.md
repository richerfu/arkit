---
title: Scroll Area
description: "主题化滚动容器。"
---

# Scroll Area

ScrollArea 提供带主题背景、边框和圆角的滚动 surface。

```rust
ScrollArea {
    column {
        for item in items() {
            ListRow { item }
        }
    }
}
```

唯一属性 `children: Element` 是滚动内容。父布局需要给出可确定高度，否则 ScrollArea 无法形成滚动视口。

避免同轴嵌套滚动。大量数据仍应使用基础 `list`/`grid` 与虚拟化 hooks，而不是一次创建全部 child。
