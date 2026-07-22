---
title: AspectRatio
description: "固定媒体宽高比。"
---

# AspectRatio

AspectRatio 创建全宽 Stack，并用 ArkUI `aspect_ratio` 约束高度。

```rust
AspectRatio {
    ratio: 16.0 / 9.0,
    image {
        width: "100%",
        height: "100%",
        src: cover,
    }
}
```

## Props

| Prop       | 类型      | 说明           |
| ---------- | --------- | -------------- |
| `ratio`    | `f32`     | width / height |
| `children` | `Element` | 容器内容       |

父容器必须给出可确定宽度。ratio 应为有限正值；需要固定高度时直接使用普通 Stack，不同时给出互相冲突的高度和 ratio。
