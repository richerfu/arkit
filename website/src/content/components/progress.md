---
title: Progress
description: "能算出百分比时的进度条。"
---

# Progress

进度确定时用的进度条。

## Props

| Prop                    | 默认值        | 说明                      |
| ----------------------- | ------------- | ------------------------- |
| `value`                 | 必填          | 当前值，超出范围会 clamp  |
| `total`                 | 100           | 总量；非正/非有限时显示空 |
| `height`                | 8vp           | track 高度                |
| `track_color`           | primary track | 轨道颜色                  |
| `indicator_color`       | primary       | 进度颜色                  |
| `radius`                | full          | 圆角                      |
| `animated`              | true          | 是否动画更新              |
| `animation_duration_ms` | 150           | 更新动画时长              |

Progress 不显示文字。业务应同时展示百分比、数量或阶段；高频更新时合并同一帧的值，避免排队播放过时进度。
