---
title: Spinner
description: "不知道进度时用的转圈指示。"
---

# Spinner

进度还算不出来时，先转个圈。

## Props

| Prop           | 默认值           | 说明                       |
| -------------- | ---------------- | -------------------------- |
| `size`         | 16vp             | 宽高                       |
| `color`        | theme foreground | 指示器颜色                 |
| `icon`         | None             | 自定义 Lucide 图标名       |
| `stroke_width` | 2.0              | 自定义图标描边             |
| `spinning`     | true             | 是否播放，false 时保留布局 |

任务完成后移除 Spinner 或设置 spinning=false。已知总量的上传、下载使用 Progress。
