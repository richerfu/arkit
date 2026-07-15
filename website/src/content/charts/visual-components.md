---
title: 视觉与交互组件
---

# 视觉与交互组件

Option component 负责说明、筛选和视觉映射；series 负责具体数据标记。

| 组件           | 职责                                |
| -------------- | ----------------------------------- |
| `Title`        | 说明图表目的                        |
| `Legend`       | 展示并切换 series/category          |
| `Tooltip`      | 展示当前命中数据                    |
| `VisualMap`    | 把数值区间映射到颜色、symbol 等视觉 |
| `DataZoom`     | slider/inside 视口缩放              |
| `BrushOptions` | 矩形、线等区域选择                  |
| `Timeline`     | 驱动多个 option 数据状态            |
| `MediaOptions` | 按 canvas 条件应用响应式 option     |

## 三态样式

| 状态     | 用途                |
| -------- | ------------------- |
| normal   | 默认展示            |
| emphasis | pointer/action 高亮 |
| blur     | 其他数据弱化        |
| select   | 持久选择            |

`ItemStyle`、`LineStyle`、`LabelStyle`、`VisualStyle` 提供 typed 样式。主题色应从应用设计 token 构造；缺失值要有独立语义，不要误映射为 0。

移动端 Tooltip 不能只依赖 hover。DataZoom/Brush 改变的是 resolved runtime state，后续坐标查询和导出均以当前状态为准。Label formatter 与 `LabelLayoutCallback` 位于 layout 热路径，应保持纯计算和低复杂度。
