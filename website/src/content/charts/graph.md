---
title: Graph 关系图
description: "节点、连线和布局。"
---

# Graph 关系图

Graph 展示任意节点和连接关系，typed constructor 使用 nodes 与按索引引用的 links。

```rust
let nodes: Vec<NodeData> = build_graph_nodes();
let links: Vec<LinkData> = build_graph_links();
let series = Series::graph("依赖关系", nodes, links);
```

`NodeData` 包含 name/value、可选固定坐标、category、symbol 和样式；`LinkData` 使用 `source`、`target` 索引与 `value`。

构造签名：`Series::graph(name, Vec<NodeData>, Vec<LinkData>)`。节点 identity 必须稳定，source/target 不得越界。大关系网应先聚合、筛选或限制邻居深度，避免把布局和 hit-test 压力全部交给 renderer。
