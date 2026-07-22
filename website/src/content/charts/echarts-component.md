---
title: ECharts 组件
description: "Props、尺寸、事件与 Controller 绑定。"
---

# ECharts 组件

`ECharts` 使用 ArkUI Custom + Drawing 原生渲染，不嵌入 WebView。`ChartOption` 是受控输入；替换 option 会进入同一 normalize、layout、display-list 和绘制流程。

```rust
let option = ChartOption::new()
    .x_axis(Axis::category(["Mon", "Tue", "Wed"]))
    .y_axis(Axis::value())
    .push_series(Series::line("访问量", [12.0, 18.0, 15.0]));

ECharts {
    option,
    width: "100%",
    height: "320",
    on_select: move |event| tracing::debug!(?event),
}
```

| Prop         | 类型                                      | 默认值   | 说明                                  |
| ------------ | ----------------------------------------- | -------- | ------------------------------------- |
| `option`     | `ChartOption`                             | 必填     | 完整图表模型                          |
| `width`      | `String`                                  | `"100%"` | CSS 宽度（vp 数字字符串或 `"N%"`）    |
| `height`     | `Option<String>`                          | `"320"`  | CSS 高度；未设时使用 320vp            |
| `on_select`  | `Option<EventHandler<ChartEvent>>`        | `None`   | point、bar、sector、node、region 选择 |
| `on_event`   | `Option<EventHandler<ChartRuntimeEvent>>` | `None`   | pointer 与 component action 统一事件  |
| `controller` | `Option<ChartController>`                 | `None`   | action、查询与导出 handle             |

父容器必须提供可确定宽度。Controller 用于命令和查询，不替代受控 option；页面卸载后不要继续向旧 instance dispatch。
