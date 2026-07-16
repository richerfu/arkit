---
title: Map 地图
description: "GeoJSON、区域数据与注册。"
---

# Map 地图

Map 在多边形区域上编码数值。可直接构造 `MapFeature`，也可先注册 GeoJSON。

```rust
register_map_str("region", geojson)?;

let feature = MapFeature::new("中心区", polygons).with_value(42.0);
let series = Series::map("区域数据", vec![feature]);

// 不再使用时
unregister_map("region");
```

`MapFeature` 包含 name、可选 value、polygons、center、normal/emphasis/select 样式、properties 与 selected 状态。`None` 是无数据，不参与 VisualMap。

构造签名：`Series::map(name, Vec<MapFeature>)`。feature name 必须与数据/GeoJSON name property 对齐。大型 GeoJSON 在解析前限制大小与层级，并统一管理注册名和生命周期。
