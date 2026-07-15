---
title: Radar 雷达图
---

# Radar 雷达图

Radar 在多个 indicator 上展示一个观测的多维轮廓。

```rust
let series = Series::radar("当前方案", [82.0, 68.0, 91.0, 74.0]);
```

构造签名：`Series::radar(name, impl IntoIterator<Item = f64>)`。构造器会创建一个多维 DataPoint；`ChartOption.radar` 需要提供相同顺序和数量的 indicator。

不同量纲应先归一化，或为每个 indicator 设置真实业务上界。Radar 适合看轮廓，不适合读取精确值；需要精确比较时配合 Table/Bar。
