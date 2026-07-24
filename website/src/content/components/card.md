---
title: Card
description: "内容卡片，可拆 Header / Content / Footer。"
---

# Card

一块有边界的内容容器，可以拆成 Header、正文和 Footer。

## 快速组合

```rust
Card {
    CardHeader {
        title: "账户",
        description: "管理公开资料",
    }
    CardContent { text { "Ada" } }
    CardFooter { Button { "编辑" } }
}
```

## Compound API

| 组件              | Props                  |
| ----------------- | ---------------------- |
| `Card`            | `children`             |
| `CardHeader`      | `title`、`description` |
| `CardTitle`       | `content`              |
| `CardDescription` | `content`              |
| `CardContent`     | `children`             |
| `CardFooter`      | `children`             |

CardHeader 是完整标题/说明组合；需要自定义 header 布局时使用 CardTitle/CardDescription。Card 不持有点击状态，整卡可点击时要避免与 Footer 内按钮形成冲突手势。
