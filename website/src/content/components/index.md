---
title: Index
description: "字母索引栏 + 分组列表。侧栏不虚拟化；列表默认走 ArkUI List 虚拟渲染。"
---

# Index

联系人、城市选择这类「按字母分组的长列表」。右侧是索引轨，点按或拖动跳到对应分组；中间弹出当前键。

默认分组：

| 键      | 内容                     |
| ------- | ------------------------ |
| `#`     | 数字、标点、汉字等非字母 |
| `A`–`Z` | 拉丁字母（大小写不敏感） |

`index` 留空时按 `title` 归入上表。显式 `"VIP"` 这类多字符键会自成一组。

## 要不要虚拟列表？

| 部分     | 数量           | 做法                                     |
| -------- | -------------- | ---------------------------------------- |
| 索引轨   | 通常 ≤ 30 个键 | 普通 column，不虚拟化                    |
| 内容列表 | 几十到上万行   | **必须虚拟化**：`List` + `VirtualSource` |

侧栏只有字母/符号，创建成本可忽略。真正贵的是每一行。`Index` 的内容区默认就是虚拟 List。

## 用法

```rust
use arkit::shadcn::components::{Index, IndexItemSpec};

Index {
    items: vec![
        IndexItemSpec::new("#", "*客服"),
        IndexItemSpec::new("", "12306").with_description("铁路"), // → #
        IndexItemSpec::new("B", "北京"),
    ],
    show_empty_indexes: false, // 隐藏没有数据的 A–Z
    on_select: move |item_index| {},
    on_index_change: move |letter: String| {},
}
```

未传 `indexes` 时轨的底稿是 `#`、`A`–`Z`。`show_empty_indexes`（默认 `false`）控制底稿里没有行的键要不要画在侧栏上。有数据但不在底稿里的键会追加到轨尾。

## 自定义渲染

行、组头、侧栏格子都可以换成自己的 RSX。用 `use_callback` 建 `Callback`：

```rust
let render_item = use_callback(|ctx: IndexItemContext| {
    rsx! { text { "{ctx.item.title}" } }
});
let render_header = use_callback(|ctx: IndexHeaderContext| {
    rsx! { text { "{ctx.index}" } }
});
let render_bar = use_callback(|slot: IndexBarSlot| {
    rsx! { text { "{slot.index}" } } // slot.empty 表示该组没有行
});

Index {
    items,
    render_item: Some(render_item),
    render_header: Some(render_header),
    render_bar: Some(render_bar),
}
```

侧栏拖动仍由 `IndexBar` 自己算，自定义格子请 `hit_test_behavior: "none"`。只要轨、列表自己管时也可以单独用 `IndexBar`。

## Props

### Index

| Prop                 | 类型                                            | 默认          | 说明                          |
| -------------------- | ----------------------------------------------- | ------------- | ----------------------------- |
| `items`              | `Vec<IndexItemSpec>`                            | —             | 列表数据                      |
| `indexes`            | `Option<Vec<String>>`                           | `#` / `A`–`Z` | 轨的底稿                      |
| `show_empty_indexes` | `bool`                                          | `false`       | `true` 时画出没有行的分组     |
| `render_item`        | `Option<Callback<IndexItemContext, Element>>`   | 默认标题+描述 | 自定义行                      |
| `render_header`      | `Option<Callback<IndexHeaderContext, Element>>` | 默认分组条    | 自定义组头                    |
| `render_bar`         | `Option<Callback<IndexBarSlot, Element>>`       | 默认字母      | 自定义侧栏格子                |
| `on_select`          | `EventHandler<usize>`                           | —             | 点中一行，参数是 `items` 下标 |
| `on_index_change`    | `EventHandler<String>`                          | —             | 当前组键                      |
| `width` / `height`   | `String`                                        | `"100%"`      | 虚拟 List 需要明确高度        |

### IndexBarSlot

| 字段     | 说明                                      |
| -------- | ----------------------------------------- |
| `index`  | 组键                                      |
| `active` | 是否当前组                                |
| `empty`  | 该键没有列表行（仅在展示空分组时为 true） |
