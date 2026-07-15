---
title: Scatter 散点图
---

# Scatter 散点图

Scatter 展示两个或更多连续变量的关系。每个点使用 `DataPoint::values` 保存 x、y 及可选额外维度。

```rust
let series = Series::scatter("样本", [
    DataPoint::values([12.0, 18.0]),
    DataPoint::values([20.0, 25.0]),
]);
```

构造签名：`Series::scatter(name, impl IntoIterator<Item = DataPoint>)`。常见数据形状是 `[x, y]`；额外维度可交给 VisualMap 映射 symbol size/color。

Scatter 是 `appendData` 支持的增量 series。大数据减少 symbol、label 和透明叠加开销，并通过 dataZoom/采样限制当前视口。
