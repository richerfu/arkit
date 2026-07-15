---
title: Chart
---

# Chart

shadcn Chart 是轻量百分比展示组件，按 `chart_1` 到 `chart_5` 主题色绘制 series progress rows；ChartCard 再增加标题和卡片 surface。

```rust
ChartCard {
    title: "任务完成率",
    values: vec![72.0, 45.0, 91.0],
}
```

| 组件        | 属性                                | 说明                        |
| ----------- | ----------------------------------- | --------------------------- |
| `Chart`     | `values: Vec<f32>`                  | 每个值按 0–100 clamp 后展示 |
| `ChartCard` | `title: String`、`values: Vec<f32>` | 标题化的 Chart 卡片         |

它不等同于 `arkit::echarts::ECharts`：

| 需求                                    | 选择           |
| --------------------------------------- | -------------- |
| 卡片内简单百分比、主题统一              | shadcn `Chart` |
| 22 类 series、坐标系、tooltip、dataZoom | `ECharts`      |
| Action、appendData、图片导出            | `ECharts`      |

需要完整图表时启用 `chart` feature，并让业务只维护一份 `ChartOption` 数据模型。
