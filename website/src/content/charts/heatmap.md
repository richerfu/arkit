---
title: Heatmap 热力图
---

# Heatmap 热力图

Heatmap 用颜色表达二维位置上的第三个数值，通常与 VisualMap 一起使用。

```rust
let series = Series::heatmap("活跃度", [
    DataPoint::values([0.0, 0.0, 12.0]),
    DataPoint::values([1.0, 0.0, 28.0]),
]);
```

构造签名：`Series::heatmap(name, impl IntoIterator<Item = DataPoint>)`。标准点形状为 `[x, y, value]`；category×category 场景中的 x/y 是对应轴索引。

缺失单元保留 missing/null，不填 0。VisualMap 的范围、单位和颜色方向必须清楚说明；网格过密时关闭 cell label，详情交给 Tooltip。
