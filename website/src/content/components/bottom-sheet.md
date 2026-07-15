---
title: Bottom Sheet
---

# Bottom Sheet

BottomSheet 是针对移动交互优化的底部模态面板，带拖拽 handle、可选 header，并考虑输入法视口。

```rust
BottomSheet {
    title: "添加备注",
    open: open(),
    show_header: true,
    on_close: move |_| open.set(false),
    BottomSheetTextInput {
        value: note(),
        placeholder: "输入备注",
        on_change: move |next| note.set(next),
    }
}
```

| 组件                   | 属性                                                                   | 说明                   |
| ---------------------- | ---------------------------------------------------------------------- | ---------------------- |
| `BottomSheet`          | `title`、`open`、`default_open`、`show_header`、`on_close`、`children` | 底部面板               |
| `BottomSheetTextInput` | `placeholder`、`value`、`on_change`                                    | 面板内的 56vp 受控输入 |

受控打开状态由调用方同步关闭。内容较长时使用 ScrollArea；表单提交失败不要自动关闭面板。
