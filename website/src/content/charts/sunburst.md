---
title: Sunburst 旭日图
description: "同心层级数据。"
---

# Sunburst 旭日图

Sunburst 用同心环表示树的深度和分支占比，typed 数据由递归 `SunburstNode` 组成。

```rust
let data = vec![SunburstNode {
    name: "前端".into(),
    value: 40.0,
    children: vec![],
    item_style: ItemStyle::default(),
}];
let series = Series::sunburst("技术栈", data);
```

`SunburstNode` 字段为 `name`、`value`、`children` 和 `item_style`。构造签名：`Series::sunburst(name, Vec<SunburstNode>)`，默认显示节点名称。

适合有限深度的层级占比，不适合读取精确值。分支点击、root 切换和选择通过 event/action 明确处理；稳定名称/层级有助于更新匹配。
