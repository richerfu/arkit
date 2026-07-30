---
title: Guide
description: "分步高亮界面元素，做功能导览。"
---

# Guide

分步高亮界面上的目标元素，适合做首次使用的功能导览。

## 主要 Props

| 属性                       | 默认值       | 说明                                     |
| -------------------------- | ------------ | ---------------------------------------- |
| `steps`                    | 必填         | `GuideStep` 列表，目标 ID 必须唯一       |
| `open` / `default_open`    | 非受控 / 否  | 受控打开状态与非受控初始状态             |
| `step` / `default_step`    | 非受控 / `0` | 受控步骤索引与非受控初始步骤             |
| `labels`                   | 当前语言     | 覆盖上一步、下一步、跳过和完成文案       |
| `style`                    | 默认样式     | 面板宽高估值、高亮边距、侧边距和遮罩颜色 |
| `allow_target_interaction` | `false`      | 是否允许点击穿过高亮区域触达原目标       |
| `on_open_change`           | 无           | 打开状态变化                             |
| `on_step_change`           | 无           | 上一步或下一步后的索引                   |
| `on_skip` / `on_finish`    | 无           | 跳过与完成回调                           |

`GuideStep::side` 设置首选方向；空间不足时组件会自动翻转。内置操作文案跟随 `arkit_i18n::I18nContext`，默认支持英文和简体中文，也可以通过 `GuideLabels` 完整覆盖。
