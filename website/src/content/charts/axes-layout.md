---
title: 坐标轴与布局
---

# 坐标轴与布局

直角坐标图由 Axis、Grid 和 series 共同定义。Axis 支持 `category`、`value`、`time` 与 `log` 类型，方向通过 x/y 轴位置确定。

```rust
let option = ChartOption::new()
    .grid(Grid::default())
    .x_axis(Axis::category(["一月", "二月", "三月"]))
    .y_axis(Axis::value())
    .push_series(Series::bar("订单", [32.0, 48.0, 41.0]));
```

## Axis

`Axis` 包含 axis line、tick、label 与 pointer 配置。category 轴保持标签顺序；time 轴保留时间数值语义；log 轴的数据必须满足其定义域。

多个 y 轴只有在单位和尺度清楚标注时使用。series 通过索引关联相应 grid/axis；动态增删坐标系时保持索引映射一致。

## Grid

Grid 定义绘图区的 left/top/right/bottom 与 containment。标题、图例和长 axis label 需要预留边距，不能依赖裁剪后仍可读。

## 响应式布局

`media` 可按 canvas 条件合并 option 片段。窗口变化后图表会重新布局；业务层不缓存像素坐标。坐标转换、`containPixel` 与 hit-test 都基于当前 resolved viewport。
