---
title: Effect Scatter 涟漪散点图
description: "重点点位和涟漪效果。"
---

# Effect Scatter 涟漪散点图

EffectScatter 在散点上增加涟漪强调，适合标记少量异常点、告警位置或当前焦点。

```rust
let series = Series::effect_scatter("告警", [
    DataPoint::values([12.0, 18.0]),
    DataPoint::values([20.0, 25.0]),
]);
```

构造签名与 Scatter 相同：`Series::effect_scatter(name, data)`，点通常为 `[x, y]`。坐标轴、VisualMap 与 tooltip 的使用方式也相同。

不要给所有数据点都加 effect；持续动画会增加绘制和电量成本。常用组合是普通 Scatter 展示全集，EffectScatter 只展示被强调的子集。
