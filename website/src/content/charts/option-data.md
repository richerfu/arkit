---
title: Option 与数据
description: "用类型化 builder 或 JSON 描述数据和 Option。"
---

# Option 与数据

`ChartOption` 是图表的 raw 受控输入。可以用类型化 builder 拼，也可以解析 ECharts 风格 JSON；两条路径都保留输入语义，不在 parser 中偷偷写入 dataset 派生的 series data。派生结果只存在于 chart runtime 和只读 `ChartSnapshot`。

## Typed Builder

```rust
let option = ChartOption::new()
    .grid(Grid::default())
    .legend(Legend::default())
    .series([line, bar]);
```

常规顶层组都有对应的 builder 方法：`title`、`legend`、`grid`、`x_axis` / `y_axis`、`radar`、`tooltip`、`data_zoom`、`dataset`、`visual_map`、`timeline`、`brush`、`animation`、`visual_style`。多坐标系图表用 `push_grid` / `push_x_axis` / `push_y_axis` / `push_series` 追加；类型化 model 尚未覆盖的 ECharts 字段走 `extra(key, value)`，不要为此改动公开 model 字段。Composite media recipe 例外，由 JSON parser 构造。

更新时构造下一份 option 并传回组件。不要直接修改 renderer 内部 series 状态。

## JSON

```rust
let option = ChartOption::from_json_str(json)?;
// 或 ChartOption::from_json_value(value)?
```

parser 返回 raw `ChartOption` 或 `ChartParseError`。不支持或非法字段可能产生 `Diagnostic`/extra；开发阶段应检查诊断，不要以“能画出来”代替配置校验。

## 数据值

`DataPoint` 覆盖常见形状：

- `scalar`：单数值。
- `named`：名称 + 数值，常用于 pie/funnel/map。
- `values`：二维或多维数据。

`DataValue` 保留 number、string 和 missing 等类型。不要提前把所有数值转字符串，否则坐标、排序和 visualMap 无法保持数值语义。

## Dataset

`Dataset` 适合多个 series 共享表格数据，再由 series encode 映射维度。独立小 series 直接传 data 更清楚；不要同时维护 dataset 与手工复制后的 series data。

dataset 驱动的 series 在 raw option 中保持空 data。更新同一个 parsed/typed option 的 dataset 后重新传给 `ECharts`，runtime 会从新 dataset 重新派生，不会复用上一版 series。显式非空 series data 始终优先，不会被 dataset 覆盖。

## Option 组件

顶层可包含 title、grid、legend、tooltip、axis、dataZoom、visualMap、brush、timeline、media、dataset、graphic 和 series。每个章节只解释其职责，最终仍组合成一个 ChartOption。`MediaOptions` 是 JSON parser 持有的 composite recipe，只提供只读 accessor；要构造或更新 `baseOption/options/media`，修改 JSON source 后重新调用 `from_json_*`。

## 受控更新

Signal 中保存领域数据或 option 均可。领域数据需要给列表和其他 UI 复用时保存领域模型，再 `use_memo` 派生 option；只服务图表时直接保存 ChartOption。

## 解析错误

加载远程 JSON 时把 parse error 渲染成页面错误，并保留配置来源信息。生产环境不要对不可信 JSON 注册任意 custom renderer callback。
