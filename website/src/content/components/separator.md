---
title: Separator
description: "水平和垂直分隔。"
---

# Separator

Separator 使用 Theme border token 绘制 1vp 分隔线。

```rust
Separator {}
Separator { vertical_height: 24.0 }
```

## Props

| Prop              | 类型          | 说明                                     |
| ----------------- | ------------- | ---------------------------------------- |
| `vertical_height` | `Option<f32>` | None 为全宽水平线；Some(height) 为垂直线 |

Separator 表达内容分组，不用于填充任意间距。间距使用 `spacing` token 或容器 margin。
