---
title: Badge
description: "紧凑状态标签。"
---

# Badge

Badge 展示简短状态、分类或计数，不承载复杂操作。

## 用法

```rust
Badge {
    content: "Beta",
    variant: BadgeVariant::Secondary,
    icon: "sparkles",
    pill: true,
}
```

## Props

| Prop          | 类型                 | 说明                                     |
| ------------- | -------------------- | ---------------------------------------- |
| `content`     | `String`             | 标签文本                                 |
| `variant`     | `BadgeVariant`       | Default、Secondary、Destructive、Outline |
| `icon`        | `Option<String>`     | Lucide 图标名                            |
| `icon_colors` | `Option<(u32, u32)>` | 覆盖背景色和前景色                       |
| `pill`        | `Option<bool>`       | 使用 full radius 与紧凑 padding          |

默认最小高度 22vp，图标 12vp。content 保持短小；需要完整说明使用 Alert 或 Text。
