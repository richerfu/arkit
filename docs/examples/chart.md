# chart

路径：`examples/chart`

`ECharts` 是受 Dioxus 控制的原生组件。它接收 `ChartOption` prop；父组件读取 `Signal` 并生成新 option 后，组件复用已有 ArkUI `Custom` 节点，只替换绘制快照并标记该节点重绘，不创建第二套状态/update runtime，也不依赖 WebView 或 JavaScript。

## 实现分层

图表不是一个包含所有分支的巨型 renderer：

- `model.rs` 只拥有 option、series 与 data 类型，`parser.rs` 只处理 ECharts-like JSON。
- `render/surface.rs`、`geometry.rs`、`scale.rs`、`layout.rs`、`style.rs`、`hit.rs` 提供 canvas、scale、布局、样式和命中等原子能力。
- `cartesian.rs` 与 `chrome.rs` 组合多 grid/多坐标轴、legend、visualMap、tooltip、axisPointer 与 dataZoom slider 等共享结构。
- `viewport.rs` 统一维护 dataZoom 窗口、slider/inside 手势和坐标轴窗口；`marker.rs` 统一组合 markPoint、markLine、markArea，series renderer 不重复实现这些能力。
- `render/series/` 下每种图表各自拥有 renderer，通过 `CartesianRenderContext` 或 `FreeRenderContext` 消费共享能力；series 不自行推导全局坐标域。
- `engine.rs` 只做布局与 series 分发，不包含任何具体图表绘制算法。

```rust
let mut tick = use_signal(|| 0_u32);
let option = ChartOption::new()
    .title(format!("Realtime #{tick}"))
    .x_axis(Axis::category(["Mon", "Tue", "Wed"]))
    .push_series(Series::line("Revenue", [12.0, 18.0, 15.0 + tick() as f64]))
    .push_series(Series::bar("Orders", [8.0, 11.0, 14.0]));

rsx! {
    ECharts {
        option,
        height: 320.0,
        on_select: move |event: ChartEvent| {
            // event 包含 series/data index、名称、数值和命中位置
        },
    }
}
```

当前 typed API 和 JSON parser 覆盖 ECharts core 的 22 种 series：

- line、bar、scatter、effectScatter、pictorialBar
- pie、radar、gauge、funnel
- heatmap、candlestick、boxplot
- tree、treemap、sunburst、graph、sankey
- map、lines、parallel、themeRiver
- custom（仅 typed Rust API，可传原生绘制回调）

`ChartOption::from_json_str` 支持 ECharts option 的 `title`、`legend`、多 `grid`、`tooltip/axisPointer`、多 `xAxis/yAxis`、`radar`、`dataset/encode`、`visualMap`、`dataZoom`、`color`、series common style 与各 series 布局字段。笛卡尔 series 还支持 `markPoint`、`markLine`、`markArea`。`register_map` / `register_map_str` 对应 `echarts.registerMap`；未知字段保留在 `extra`，不会在 parser 中静默丢失。

交互直接注册 ArkUI `TouchEvent`，在 native hit region 上完成 item/axis tooltip、axisPointer、selection 回调、legend 显隐、slider handle/window 拖动和 inside 平移。dataZoom 会同步作用到共享坐标域、series、marker、坐标轴刻度和命中区域。option prop 变化只更新同一个 `Custom` canvas 节点，因此 signal 驱动的实时数据不会重建视图或启动 JavaScript runtime。

下面的 JSON 可同时验证 slider、inside、cross axisPointer 与三类 marker：

```json
{
  "tooltip": { "trigger": "axis", "axisPointer": { "type": "cross", "snap": true } },
  "dataZoom": [
    { "type": "slider", "startValue": 1, "endValue": 4 },
    { "type": "inside", "start": 0, "end": 100 }
  ],
  "series": [{
    "type": "line",
    "data": [18, 24, 22, 31, 38, 35],
    "markPoint": { "data": [{ "type": "max" }, { "type": "min" }] },
    "markLine": { "data": [{ "type": "average" }] },
    "markArea": { "data": [[{ "xAxis": 1 }, { "xAxis": 3 }]] }
  }]
}
```

构建：

```sh
cd examples/chart
ohrs build --arch aarch
```
