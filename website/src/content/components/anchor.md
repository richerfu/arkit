---
title: Anchor
description: "页内锚点导航：点击导航项跳转到指定区块，滚动时自动高亮当前可见区块。"
---

# Anchor

页内锚点导航（scrollspy）。左侧导航列表点击后让右侧 `Scroll` 滚动到对应区块顶部，滚动过程中自动高亮当前可见区块。

## 用法

```rust
Anchor {
    scroll_duration: 300,
    nav: rsx! {
        column {
            AnchorItem { id: "intro".to_string(), title: "Introduction".to_string() }
            AnchorItem { id: "install".to_string(), title: "Installation".to_string() }
        }
    },
    children: rsx! {
        column {
            width: "100%",
            AnchorSection {
                id: "intro".to_string(),
                children: rsx! { /* 区块内容 */ }
            }
            AnchorSection {
                id: "install".to_string(),
                children: rsx! { /* 区块内容 */ }
            }
        }
    },
}
```

`Anchor` 渲染 `row { nav, scroll }`：`nav` 通常是一列 [`AnchorItem`](#anchoritem)，`children` 内放置一组 [`AnchorSection`](#anchorsection)。区块位置通过 `use_native_element_ref` + `use_layout_frame` 测量（物理像素），滚动使用 `<scroll>` 的 `scroll_offset` 一次性命令（vp），经 `WindowMetricsHandle.scale` 换算；`onscroll` 的 vp 增量累积为当前位置，与注册的区块帧比较得出激活项。

## Props

### Anchor

| Prop               | 类型                       | 默认     | 说明                                                 |
| ------------------ | -------------------------- | -------- | ---------------------------------------------------- |
| `nav`              | `Element`                  | —        | 左侧导航内容（通常是一列 `AnchorItem`）              |
| `children`         | `Element`                  | —        | 右侧滚动内容（通常是一组 `AnchorSection`）           |
| `scroll_ref`       | `Option<NativeElementRef>` | 自建     | 外部 scroll ref，传入后调用方可以额外观察滚动容器    |
| `scroll_bar`       | `Option<String>`           | `"auto"` | 滚动条策略：`"off"` / `"auto"` / `"on"`              |
| `scroll_duration`  | `u32`                      | `0`      | 跳转动画时长（ms），0 = 瞬跳                         |
| `active_threshold` | `f32`                      | `0.0`    | 区块判定为"已进入视口"的阈值（vp），避免区块边界闪烁 |

`scroll_duration` / `active_threshold` 只在挂载时生效，运行期修改不回传上下文。

### AnchorSection

| Prop       | 类型      | 说明                                |
| ---------- | --------- | ----------------------------------- |
| `id`       | `String`  | 区块 id，需在最近的 `Anchor` 内唯一 |
| `children` | `Element` | 区块内容                            |

包一层 `column { width: "100%" }` 并注册其测量位置到最近的 `Anchor`，卸载时自动注销。

### AnchorItem

| Prop      | 类型                       | 默认     | 说明                       |
| --------- | -------------------------- | -------- | -------------------------- |
| `id`      | `String`                   | —        | 对应 `AnchorSection` 的 id |
| `title`   | `String`                   | —        | 导航文案                   |
| `active`  | `Option<bool>`             | 自动计算 | 手动指定激活态             |
| `onclick` | `Option<EventHandler<()>>` | —        | 点击回调                   |

点击时滚动到对应区块；`active` 为 `None` 时随滚动自动高亮。

### use_anchor()

```rust
pub fn use_anchor() -> Option<AnchorContext>
```

读取最近的 `Anchor` 上下文（必须在 `Anchor` 子树内调用），返回：

| 方法                | 说明               |
| ------------------- | ------------------ |
| `jump(id)`          | 滚动到指定区块顶部 |
| `active_id()`       | 当前激活区块 id    |
| `scroll_position()` | 当前滚动位置（vp） |
