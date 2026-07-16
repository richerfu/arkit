---
title: Breadcrumb
description: "页面层级路径。"
---

# Breadcrumb

Breadcrumb 展示当前位置层级；它只负责视觉，不直接改变 Router。

```rust
Breadcrumb {
    items: vec!["项目".into(), "设置".into(), "成员".into()],
}
```

`Breadcrumb` 接收 `items: Vec<String>`，最后一项使用当前页样式，其余项为 muted 文本并自动添加分隔符。`BreadcrumbItem` 可单独渲染一个 `content: String`。

当前组件不提供点击回调。需要可点击路径时可自行组合 BreadcrumbItem 与 Button，并让点击事件派发 typed Route。移动端应压缩中间层级，避免一行溢出。
