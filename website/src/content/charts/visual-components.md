---
title: 视觉与交互组件
description: "图例、提示、缩放和 VisualMap：给图表补说明和筛选。"
---

# 视觉与交互组件

图例、提示、缩放、VisualMap 这些负责说明和筛选；series 才负责画数据本身。

## 三态样式

| 状态     | 用途                |
| -------- | ------------------- |
| normal   | 默认展示            |
| emphasis | pointer/action 高亮 |
| blur     | 其他数据弱化        |
| select   | 持久选择            |

`ItemStyle`、`LineStyle`、`LabelStyle`、`VisualStyle` 提供 typed 样式。主题色应从应用设计 token 构造；缺失值要有独立语义，不要误映射为 0。

移动端 Tooltip 不能只依赖 hover。DataZoom/Brush 改变的是 resolved runtime state，后续坐标查询和导出均以当前状态为准。Label formatter 与 `LabelLayoutCallback` 位于 layout 热路径，应保持纯计算和低复杂度。
