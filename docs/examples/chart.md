# chart

路径：`examples/chart`

chart 示例展示 `arkit_chart` 的原生图表能力。图表由 ArkUI `Custom` 节点和 drawing canvas 绘制，不使用 WebView，也不执行 ECharts/JavaScript runtime。

示例包含四个入口：

- typed option：用 Rust 强类型 builder 组装 `ChartOption`。
- JSON option：解析 ECharts-like JSON option。
- series gallery：覆盖 line、bar、pie、scatter、radar、gauge、funnel、heatmap、candlestick、tree、treemap、graph、sankey、map 的最小可视案例。
- tooltip hit-test：点击图表后通过 `on_select` 返回 `ChartEvent`。

构建：

```sh
cd examples/chart
ohrs build --arch aarch
```

`map` series 需要用户传入 GeoJSON/feature polygon 数据；框架不内置地图资源。JSON `custom` series 会被解析为 unsupported diagnostic，只有 Rust typed API 支持自定义 draw callback。
