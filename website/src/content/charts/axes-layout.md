---
title: 坐标轴与布局
description: "坐标轴、网格和绘图区怎么摆，坐标如何换算。"
---

# 坐标轴与布局

直角坐标图由坐标轴、网格和 series 一起定调。轴可以是类目、数值、时间或对数，位置决定它是横轴还是纵轴。

## Axis

`Axis` 包含 axis line、tick、label 与 pointer 配置。category 轴保持标签顺序；time 轴保留时间数值语义；log 轴的数据必须满足其定义域。

多个 y 轴只有在单位和尺度清楚标注时使用。series 通过索引关联相应 grid/axis；动态增删坐标系时保持索引映射一致。

## Grid

Grid 定义绘图区的 left/top/right/bottom 与 containment。标题、图例和长 axis label 需要预留边距，不能依赖裁剪后仍可读。

## 响应式布局

`media` 可按 canvas 条件合并 option 片段。窗口变化后图表会重新布局；业务层不缓存像素坐标。坐标转换、`containPixel` 与 hit-test 都基于当前 resolved viewport。
