---
title: Separator
description: "横线或竖线，用来隔开内容区块。"
---

# Separator

横线或竖线，用来在视觉上切开区块。

## Props

| Prop              | 类型          | 说明                                     |
| ----------------- | ------------- | ---------------------------------------- |
| `vertical_height` | `Option<f32>` | None 为全宽水平线；Some(height) 为垂直线 |

Separator 表达内容分组，不用于填充任意间距。间距使用 `spacing` token 或容器 margin。
