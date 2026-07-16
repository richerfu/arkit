---
title: Funnel 漏斗图
description: "阶段顺序和转化。"
---

# Funnel 漏斗图

Funnel 展示有顺序的流程阶段及其数量，数据使用 named DataPoint。

```rust
let series = Series::funnel("转化", [
    DataPoint::named("访问", 1000.0),
    DataPoint::named("注册", 420.0),
    DataPoint::named("购买", 96.0),
]);
```

构造签名：`Series::funnel(name, impl IntoIterator<Item = DataPoint>)`。默认显示阶段名称；排序、gap、min/max size、label 和 item style 由 options 控制。

阶段按业务流程排序，不应因某次数据波动自动换序。Tooltip/label 中明确转化率分母、时间范围和去重口径。
