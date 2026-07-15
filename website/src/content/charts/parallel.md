---
title: Parallel 平行坐标图
---

# Parallel 平行坐标图

Parallel 用多条平行轴展示多维观测，每个 DataPoint 的 values 顺序必须与 parallel axes 一致。

```rust
let series = Series::parallel("设备", [
    DataPoint::values([82.0, 43.0, 120.0, 0.8]),
    DataPoint::values([68.0, 51.0, 95.0, 0.6]),
]);
```

构造签名：`Series::parallel(name, impl IntoIterator<Item = DataPoint>)`。每个点是一组多维数值，维度数量应保持一致。

不同量纲要归一化或清楚展示各轴范围。大量折线使用较低 opacity 和 Brush 筛选，否则交叉遮挡会让图失去可读性。
