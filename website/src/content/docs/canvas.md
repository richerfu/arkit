---
title: Canvas 2D
description: "W3C 风格 CanvasRenderingContext2D 与 ArkUI 原生绘制。"
---

# Canvas 2D

`canvas` feature 以 [HTML Canvas 2D 标准](https://html.spec.whatwg.org/multipage/canvas.html#the-canvas-element) 为 API 与默认语义基线，提供 W3C/WHATWG 风格的 `CanvasRenderingContext2D`。组件直接使用 ArkUI Custom draw context，不嵌入 WebView，也不创建 XComponent surface。

```toml
arkit = { version = "*", features = ["canvas"] }
```

```rust
let draw = CanvasRenderer::new(move |context| {
    let gradient = context.create_linear_gradient(16.0, 16.0, 196.0, 96.0).unwrap();
    gradient.add_color_stop(0.0, "#2563eb").unwrap();
    gradient.add_color_stop(1.0, "#7c3aed").unwrap();
    context.set_fill_style(gradient);
    context.begin_path();
    context.round_rect(16.0, 16.0, 180.0, 80.0, 18.0).unwrap();
    context.fill();

    context.save();
    context.translate(180.0, 80.0);
    context.rotate(0.25);
    context.set_stroke_style("rgba(15, 23, 42, 0.8)");
    context.set_line_width(4.0);
    context.stroke_rect(-60.0, -36.0, 120.0, 72.0);
    context.restore();
});

rsx! {
    Canvas {
        draw,
        percent_width: 1.0,
        height: 320.0,
    }
}
```

## 组件属性

| Prop                | 默认值       | 说明                                                  |
| ------------------- | ------------ | ----------------------------------------------------- |
| `draw`              | 必填         | native draw frame 内同步执行的 renderer               |
| `width` / `height`  | 高度 300vp   | 固定 logical size                                     |
| `percent_width`     | `1.0`        | 相对宽度                                              |
| `percent_height`    | `None`       | 相对高度；设置后取消默认高度                          |
| `clear_before_draw` | `false`      | 每次受控重绘前清为 transparent black                  |
| `settings`          | 默认 2D 配置 | alpha、色彩空间、存储格式与性能提示                   |
| `controller`        | `None`       | 重绘、logical size 查询与 backing-store snapshot 句柄 |

## W3C 对齐范围

Canvas 使用持久 native bitmap 作为 backing store。组件重绘只把 backing store 合成到 ArkUI Custom canvas；尺寸或 DPR 变化时才重建表面。因此像 Web Canvas 一样，未主动清除的像素、drawing state、clip 和当前 path 会跨 draw callback 保留。当前实现保持左上原点、logical pixel、无效赋值保持旧值、路径不属于 drawing state、`save/restore` 状态栈和 clockwise-positive 角度语义；current path 的坐标在命令执行时固化当前变换，不会在 `fill/stroke` 时重复变换。

- 状态：`save/restore/reset`、实际生效的 context attributes、global alpha、全部 Porter-Duff/混合 composite operation、image smoothing、line dash/cap/join/miter。`alpha: false` 使用 opaque backing store；当前 OH_Drawing bitmap 不支持 Display-P3/float16 backing store，请求这两项时 `get_context_attributes()` 会如实返回 sRGB/unorm8 fallback。
- 样式：CSS Color 4 absolute color（完整命名色、hex、rgb/hsl/hwb、Lab/LCH、OKLab/OKLCH、`color()` 预定义色彩空间）、linear/radial/conic gradient、CanvasPattern、shadow、CSS filter chain。gradient 保留创建时包含非均匀缩放与 skew 的完整 CTM；Pattern 与 drawImage 共用 context 的 sampling quality。依赖 DOM computed style 的 `currentColor` 不在 native context 内解析。
- 矩形：`clear_rect`、`fill_rect`、`stroke_rect`。
- 路径：`Path2D` SVG/clone/addPath、move/line/quadratic/Bezier、`arc_to`、arc、ellipse、rect、1–4 radii roundRect、fill/stroke/clip、point-in-path 与 point-in-stroke。显式 `Path2D` 在使用时应用 CTM，默认 path 不会二次变换；stroke outline 在当前用户空间生成后应用完整 CTM，因此非均匀缩放与 skew 不再退化为平均线宽。
- 变换：scale/rotate/translate/transform/set/reset/get transform。
- 文字：CSS font shorthand、fill/stroke text、max width、direction、CSS length spacing、kerning、stretch、caps、text-rendering，以及完整字段的 `CanvasTextMetrics`。字体族/weight/width/slant 通过系统 FontManager 匹配，OpenType caps/kerning feature 和 hinting/edging 设置直接作用于最终字形；`start/end` 随 LTR/RTL 解析，metrics 使用实际 native glyph bounds 并相对当前 align/baseline 返回。ASCII control whitespace 会按标准折叠为空格，非正数或 NaN `maxWidth` 不绘制。
- 图片与像素：`CanvasImage`、三种 drawImage 参数形态、`ImageData` create/get/put/dirty rect、Pattern 和 sampling quality。`ImageData` 支持 `rgba-unorm8` 与 `rgba-float16`、sRGB 与 Display-P3 元数据和转换；create/get/dirty rect 接受 W3C 对应的有符号尺寸，负尺寸会归一化，零尺寸返回错误。`CanvasController::snapshot()` 将已挂载 Canvas 的 backing store 复制为 `CanvasImage`，可交给另一个 Canvas 的 `draw_image*` 或 `create_pattern`，对应 Web Canvas 作为 `CanvasImageSource` 的场景。

## 平台边界

下面的差异不是遗漏的 native 实现，而是当前 OHOS drawing/backing store 或无 DOM 环境没有等价能力：

| 标准能力                                                               | 当前处理                                                                                                                                                                        |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Display-P3 / float16 context backing store                             | `get_context_attributes()` 如实返回 sRGB/unorm8 fallback；`ImageData` 仍支持 P3/float16 数据与转换                                                                              |
| `currentColor`、relative/system font、`rem` 等依赖 computed style 的值 | 不解析；font 支持 absolute length，spacing 支持 absolute length、`em` 和 `normal`                                                                                               |
| `drawFocusIfNeeded()`、`scrollPathIntoView()`                          | 依赖 DOM `Element`，由 ArkUI focus/scroll 容器处理                                                                                                                              |
| HTML image/video、`VideoFrame`、`ImageBitmap`                          | native API 统一接收 `CanvasImage`；Canvas snapshot 可直接转为该类型                                                                                                             |
| CSS `filter: url(...)`                                                 | 依赖 DOM/SVG resource，不接受；颜色矩阵、blur 与单个 drop-shadow 直接走 native filter。当前 native filter 只能保留一个 shadow layer，因此多个/交错 drop-shadow chain 不宣称等价 |
| 浏览器级 bidi、连字与 fallback-run shaping                             | 可见文字使用稳定的 native Font/TextBlob 路径；字体选择、OpenType caps、spacing 和 metrics 已生效，但复杂脚本的整段 shaping 不宣称与浏览器完全一致                               |
| context lost/restored event                                            | backing store 是进程内 CPU bitmap，不存在 GPU context loss；`is_context_lost()` 恒为 `false`                                                                                    |
| `direction: inherit`                                                   | native context 没有 DOM ancestor direction，按 LTR fallback；显式 LTR/RTL 完整用于 logical align                                                                                |

`ImageBitmap`、HTML image/video 等浏览器对象不直接泄漏到 Rust/OHOS API。平台资源应先适配为 `CanvasImage`。

`CanvasRenderingContext2D` 只能在 `CanvasRenderer` 回调内使用，不能跨帧保存 native 引用。外部数据变化应通过 Dioxus signal 生成新的 renderer，或调用 `CanvasController::request_redraw()`。
