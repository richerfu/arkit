---
title: Pie 饼图
---

# Pie 饼图

Pie 用扇区展示整体中的分类占比，数据使用带名称的 `DataPoint`。

```rust
let series = Series::pie("来源", [
    DataPoint::named("搜索", 42.0),
    DataPoint::named("直接访问", 28.0),
    DataPoint::named("其他", 12.0),
]);
```

构造签名：`Series::pie(name, impl IntoIterator<Item = DataPoint>)`。默认显示 `{b}` 名称 label；radius、center、rose、selected mode 和三态样式通过 series options/JSON 配置。

名称保持稳定且数值非负。小扇区过多时合并“其他”或改用 Bar；不要依赖几十种颜色让用户比较精确差异。
