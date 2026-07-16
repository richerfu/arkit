---
title: Gauge 仪表盘
description: "单值、区间和进度。"
---

# Gauge 仪表盘

Gauge 展示某个区间中的当前单值。

```rust
let series = Series::gauge("CPU", 72.0);
let option = ChartOption::new().push_series(series);
```

构造签名：`Series::gauge(name, value: f64)`。min/max、axis line、split、pointer、progress、detail 和 formatter 通过 series options/JSON 配置。

value 必须与配置区间一致，颜色阈值要有业务含义。Gauge 表达“当前状态”，时间趋势仍应使用 Line；高频刷新使用受控 option 与较短更新动画。
