---
title: Tree 树图
---

# Tree 树图

Tree 展示有根层级。当前 typed constructor 使用 `NodeData` 与按索引连接的 `LinkData`。

```rust
let nodes: Vec<NodeData> = build_tree_nodes();
let links: Vec<LinkData> = build_parent_child_links();
let series = Series::tree("组织结构", nodes, links);
```

| 数据       | 关键字段                                                    |
| ---------- | ----------------------------------------------------------- |
| `NodeData` | `name`、`value`、可选 x/y/category/symbol、item/label style |
| `LinkData` | `source: usize`、`target: usize`、`value`                   |

构造签名：`Series::tree(name, Vec<NodeData>, Vec<LinkData>)`，默认显示节点名称。source/target 是 nodes 索引，重排节点时必须同步链接。

深树应默认折叠非关键层级并控制 label 重叠。若展开状态需要跨刷新保留，应同步到受控 option。
