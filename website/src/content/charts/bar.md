---
title: Bar 柱状图
description: "离散比较、堆叠和柱宽。"
---

# Bar 柱状图

Bar 用柱形比较离散类别，也可通过 stack 表示组成。

```rust
let series = Series::bar("订单", [4.0, 7.0, 5.0]);
let option = ChartOption::new()
    .x_axis(Axis::category(["一", "二", "三"]))
    .y_axis(Axis::value())
    .push_series(series);
```

构造签名：`Series::bar(name, impl IntoIterator<Item = f64>)`。bar width/gap、stack、background、label 与 item style 使用 `SeriesOptions` 或 JSON option 设置。

柱长应从可解释的基线开始；截断 value 轴必须明确提示。类别很多时使用 DataZoom 或横向排列，不把标签缩到不可读。
