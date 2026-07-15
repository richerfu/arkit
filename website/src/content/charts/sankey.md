---
title: Sankey 桑基图
---

# Sankey 桑基图

Sankey 用连接宽度表达节点之间的流量。

```rust
let nodes: Vec<NodeData> = build_flow_nodes();
let links: Vec<LinkData> = build_flow_links();
let series = Series::sankey("流量去向", nodes, links);
```

构造签名：`Series::sankey(name, Vec<NodeData>, Vec<LinkData>)`。节点复用 `NodeData`，链接的 `value` 是流量值，`source`/`target` 引用 nodes 索引。

每层总流量口径必须一致；循环、负值和无法解释的损耗应在建模前处理。节点 label 展示名称，link Tooltip 说明来源、去向、数值和单位。
