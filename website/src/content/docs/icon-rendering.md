---
title: 渲染与缓存
description: "图标尺寸、颜色和描边，以及栅格缓存怎么工作。"
---

# 渲染与缓存

图标最终会栅格成位图再画上去。尺寸、颜色、描边宽度都可以配，结果会进有界缓存，避免同一图标反复解码。

## 基础与描边

```rust
rsx! {
    row {
        {icon("settings", 24.0, 0xFF334155)}
        {arkit_icon::icon_with_stroke(
            "arrow-right",
            20.0,
            0xFF16A34A,
            1.5,
        )}
    }
}
```

facade 直接导出 `icon`、`has_icon`、`icon_names`；`icon_with_stroke` 和默认常量由 `arkit_icon` crate 公开，直接使用时需把该 workspace crate 加入依赖。

## 参数

| 参数           | 说明                              |
| -------------- | --------------------------------- |
| `name`         | Lucide 名称；内部会规范化         |
| `size`         | 逻辑 vp；最小钳制为 1             |
| `color`        | ARGB `u32`；alpha 会写入 SVG rgba |
| `stroke_width` | 描边宽度；最小钳制为 0.1          |

raster edge 是 `size × display pixel ratio`，因此 RSX 布局仍用 vp，而位图按设备密度生成。

## 缺失图标

`has_icon` 可在开发期提前校验。运行时找不到名称时组件显示统一 missing glyph，而不是 panic；这保证页面可渲染，但不应代替测试。

## 缓存

渲染后的 SVG 使用 UI-thread local、容量 128 的有界缓存。key 包含 name、size、color、stroke width 和 pixel ratio。相同规格复用字符串和 renderer 资源路径。

不要为一个连续动画每帧生成不同 size/color，这会不断产生新 cache key。需要缩放/透明度动画时让 image node 做 transform/opacity；主题切换只生成有限颜色变体。

## 列表性能

列表中使用固定图标规格并保留稳定 item key。图标本身会创建原生 image node；超大列表仍应结合 NodeAdapter 虚拟化，而不是依赖 SVG 缓存解决节点数量。
