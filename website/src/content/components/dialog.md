---
title: Dialog
description: "居中模态框及 Header、Footer。"
---

# Dialog

Dialog 是居中的模态面板，通过 OverlayRoot 渲染，不参与页面原有布局。面板最大宽度 512vp，点击遮罩或关闭按钮触发 `on_close`。

```rust
let mut open = use_signal(|| false);

Dialog {
    open: open(),
    on_close: move |_| open.set(false),
    DialogHeader {
        title: "编辑资料",
        description: "修改后立即生效",
    }
    ProfileForm {}
    DialogFooter {
        Button { onclick: move |_| save(), "保存" }
    }
}
```

| 组件           | 属性                                           | 说明               |
| -------------- | ---------------------------------------------- | ------------------ |
| `Dialog`       | `open`、`default_open`、`on_close`、`children` | 模态容器           |
| `DialogHeader` | `title`、`description`                         | 居中标题与可选说明 |
| `DialogFooter` | `children`                                     | 操作区             |

受控模式下 `on_close` 必须把外部 `open` 设为 false。提交失败应保持弹窗打开并展示字段错误；长内容放入 ScrollArea。
