---
title: Lines 路径图
description: "路径、迁徙和增量数据。"
---

# Lines 路径图

Lines 展示起终点或完整坐标路径，适合迁徙、航线与网络连接。

```rust
let route = LineSegment {
    name: Some("北京 → 上海".into()),
    from: (116.40, 39.90),
    to: (121.47, 31.23),
    coords: vec![(116.40, 39.90), (121.47, 31.23)],
    value: 120.0,
};
let series = Series::lines("航线", vec![route]);
```

`LineSegment` 字段为 `name`、`from`、`to`、`coords` 和 `value`；coords 可保存完整路径，from/to 保持 typed API 兼容。Lines 是 `appendData` 支持的另一类增量 series。

大量动态路径应降低 effect、symbol 和透明叠加成本，并按视口或时间窗口裁剪。坐标系、投影与数据单位必须一致。
