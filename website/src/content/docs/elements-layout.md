---
title: 元素与布局
---

# 元素与布局

`arkit::prelude::*` re-export ArkUI element registry。RSX tag 会创建对应原生节点，不经过 HTML 或 WebView。

## 27 个元素

| 分类     | RSX tag                                           |
| -------- | ------------------------------------------------- |
| 布局     | `column`、`row`、`stack`、`flex`                  |
| 绘制     | `custom`                                          |
| 内容     | `text`、`image`                                   |
| 输入     | `button`、`checkbox`、`toggle`、`radio`、`slider` |
| 状态     | `progress`、`loadingprogress`                     |
| 滚动     | `scroll`、`swiper`、`refresh`                     |
| 大数据   | `list`、`listitem`、`grid`、`griditem`            |
| 瀑布流   | `waterflow`、`flowitem`                           |
| 日期     | `calendar`、`datepicker`                          |
| 文本输入 | `textinput`、`textarea`                           |

## 基础布局

```rust
rsx! {
    column {
        percent_width: 1.0,
        percent_height: 1.0,
        padding: 16.0,
        align_items: "stretch",
        row {
            percent_width: 1.0,
            justify_content: "space-between",
            text { "标题" }
            button { "操作" }
        }
    }
}
```

`column`/`row` 负责主轴排列，`stack` 负责叠放和 alignment，`flex` 暴露更完整的 direction/wrap 语义。复杂页面先建立清晰容器层级，再使用绝对定位。

## 尺寸与盒模型

- `width`/`height` 默认是 vp 的 `f32`。
- `percent_width`/`percent_height` 使用 `0.0..=1.0`。
- `padding`/`margin` 可统一设置，也可用四向属性覆盖。
- `constraint_size`、`aspect_ratio`、`layout_weight` 用于约束和剩余空间分配。

```rust
row {
    width: 320.0,
    min_height: 48.0,
    padding_horizontal: 12.0,
    margin_bottom: 8.0,
}
```

属性是否存在以具体 tag descriptor 为准；不要假设所有节点拥有相同的 ArkUI attribute。

## 滚动内容

```rust
scroll {
    percent_width: 1.0,
    percent_height: 1.0,
    scroll_bar: true,
    column {
        for index in 0..100 {
            text { key: "{index}", padding: 12.0, "Item {index}" }
        }
    }
}
```

几十个轻量 item 可以直接声明；数百个以上或 item 构建昂贵时使用虚拟列表章节的 NodeAdapter。

## 布局排查

先确认父容器给出了可用尺寸，再检查百分比、主轴与交叉轴对齐。ArkUI 不会因为 child 设置 `percent_height: 1.0` 就替一个无确定高度的 parent 推导高度。
