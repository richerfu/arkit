---
title: 属性与样式
description: "类型约定、颜色、单位与条件样式。"
---

# 属性与样式

Arkit element 的属性是编译期 descriptor。RSX 更新后 renderer 只提交变化的 attribute；删除条件属性会恢复对应 native 默认值或清除状态。

属性值按 **CSS 语义** 编码：长度用 vp / `"N%"`，枚举用关键字字符串。完整关键字表见 [元素与布局](/docs/elements-layout)。

## 常用属性族

| 属性族 | 示例                                                                 |
| ------ | -------------------------------------------------------------------- |
| 盒模型 | `width`、`height`、`padding_*`、`margin_*`                           |
| 排列   | `align_items`、`justify_content`、`align_self`、`alignment`          |
| 外观   | `background_color`、`opacity`、`border_*`、`shadow`、`clip`          |
| 文本   | `font_size`、`font_weight`、`font_color`、`line_height`、`max_lines` |
| 交互   | `enabled`、`visibility`、`focusable`、`hit_test_behavior`            |
| 定位   | `position`、`z_index`                                                |

## 颜色与单位

颜色接受 ARGB `u32` 或 hex 字符串：

```rust
text {
    font_color: 0xFF0F172Au32,
    background_color: "#fff8fafc",
    font_size: 18.0,
    "内容"
}
```

尺寸默认是 vp。百分比写在 `width` / `height` 上：

```rust
column {
    width: "100%",
    height: "50%",
    padding: "12 16",
}
```

需要物理像素或窗口换算时从 `WindowMetrics` 读取 scale，不在业务中写设备常量。

## 条件样式

把视觉状态从业务状态直接派生：

```rust
let active = use_signal(|| false);

rsx! {
    button {
        background_color: if active() { 0xFF16A34Au32 } else { 0xFF334155u32 },
        opacity: if active() { 1.0 } else { 0.72 },
        shadow: if active() { "sm" },
        "状态"
    }
}
```

不要在点击回调里直接修改 native attribute，再让 RSX 保留旧值；下一次 diff 会以 RSX 为准覆盖它。

## 封装设计 Token

在组件层集中定义 color、spacing、radius、typography 与关键字常量，页面只消费 token，不散落硬编码色值与尺寸。

需要完整主题系统时使用顶部“组件”中的 ThemeProvider。基础 element 本身不依赖 shadcn feature。

## 文本与裁剪

```rust
text {
    max_lines: 2,
    text_overflow: "ellipsis",
    line_height: 22.0,
    "最多显示两行"
}
```

文本换行依赖可确定的宽度。圆角、阴影和 child 越界组合时明确设置 `clip`，并在真机检查 renderer 与 ArkUI 的实际层合成结果。
