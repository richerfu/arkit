---
title: 图表总览
description: "原生 Drawing 图表能做什么、不支持什么，建议怎么读后面的文档。"
---

# 图表总览

打开 `chart` 之后，可以通过 `arkit::echarts` 用接近 ECharts 的 Option 模型画图。渲染在 ArkUI Custom + Drawing 上完成，**不嵌 WebView**。

下面按「怎么建模 → 有哪些系列 → 交互和性能」组织，建议先读组件与 Option，再按需点进具体 series。

## 接入

```toml
[dependencies]
arkit = { version = "*", features = ["chart"] }
```

```rust
use arkit::prelude::*;
use arkit::echarts::*;
```

## 最小图表

```rust
let option = ChartOption::new()
    .title("Traffic")
    .x_axis(Axis::category(["Mon", "Tue", "Wed"]))
    .y_axis(Axis::value())
    .push_series(Series::line("Visits", [12.0, 18.0, 15.0]));

rsx! {
    ECharts {
        option,
        width: "100%",
        height: "320",
    }
}
```

未指定高度时默认 320vp。parent 仍需提供可确定宽度。

## ECharts Props

| Prop               | 说明                                 |
| ------------------ | ------------------------------------ |
| `option`           | 完整受控 `ChartOption`               |
| `width` / `height` | CSS 尺寸（vp 数字字符串或 `"N%"`）   |
| `on_select`        | point/bar/sector/node/region 选择    |
| `on_event`         | pointer 与 component action 统一事件 |
| `controller`       | imperative action/query handle       |

option 变化进入同一 model/layout/render path；Controller 适合命令和查询，不替代受控 option。

## 渲染流程

```text
ChartOption → normalize/model → layout → display list → Drawing canvas
                                              ↘ hit-test cache
```

tooltip、selection、坐标转换和导出使用实际 resolved model/绘制结果，不维护一份与画面分离的 Web chart。

## 文档路径

先读 Option 与数据，再按 series 家族选择章节。坐标/样式、Action/Event、实时更新和导出各自独立，避免在一个巨大配置页中查找所有能力。
