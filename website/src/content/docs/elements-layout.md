---
title: 元素与布局
description: "有哪些原生元素可用，以及如何用接近 CSS 的写法描述布局。"
---

# 元素与布局

`arkit::prelude::*` 会带上 ArkUI 元素表。RSX 里的 tag 对应原生节点。

属性尽量按 **CSS 语义** 写：长度用 vp 或 `"N%"`，盒模型支持简写，枚举类属性用字符串关键字。

## 元素一览

| 分类 | RSX tag                                                                      |
| ---- | ---------------------------------------------------------------------------- |
| 布局 | `column` `row` `stack` `flex`                                                |
| 内容 | `text` `image` `button`                                                      |
| 输入 | `textinput` `textarea` `checkbox` `toggle` `radio` `slider`                  |
| 滚动 | `scroll` `list` `grid` `waterflow` `swiper` `refresh`                        |
| 其它 | `progress` `loadingprogress` `calendar` `datepicker` `custom` `xcomponent` … |

## 长度与百分比

| 写法                                         | 结果                                      |
| -------------------------------------------- | ----------------------------------------- |
| `12` / `12.0` / `"12"` / `"12px"` / `"12vp"` | 绝对 vp                                   |
| `"50%"` / `"100%"`                           | 相对父级（`100%` → ArkUI fraction `1.0`） |

`width` / `height` 写百分比时映射为 ArkUI `WidthPercent` / `HeightPercent`。

```rust
column {
    width: "100%",
    height: "100%",
    padding: "16 20",
}
```

## margin / padding

### 简写（CSS box，1–4 值，空格或逗号）

| 输入          | `[top, right, bottom, left]` |
| ------------- | ---------------------------- |
| `8` / `"8px"` | 全 8                         |
| `"8 16"`      | 上下 8、左右 16              |
| `"8 16 12"`   | 上 8、左右 16、下 12         |
| `"8 16 12 4"` | 四边                         |

### 拆分与轴

| 属性                                  | 含义 |
| ------------------------------------- | ---- |
| `*_top` `*_right` `*_bottom` `*_left` | 单边 |
| `*_x` / `*_horizontal`                | 左右 |
| `*_y` / `*_vertical`                  | 上下 |

**合并顺序**：简写 → 轴 → 单边（后者覆盖前者）。

```rust
column {
    margin: "12 0",
    margin_top: 8.0,
    padding_horizontal: 16.0,
    padding_y: "8px",
}
```

### 约束

`min_width` / `max_width` / `min_height` / `max_height`（及 `constraint_size` / `max_width_constraint`）合并为 ArkUI `ConstraintSize`。

## 关键字总表

枚举类属性使用 **字符串关键字**（或文档中写明的布尔写法）。`font_weight` 另接受 CSS 数字权重 `100`–`900`。

### 排版 / 文本

| 属性                                           | 关键字                                                        |
| ---------------------------------------------- | ------------------------------------------------------------- |
| `text_align`                                   | `start`/`left` · `center` · `end`/`right` · `justify`         |
| `font_weight`                                  | `thin`…`black` / `normal` / `medium` / `bold`，或 `400`/`700` |
| `font_style`                                   | `normal` · `italic`                                           |
| `text_decoration`                              | `none` · `underline` · `overline` · `line-through`            |
| `text_overflow`                                | `none` · `clip` · `ellipsis` · `marquee`                      |
| `font_size` / `line_height` / `letter_spacing` | 长度（可带 `px`/`vp`）                                        |

### 布局

| 属性                            | 关键字                                                                                      |
| ------------------------------- | ------------------------------------------------------------------------------------------- |
| `align_items`                   | `start`/`top` · `center` · `end`/`bottom`（column/row）；flex 另支持 `stretch` `baseline`   |
| `justify_content`               | `start` · `center` · `end` · `space-between` · `space-around` · `space-evenly`              |
| `flex_direction`                | `row` · `column` · `row-reverse` · `column-reverse`                                         |
| `flex_wrap`                     | `nowrap` · `wrap` · `wrap-reverse`                                                          |
| `align_self` / `item_alignment` | `auto` · `start` · `center` · `end` · `stretch` · `baseline`                                |
| `alignment`（stack）            | `center` · `top` · `bottom` · `start`/`left` · `end`/`right` · `top-start` · `bottom-end` … |
| `position`                      | `"x y"` 或单值（可带单位）                                                                  |
| `opacity`                       | `0..=1` 或 `"50%"`                                                                          |

### 视觉

| 属性                             | 关键字                                                        |
| -------------------------------- | ------------------------------------------------------------- |
| 颜色类                           | `#rgb` `#rrggbb` `#aarrggbb`；或 `0xAARRGGBB`                 |
| `border_style`                   | `solid` · `dashed` · `dotted`                                 |
| `border_width` / `border_radius` | 盒模型简写（1–4 值）                                          |
| `visibility`                     | `visible` · `hidden` · `none`                                 |
| `object_fit`                     | `contain` · `cover` · `fill` · `scale-down` · `none` · `auto` |
| `shadow`                         | `none` · `xs` · `sm` · `md` · `lg` · `floating-sm` …          |
| `hit_test_behavior`              | `default`/`auto` · `block` · `transparent` · `none`           |
| `clip` / `enabled` / `focusable` | `true`/`false` 或 `"on"`/`"off"`                              |

### 滚动 / 列表 / 输入 / 控件

| 属性                                       | 关键字                                                           |
| ------------------------------------------ | ---------------------------------------------------------------- |
| `scroll_bar`（scroll/list/grid/waterflow） | `off`/`false` · `auto` · `on`/`true`                             |
| `scroll_edge_effect`                       | `spring`/`bounce` · `fade` · `none`                              |
| `scroll_enabled`                           | bool / `"on"`/`"off"`                                            |
| `list_sticky`                              | `none` · `header` · `footer` · `both`                            |
| `scroll_to_index`（list，一次性命令）      | `"12"` 或 `"12,0,0"`（index[,smooth,align]），不写入声明式状态   |
| `input_type`                               | `text` · `number` · `phone` · `email` · `password` · `decimal` … |
| `show_password_icon`（textinput）          | bool；显示密码显隐图标                                           |
| `progress_type`                            | `linear`/`bar` · `ring` · `eclipse` · `scale-ring` · `capsule`   |
| `button_type`                              | `normal` · `capsule` · `circle`                                  |
| `swiper_curve`                             | `linear` · `ease` · `ease-in` · `ease-out` · `ease-in-out` · …   |
| `swiper_*` 开关                            | bool / `"on"`/`"off"`                                            |

## 示例

```rust
rsx! {
    column {
        width: "100%",
        height: "100%",
        padding: "16 20",
        align_items: "stretch",
        background_color: "#fafafa",

        scroll {
            width: "100%",
            height: "100%",
            scroll_bar: "off",
            scroll_edge_effect: "spring",

            text {
                font_size: "17px",
                font_weight: "semibold",
                text_align: "center",
                text_decoration: "none",
                "标题"
            }

            image {
                width: "100%",
                height: 180.0,
                object_fit: "cover",
                border_radius: "12 12 0 0",
                src: url,
            }

            button {
                margin_top: 12.0,
                button_type: "capsule",
                "操作"
            }
        }
    }
}
```

## 编码规则

- 长度：数字 vp（`padding: 16.0`）或带单位字符串（`"16px"` / `"12vp"`）。
- 百分比尺寸：`width` / `height` 写 `"N%"`。
- 枚举：关键字字符串（`text_align: "center"`、`shadow: "sm"`、`visibility: "hidden"`）。
- 无法识别的关键字不写入 native。
- 虚拟列表等场景常用 `scroll_bar: false` / `"off"`。
