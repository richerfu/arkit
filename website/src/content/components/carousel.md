---
title: Carousel
description: "轮播内容，带控制器和指示点。"
---

# Carousel

横向或纵向轮播，附带控制器和指示点。

## 主要 Props

| 属性                                | 默认值         | 说明                                   |
| ----------------------------------- | -------------- | -------------------------------------- |
| `slides`                            | 必填           | 每页一个根 Element                     |
| `index` / `default_index`           | 非受控 / `0`   | 当前页与初始页                         |
| `height`                            | `240.0`        | Swiper 视口高度                        |
| `looping` / `autoplay`              | `false`        | 循环和自动播放                         |
| `interval_ms` / `duration_ms`       | `3000` / `300` | 自动播放间隔和切换时长                 |
| `transition_curve`                  | 默认曲线       | `CarouselTransitionCurve`              |
| `item_spacing`                      | `0`            | 相邻页面间距                           |
| `swipe_enabled`                     | `true`         | 是否响应触摸滑动                       |
| `show_controls` / `show_indicators` | `true`         | 导航按钮和指示器                       |
| `controls_placement`                | `Below`        | `Below`、`Overlay`、`OverlayCenter`    |
| `style`                             | 默认主题       | `CarouselStyle` 视口、导航、指示器配置 |

各 slide 应保持一致主要尺寸；图片提前使用 AspectRatio，避免翻页时布局抖动。自动播放内容必须允许用户手动控制。
