---
title: Sheet
description: "四边侧滑面板。"
---

# Sheet

Sheet 从视口边缘进入，适合设置、筛选和辅助面板。默认从右侧打开，固定使用模态遮罩。

```rust
Sheet {
    title: "筛选",
    side: "right",
    open: open(),
    on_close: move |_| open.set(false),
    FilterForm {}
}
```

| 属性                    | 类型                       | 说明                                         |
| ----------------------- | -------------------------- | -------------------------------------------- |
| `title`                 | `String`                   | 面板标题                                     |
| `side`                  | `Option<String>`           | `top`、`bottom`、`left`、`right`；默认 right |
| `open` / `default_open` | `Option<bool>`             | 受控状态与初始状态                           |
| `on_close`              | `Option<EventHandler<()>>` | 关闭回调                                     |
| `children`              | `Element`                  | 面板内容                                     |

长内容使用 ScrollArea。窗口尺寸与键盘变化时不要缓存绝对坐标；小屏筛选面板通常使用 bottom，宽屏辅助面板使用 right。
