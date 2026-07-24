---
title: AspectRatio
description: "给图片或视频框住固定宽高比。"
---

# AspectRatio

给媒体区域锁住宽高比，避免图片或视频把布局撑得忽高忽低。

## Props

| Prop       | 类型      | 说明           |
| ---------- | --------- | -------------- |
| `ratio`    | `f32`     | width / height |
| `children` | `Element` | 容器内容       |

父容器必须给出可确定宽度。ratio 应为有限正值；需要固定高度时直接使用普通 Stack，不同时给出互相冲突的高度和 ratio。
