---
title: Resizable
description: "双栏内容与分隔布局。"
---

# Resizable

Resizable 是带视觉分隔线的双栏容器。

```rust
Resizable {
    left: rsx! { FileTree {} },
    right: rsx! { Editor {} },
}
```

| 属性    | 类型      | 说明     |
| ------- | --------- | -------- |
| `left`  | `Element` | 左侧内容 |
| `right` | `Element` | 右侧内容 |

当前实现提供固定的双栏排列和分隔视觉，不暴露拖拽尺寸状态。需要真正可拖拽的 pane 时，应基于指针事件与宽度约束封装业务组件，并为小屏提供单栏降级。
