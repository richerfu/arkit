---
title: Line 折线图
description: "连续趋势、平滑、区域和标记。"
---

# Line 折线图

Line 用于连续趋势或按顺序连接的数据。构造器接收一维数值，横坐标通常由 category/time Axis 提供。

```rust
let series = Series::line("访问量", [12.0, 18.0, 15.0]);
let option = ChartOption::new()
    .x_axis(Axis::category(["一", "二", "三"]))
    .y_axis(Axis::value())
    .push_series(series);
```

构造签名：`Series::line(name, impl IntoIterator<Item = f64>)`。默认 symbol 为 `emptyCircle`、大小 6；线条、area、smooth、stack、label 和三态样式通过 series options/JSON 配置。

时间数据保持排序。缺失点使用 missing/null，不用 0 伪造；多条线共享轴时单位必须一致，数量过多时配合 Legend 与 DataZoom。
