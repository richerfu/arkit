---
title: Treemap 矩形树图
---

# Treemap 矩形树图

Treemap 用矩形面积编码分类或层级数值。

```rust
let series = Series::treemap("目录占用", [
    DataPoint::named("target", 420.0),
    DataPoint::named("src", 96.0),
    DataPoint::named("assets", 64.0),
]);
```

构造签名：`Series::treemap(name, impl IntoIterator<Item = DataPoint>)`，默认显示名称。平面 named points 可直接使用；完整层级配置可由 ECharts JSON 解析进入同一模型。

parent value 与 children 汇总规则必须一致。小矩形自动隐藏 label，完整路径和值放入 Tooltip；需要精确排序比较时使用 Bar/Table。
