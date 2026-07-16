---
title: Sonner
description: "安全区感知的 Toast 队列。"
---

# Sonner

Sonner 是安全区域感知的全局 Toast 堆栈。调用方持有 `Vec<SonnerToast>`，每个 live toast 必须有唯一稳定 id。

```rust
let toast = SonnerToast::success(42, "保存成功")
    .description("资料已同步")
    .duration_ms(3000);

Sonner {
    toasts: vec![toast],
    position: SonnerPosition::BottomCenter,
    visible_toasts: 3,
    rich_colors: true,
}
```

| 属性             | 默认值         | 说明                                                |
| ---------------- | -------------- | --------------------------------------------------- |
| `toasts`         | 空             | 结构化 `SonnerToast` 列表                           |
| `messages`       | 空             | 兼容用纯文本列表，新代码优先 toasts                 |
| `position`       | `BottomCenter` | 上/下 × 左/中/右六种位置                            |
| `visible_toasts` | `3`            | 同时可见数量，至少 1                                |
| `rich_colors`    | `false`        | 语义强化配色                                        |
| `style`          | 默认           | 最大宽度、safe-area offset、inset、gap 与 card 样式 |

`SonnerToast` 提供 default/success/info/warning/error/loading constructors，以及 description、duration、action、dismiss builder。`duration_ms(0)` 保持到显式关闭；Loading 默认保持。timer、action、关闭按钮和 swipe 都必须最终从调用方列表清理对应 id。
