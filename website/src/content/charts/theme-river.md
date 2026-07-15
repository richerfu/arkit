---
title: Theme River 主题河流图
---

# Theme River 主题河流图

ThemeRiver 展示多个主题随时间变化的流量。每个点需要时间、数值与主题名称三个维度。

```rust
let series = Series::theme_river("话题", [
    DataPoint::values(vec![
        DataValue::String("2026-07-01".into()),
        DataValue::Number(12.0),
        DataValue::String("Rust".into()),
    ]),
    DataPoint::values(vec![
        DataValue::String("2026-07-01".into()),
        DataValue::Number(8.0),
        DataValue::String("ArkUI".into()),
    ]),
]);
```

构造签名：`Series::theme_river(name, impl IntoIterator<Item = DataPoint>)`。JSON/typed 数据应保持 `[time, value, name]` 语义；时间格式对所有主题一致。

时间和主题保留字符串，value 保留数值语义。明确处理负值和缺失点，主题过多时先聚合。
