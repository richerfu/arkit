---
title: Card
description: "卡片及 Header、Content、Footer。"
---

# Card

Card 是带 border、radius、shadow 和主题色的内容 surface。

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
