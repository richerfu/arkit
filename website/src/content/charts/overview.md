---
title: 图表总览
---

# 图表总览

启用 `chart` 后，`arkit::echarts` 提供 ECharts-compatible typed/JSON model，以及 ArkUI Custom + Drawing 原生 renderer。它不嵌 WebView；`chart` 自动启用 root `animation`。

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
        percent_width: 1.0,
        height: 320.0,
    }
}
```

未指定高度时默认 320vp。parent 仍需提供可确定宽度。

## ECharts Props

| Prop                               | 说明                                 |
| ---------------------------------- | ------------------------------------ |
| `option`                           | 完整受控 `ChartOption`               |
| `width` / `height`                 | 固定 vp                              |
| `percent_width` / `percent_height` | 相对尺寸                             |
| `on_select`                        | point/bar/sector/node/region 选择    |
| `on_event`                         | pointer 与 component action 统一事件 |
| `controller`                       | imperative action/query handle       |

option 变化进入同一 model/layout/render path；Controller 适合命令和查询，不替代受控 option。

## 渲染流程

```text
ChartOption → normalize/model → layout → display list → Drawing canvas
                                              ↘ hit-test cache
```

tooltip、selection、坐标转换和导出使用实际 resolved model/绘制结果，不维护一份与画面分离的 Web chart。

## 文档路径

先读 Option 与数据，再按 series 家族选择章节。坐标/样式、Action/Event、实时更新和导出各自独立，避免在一个巨大配置页中查找所有能力。
