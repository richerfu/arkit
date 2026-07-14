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

`ChartOption::from_json_str` 支持 ECharts option 的 `title`、`legend`、多 `grid`、`tooltip/axisPointer`、多 `xAxis/yAxis`、`radar`、`dataset/encode`、`visualMap`、`dataZoom`、`color`、series common style 与各 series 布局字段。笛卡尔坐标轴支持 `position/offset`、`axisLine`、`axisTick`、`axisLabel`（颜色、字号、旋转、间距、interval 与字符串 formatter）和 `splitLine` 样式；笛卡尔 series 还支持 `markPoint`、`markLine`、`markArea`。`register_map` / `register_map_str` 对应 `echarts.registerMap`；未知字段保留在 `extra`，不会在 parser 中静默丢失。

legend 支持 `selected/selectedMode` 的初始显隐、`single/multiple/false` 原生点击语义，支持字符串 formatter、全局和 data item 级 icon、`inactiveColor/itemGap/align/orient`；横向 plain legend 会在 Canvas 可用宽度内自动换行，避免条目或点击区域越过图表边界。restore 会恢复 option 声明的初始 legend 选择状态。

响应式 option 支持 ECharts 的 `baseOption + media`：`media.query` 可使用 `minWidth/maxWidth/minHeight/maxHeight/minAspectRatio/maxAspectRatio`，同一尺寸命中的多条规则按声明顺序合并，后声明规则优先；没有 query 命中时使用首条无 query 的 default media。尺寸变化会在同一个原生 Canvas 节点上重新解析布局，并与当前 timeline frame、toolbox 运行时状态一起保留。

动画运行时遵循 ECharts 的全局首次、更新和状态动画配置：支持 `animation/animationThreshold`、duration/easing/delay 及对应 Update 字段、`stateAnimation`，series 和 data item 按 id/name/index 匹配并对数值、坐标、颜色、透明度、线宽、symbol、label 和节点/连线几何执行原生补间。通用 `emphasis/blur/select` 支持 `self/series/adjacency/ancestor/descendant` focus、`series/coordinateSystem/global` blurScope、scale 与 itemStyle/lineStyle/label；未声明 blur opacity 时使用 ECharts 的 0.1 默认值，selectedMode 已覆盖所有声明该能力的 series。`labelLayout.hideOverlap/moveOverlap` 使用 Canvas 级共享标签占用表处理跨 series 重叠，支持 shiftX/shiftY/shuffleX/shuffleY。

`ChartController::dispatch_action` 提供原生 `dispatchAction` 入口，覆盖 highlight/downplay、select/unselect/toggleSelect、showTip/hideTip、legend select/unselect/toggle、dataZoom、timeline change/play 与 restore；`dispatch_actions` 对应 ECharts batch，一次完成全部状态变更并发出一个带 `batch` 明细的聚合事件。组件通过 `on_event` 统一发出 mousedown/mouseup/mouseover/mouseout/mousemove/globalout/click、selectchanged、legendselectchanged、datazoom 和 timeline 事件，`silent` action 只改变状态而不发事件。`labelLayout` 同时支持 x/y（像素或百分比）、dx/dy、rotate、align/verticalAlign、width/height/fontSize、hideOverlap、shiftX/shiftY/shuffleX/shuffleY 和可拖拽偏移；typed API 可用 `LabelLayoutOptions::with_callback` 返回逐标签布局结果，回调收到真实 `seriesIndex/dataIndex/labelLinePoints`，饼图回调返回的引导线点会直接参与原生绘制。拖拽偏移属于单个图表实例，配置一致的受控 option 更新会保留，restore 会复位。

实例 API 还提供 `append_data`、`get_option/get_size/get_width/get_height`、`convert_to_pixel/convert_from_pixel/contain_pixel` 与 `clear`。`append_data` 遵循 ECharts 增量渲染的 series 限制，使用 `ChartAppendData::scatter` 或 `ChartAppendData::lines`；dataset 驱动的 scatter 不接受增量追加。`get_option` 返回已经合并运行时 legend 显隐、dataZoom 窗口和 data item 选择状态的快照。坐标转换通过 `ChartCoordinateFinder::series/grid/axes` 定位笛卡尔坐标系，输入输出 `ChartCoordinatePoint`，category axis 会保留类目字符串，其余轴返回数值；转换读取当前响应式 grid、dataZoom 和实际逻辑画布尺寸。append/clear 属于实例状态，相同的受控 prop 重新渲染不会立即回滚它们；父组件真正传入不同 option 时，新的受控值仍然优先。

```rust
let controller = use_hook(ChartController::new);
controller.dispatch_action(ChartAction::new(ChartActionKind::Highlight(
    ChartActionTarget::item(0, 2),
)));
controller.append_data(ChartAppendData::scatter(
    1,
    [DataPoint::values([12.0, 28.0])],
));
let current_size = controller.get_size();
let pixel = controller.convert_to_pixel(
    ChartCoordinateFinder::series(1),
    ChartCoordinatePoint::values("Tue", 28.0),
);

rsx! {
    ECharts {
        option,
        controller: Some(controller),
        on_event: move |event: ChartRuntimeEvent| {
            // event.event_type / event.from_action / event.selected
        },
    }
}
```

坐标系分发覆盖常用原生组合：line/bar/scatter/effectScatter 可使用 polar，scatter/effectScatter 可使用 singleAxis，scatter/effectScatter/heatmap/lines/graph 可叠加在独立 geo 组件上，heatmap 可使用 calendar，map 使用注册或内联 GeoJSON；grid 继续承载所有笛卡尔 series。polar bar 使用环形扇区命中，polar/singleAxis/geo effectScatter 复用原生波纹帧循环，calendar heatmap 按日期范围生成周/星期单元格。

共享数据与组件支持多 `dataset` + `datasetIndex`、多 `visualMap` + `seriesIndex`，以及 continuous/piecewise 色彩和 symbolSize 映射。`timeline` 支持 `baseOption + options` 快照合并、节点/前后/播放控制、autoPlay/rewind/loop；`brush` 支持 rect/lineX/lineY、single/multiple 选择、原生拖拽反馈和 toolbox rect/clear。`graphic` 支持 group、rect、circle、ellipse、line、polygon、polyline、text 原生标注。toolbox 支持 restore、dataZoom 框选/历史回退、magicType line/bar/stack 切换；dataView 提供原生只读覆盖层和关闭交互，`readOnly: false` 时仍发出 action 交给宿主提供 ArkUI 编辑器。saveAsImage 使用离屏 Canvas、PixelMap 和 ImagePacker 输出 png/jpeg，默认写入 `/data/storage/el2/base/files`，并支持原生扩展字段 `path` 指定完整目标路径。restore 会复位 dataZoom、magicType、timeline、brush、dataView、legend 显隐和选择状态。任意 JavaScript formatter/renderItem 不属于原生运行时能力，typed custom renderer 是对应扩展点。

dataset 数据管线支持显式 `dimensions/sourceHeader`、array rows 与 object rows、`fromDatasetIndex/fromDatasetId`，并原生执行内置 `filter`、`sort` 和数组形式的管线 transform。filter 支持维度名/索引、比较操作符、`and/or/not` 嵌套条件和 number/trim/time parser；sort 支持多字段、升降序、parser 与 incomparable 排序。transform 结果会同时驱动 series 数据和自动类目轴，不会继续误用原始 dataset 的行顺序。

JSON 数据中的 `null` 与 `"-"` 会保留为 `DataValue::Null`，不再被错误地当作数值 `0`。line 默认在空值处断开；设置 `connectNulls: true` 后只连接两侧有效点。typed API 可使用 `DataPoint::missing()` 或 `DataValue::Null` 构造同样的数据。

line 的几何和样式语义直接对应 ECharts option：`step` 支持 `start/middle/end`，`smoothMonotone` 会约束平滑曲线的过冲，`clip` 控制 grid 裁剪，`sampling` 支持 `average/min/max/sum/lttb`，`lineStyle.type` 支持 `solid/dashed/dotted`。`symbol` 支持 circle、emptyCircle、rect、roundRect、triangle、diamond、arrow、pin、none，并支持 series/data item 两级的 `symbolSize/symbolRotate/symbolOffset`；普通 label 支持 position/distance/rotate/offset，`endLabel` 使用末个有效数据点。`coordinateSystem: "polar"` 会读取对应的 `polar/angleAxis/radiusAxis`，标量 data 按 radius + category index 投影，二维 data 按 `[radius, angle]` 投影，并复用 line 的 smooth/area/symbol/label/hit 行为。

bar 会根据 category axis 所在方向自动选择纵向或横向布局；同名 `stack` 共享一个 bar group，并分别累积正值与负值。`barWidth/barMinWidth/barMaxWidth` 接受像素或百分比，`barGap/barCategoryGap` 控制组间和类目间距，`barMinHeight` 约束 value axis 上的最小可见长度；`showBackground/backgroundStyle`、series/data item 两级 `itemStyle.borderRadius`（兼容旧字段 `barBorderRadius`）和常用 label position 直接参与原生绘制与命中区域计算。

line、scatter 与 effectScatter 共用同一套 symbol 解析和绘制：circle、emptyCircle、rect、roundRect、triangle、diamond、arrow、pin、none 均支持 series/data item 两级 `symbol/symbolSize/symbolRotate/symbolOffset`，`symbolSize` 可用标量或 `[width, height]`。scatter 会读取 `visualMap.dimension` 以及 `inRange.color/inRange.symbolSize` 生成气泡颜色和尺寸；effectScatter 由组件级帧循环驱动，读取 `rippleEffect.period/number/scale/brushType/color` 绘制与 symbol 同比例的原生动画波纹。

pictorialBar 使用同一 symbol 系统，支持横向/纵向、分组布局、`symbolRepeat/symbolMargin/symbolClip/symbolBoundingData/symbolPosition/symbolRepeatDirection`。tree/graph/sankey 保留 data item 的 symbol、symbolSize、symbolRotate、itemStyle 和 label；treemap 递归布局 `children`；lines 保留完整多点 coords，支持终点 symbol、虚线、窄路径命中和 `effect` 移动 symbol。

pie 的扇区、label、labelLine 与命中区域共用统一角度分配结果，支持 `startAngle/endAngle/clockwise/minAngle/padAngle/minShowLabelAngle/stillShowZeroSum`、普通/玫瑰图半径和部分圆布局。label 与 tooltip formatter 支持 `{d}` 百分比及 `percentPrecision`；外部 labelLine 支持长度和线型并进行基础防重叠排布。`selectedMode/selectedOffset` 与 data item `selected` 接入原生选择状态，点击扇区后对应几何会沿角平分线偏移。

map 使用 GeoJSON 的真实面结构，而不是包围矩形近似。Polygon 的首个 ring 是 exterior，其余 ring 保留为 holes；MultiPolygon 的每个 polygon 独立保存。`MapFeature::value` 为 `Option<f64>`，没有 data 的地区保持 no-data，不参与 visualMap。解析和渲染支持 `nameProperty/nameMap`、`regions`、data item/style/label、normal/emphasis/select、`selectedMode`、`boundingCoords`、`center/zoom/scaleLimit/aspectScale`、box layout 与 `layoutCenter/layoutSize`。命中测试使用 point-in-polygon 并排除 holes，点击选择不会再落到地区包围盒的空白区域；`roam: true/"move"` 允许原生触摸拖动，并在 option 重绘时保留平移量。

交互直接注册 ArkUI `TouchEvent`，在 native hit region 上完成 item/axis tooltip、axisPointer、selection 回调、legend 显隐、timeline 切帧、brush 拖选、slider handle/window 拖动和 inside 平移。dataZoom 会同步作用到共享坐标域、series、marker、坐标轴刻度和命中区域。option prop 变化只更新同一个 `Custom` canvas 节点，因此 signal 驱动的实时数据不会重建视图或启动 JavaScript runtime。

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

cd ../../app
./run.sh chart
```
