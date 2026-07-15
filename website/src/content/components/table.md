---
title: Table
---

# Table

Table 展示字符串行列数据，自动应用主题 header、分隔线、边框和圆角。

```rust
Table {
    headers: vec!["名称".into(), "状态".into()],
    rows: vec![
        vec!["编译".into(), "通过".into()],
        vec!["测试".into(), "运行中".into()],
    ],
}
```

| 属性      | 类型               | 说明           |
| --------- | ------------------ | -------------- |
| `headers` | `Vec<String>`      | 表头文本       |
| `rows`    | `Vec<Vec<String>>` | 二维单元格文本 |

每行单元格数量应与 header 数量一致。当前 Table 不提供排序、选择、固定列和虚拟化；数百行以上应分页，复杂单元格或超大数据使用基础 `list/grid` 构建专用表格。
