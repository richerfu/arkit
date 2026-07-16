---
title: Boxplot 箱线图
description: "五数摘要和异常点。"
---

# Boxplot 箱线图

Boxplot 用五数摘要比较数据分布。

```rust
let series = Series::boxplot("延迟", [
    DataPoint::values([12.0, 18.0, 24.0, 31.0, 48.0]),
    DataPoint::values([10.0, 16.0, 22.0, 29.0, 45.0]),
]);
```

构造签名：`Series::boxplot(name, data)`。每个点按 `[min, Q1, median, Q3, max]` 提供。

原始样本应先在业务层计算摘要；异常值用额外 Scatter series 展示。不同组必须采用同一四分位数算法，并在说明中明确 whisker 的口径。
