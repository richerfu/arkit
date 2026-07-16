---
title: 属性与样式
description: "类型约定、颜色、单位与条件样式。"
---

# 属性与样式

Arkit element 的属性是编译期 descriptor。RSX 更新后 renderer 只提交变化的 attribute；删除条件属性会恢复对应 native 默认值或清除状态。

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

颜色接受 renderer 支持的 ARGB `u32` 或颜色字符串：

```rust
text {
    font_color: 0xFF0F172Au32,
    background_color: "#fff8fafc",
    font_size: 18.0,
    "内容"
}
```

尺寸值默认是 vp。需要物理像素或窗口换算时从 `WindowMetrics` 读取 scale，不在业务中写设备常量。

## 条件样式

把视觉状态从业务状态直接派生：

```rust
let active = use_signal(|| false);

rsx! {
    button {
        background_color: if active() { 0xFF16A34Au32 } else { 0xFF334155u32 },
        opacity: if active() { 1.0 } else { 0.72 },
        "状态"
    }
}
```

不要在点击回调里直接修改 native attribute，再让 RSX 保留旧值；下一次 diff 会以 RSX 为准覆盖它。

## 封装设计 Token

带枚举语义的底层属性可能接受整数或字符串编码。项目应在组件层集中定义 color、spacing、radius、typography 和 ArkUI enum 映射，避免 magic number 散落在页面。

需要完整主题系统时使用顶部“组件”中的 ThemeProvider。基础 element 本身不依赖 shadcn feature。

## 文本与裁剪

```rust
text {
    max_lines: 2,
    text_overflow: 1,
    line_height: 22.0,
    "最多显示两行"
}
```

文本换行依赖可确定的宽度。圆角、阴影和 child 越界组合时明确设置 `clip`，并在真机检查 renderer 与 ArkUI 的实际层合成结果。
