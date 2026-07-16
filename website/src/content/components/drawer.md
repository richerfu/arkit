---
title: Drawer
description: "抽屉式面板。"
---

# Drawer

Drawer 是较宽的边缘模态面板，默认从底部进入，可用于移动端导航、详情或临时工作区。

```rust
Drawer {
    title: "项目导航",
    side: "left",
    open: open(),
    on_close: move |_| open.set(false),
    ProjectNavigation {}
}
```

Props 与 Sheet 一致：`title`、`side`、`open`、`default_open`、`on_close` 和 `children`；Drawer 的默认 side 是 `bottom`。

Drawer 不拥有路由状态。导航项点击后先派发 typed Route，再关闭面板；异步内容加载失败时在 Drawer 内保留可重试状态。
