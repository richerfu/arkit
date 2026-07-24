---
title: Option 与数据
description: "用类型化 builder 或 JSON 描述数据和 Option。"
---

# Option 与数据

`ChartOption` 是图表的受控输入。可以用类型化 builder 拼，也可以解析 ECharts 风格 JSON，最后进的是同一套 model。

## Typed Builder

```rust
let option = ChartOption::new()
    .grid(Grid::default())
    .legend(Legend::default())
    .series([line, bar]);
```

更新时构造下一份 option 并传回组件。不要直接修改 renderer 内部 series 状态。

## JSON

```rust
let option = ChartOption::from_json_str(json)?;
// 或 ChartOption::from_json_value(value)?
```

parser 返回 `ChartParseError`。不支持或非法字段可能产生 `Diagnostic`/extra；开发阶段应检查诊断，不要以“能画出来”代替配置校验。

## 数据值

`DataPoint` 覆盖常见形状：

- `scalar`：单数值。
- `named`：名称 + 数值，常用于 pie/funnel/map。
- `values`：二维或多维数据。

`DataValue` 保留 number、string 和 missing 等类型。不要提前把所有数值转字符串，否则坐标、排序和 visualMap 无法保持数值语义。

## Dataset

`Dataset` 适合多个 series 共享表格数据，再由 series encode 映射维度。独立小 series 直接传 data 更清楚；不要同时维护 dataset 与手工复制后的 series data。

## Option 组件

顶层可包含 title、grid、legend、tooltip、axis、dataZoom、visualMap、brush、timeline、media、dataset、graphic 和 series。每个章节只解释其职责，最终仍组合成一个 ChartOption。

## 受控更新

Signal 中保存领域数据或 option 均可。领域数据需要给列表和其他 UI 复用时保存领域模型，再 `use_memo` 派生 option；只服务图表时直接保存 ChartOption。

## 解析错误

加载远程 JSON 时把 parse error 渲染成页面错误，并保留配置来源信息。生产环境不要对不可信 JSON 注册任意 custom renderer callback。
