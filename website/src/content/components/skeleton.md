---
title: Skeleton
---

# Skeleton

Skeleton 是固定尺寸加载占位，使用 Theme accent 色。

```rust
column {
    Skeleton { width: 160.0, height: 20.0 }
    Skeleton { width: 40.0, height: 40.0 }
}
```

## Props

| Prop     | 类型  | 说明    |
| -------- | ----- | ------- |
| `width`  | `f32` | 宽度 vp |
| `height` | `f32` | 高度 vp |

宽高相等且至少 40vp 时自动使用圆形 radius，否则使用 md radius。Skeleton 应匹配最终内容的主要几何，数据加载失败后切换 Error 状态，不无限保留占位。
