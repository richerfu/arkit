---
title: Candlestick K 线图
description: "OHLC 数据和金融样式。"
---

# Candlestick K 线图

Candlestick 展示一段时间内的开盘、收盘、最低和最高值。

```rust
let series = Series::candlestick("价格", [
    DataPoint::values([20.0, 24.0, 18.0, 26.0]),
    DataPoint::values([24.0, 22.0, 21.0, 27.0]),
]);
```

构造签名：`Series::candlestick(name, data)`。每个点按 `[open, close, low, high]` 提供，并与 category/time 轴一一对应。

数据必须按时间排序，low/high 与 open/close 保持合法关系。成交量通常用第二个 Grid 中的 Bar 展示；Tooltip 要注明交易时区、币种和复权口径。
