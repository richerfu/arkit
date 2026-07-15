---
title: Progress
---

# Progress

Progress 是受控水平进度条，值变化通过 Arkit Animatable 平滑过渡。

```rust
Progress {
    value: uploaded(),
    total: 100.0,
    height: 8.0,
    animated: true,
    animation_duration_ms: 150,
}
```

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
