---
title: Watermark
description: "使用单个原生绘制节点覆盖文本水印，并在长内容场景保持固定节点数和绘制调用数。"
---

# Watermark

`Watermark` 在内容上方重复绘制文字或图片，同时不拦截子内容的点击、滑动或滚动事件。

```rust
use arkit::shadcn::components::{Watermark, WatermarkSource, WatermarkStyle};

Watermark {
    source: WatermarkSource::text("ARKIT · INTERNAL"),
    width: "100%".to_string(),
    height: "1200".to_string(),
    style: WatermarkStyle {
        rotation_degrees: -22.0,
        gap_x: 96.0,
        gap_y: 80.0,
        ..WatermarkStyle::default()
    },
    column {
        width: "100%",
        height: 1200.0,
        // 页面内容
    }
}
```

文字源支持换行符，可用于多行水印。`height` 默认是 `auto`，会跟随被包裹的内容；固定高度或长列表场景也可以显式传入尺寸。

## 自定义文字样式

```rust
use arkit::shadcn::components::{
    WatermarkBlendMode, WatermarkFontStyle, WatermarkShadow, WatermarkStroke,
};

Watermark {
    source: WatermarkSource::text("ARKIT\nCONFIDENTIAL"),
    style: WatermarkStyle {
        color: Some(0xFF2563EB),
        font_size: 16.0,
        font_weight: 700,
        font_style: WatermarkFontStyle::Italic,
        font_family: Some("HarmonyOS Sans".to_string()),
        opacity: 0.24,
        rotation_degrees: -18.0,
        gap_x: 72.0,
        gap_y: 56.0,
        offset_x: 18.0,
        offset_y: -8.0,
        repeat_origin_x: 28.0,
        repeat_origin_y: 20.0,
        blend_mode: WatermarkBlendMode::Multiply,
        stroke: Some(WatermarkStroke::new(0xCCFFFFFF, 1.2)),
        shadow: Some(WatermarkShadow::new(0x66000000, 4.0, 3.0, 4.0)),
        ..WatermarkStyle::default()
    },
    column {
        width: "100%",
        // 多段原生文档内容
    }
}
```

### 偏移与重复起点

`offset_x`、`offset_y` 移动每一个重复单元里的水印标记，不改变单元尺寸和重复间距。标记跨过单元边界时会在相邻边缘连续绘制，不会被截断。

`repeat_origin_x`、`repeat_origin_y` 移动整张重复网格相对容器左上角的起点。它只改变 shader 相位，不重新生成纹理。正值向右、向下移动。

### 混合模式

`blend_mode` 支持：

- `Normal`、`Multiply`、`Screen`、`Overlay`；
- `Darken`、`Lighten`、`ColorDodge`、`ColorBurn`；
- `HardLight`、`SoftLight`、`Difference`、`Exclusion`；
- `Hue`、`Saturation`、`Color`、`Luminosity`、`Plus`。

混合发生在最终一次水印覆盖绘制中。默认值是 `Normal`。

### 描边与阴影

`WatermarkStroke` 设置文字轮廓颜色和宽度，只作用于文字源。图片源会忽略 `stroke`，避免将透明 Logo 错误地描成矩形边框。

`WatermarkShadow` 设置阴影颜色、模糊半径以及水平、垂直偏移，文字和图片源都支持。描边和阴影只在缓存纹理生成时绘制一次；组件会把效果范围计入纹理边界，避免旋转后被裁切。

## 图片水印源

图片水印支持 SVG 以及 PNG、WebP 等已编码图片数据。图片在生成水印纹理时解码一次，按指定尺寸绘制后由同一个重复 shader 覆盖内容。

```rust
use arkit::ArkImageSource;
use arkit::shadcn::components::{
    Watermark, WatermarkShadow, WatermarkSource, WatermarkStyle,
};

let logo = ArkImageSource::encoded(
    "brand-logo",
    include_bytes!("brand-logo.png").to_vec(),
    512,
    160,
);

Watermark {
    source: WatermarkSource::image(logo, 128.0, 40.0),
    style: WatermarkStyle {
        opacity: 0.2,
        rotation_degrees: -16.0,
        gap_x: 72.0,
        gap_y: 58.0,
        shadow: Some(WatermarkShadow::new(0x55000000, 5.0, 2.0, 3.0)),
        ..WatermarkStyle::default()
    },
    column {
        width: "100%",
        // 页面内容
    }
}
```

首期图片源是内嵌或本地编码数据，不直接加载网络 URL。网络图片应由业务层完成下载和缓存后，再构造 `ArkImageSource`。

## 覆盖图片内容

水印源和被覆盖内容互相独立，`children` 也可以直接是图片。

```rust
Watermark {
    source: WatermarkSource::text("PREVIEW"),
    height: "236".to_string(),
    style: WatermarkStyle {
        color: Some(0xFFFFFFFF),
        font_size: 16.0,
        font_weight: 700,
        opacity: 0.6,
        ..WatermarkStyle::default()
    },
    image {
        src: "https://example.com/preview.jpg".to_string(),
        width: "100%",
        height: 236.0,
        object_fit: "cover",
    }
}
```

## Props

| 属性       | 类型              | 默认值   | 说明                   |
| ---------- | ----------------- | -------- | ---------------------- |
| `source`   | `WatermarkSource` | 必填     | 重复绘制的文字或图片   |
| `width`    | `String`          | `100%`   | 容器宽度               |
| `height`   | `String`          | `auto`   | 容器高度               |
| `style`    | `WatermarkStyle`  | 默认样式 | 字体、效果、位置和间距 |
| `children` | `Element`         | 必填     | 被水印覆盖的内容       |

### WatermarkStyle

| 字段                                 | 默认值                       | 作用范围  | 说明                           |
| ------------------------------------ | ---------------------------- | --------- | ------------------------------ |
| `color`                              | 当前主题前景色               | 文字      | 填充颜色                       |
| `font_size`                          | `14.0`                       | 文字      | 字号，单位 vp                  |
| `font_weight`                        | `500`                        | 文字      | 字重，限制为 `100..=900`       |
| `font_style`                         | `Normal`                     | 文字      | 正常、斜体或倾斜               |
| `font_family`                        | `None`                       | 文字      | 自定义字体族                   |
| `opacity`                            | `0.14`                       | 通用      | 最终覆盖透明度，限制为 `0..=1` |
| `rotation_degrees`                   | `-22.0`                      | 通用      | 顺时针旋转角度                 |
| `gap_x`、`gap_y`                     | `80.0`、`64.0`               | 通用      | 重复单元之间的空白             |
| `offset_x`、`offset_y`               | `0.0`、`0.0`                 | 通用      | 标记在重复单元内的偏移         |
| `repeat_origin_x`、`repeat_origin_y` | `0.0`、`0.0`                 | 通用      | 整张重复网格的起点             |
| `blend_mode`                         | `WatermarkBlendMode::Normal` | 通用      | 与下层内容的混合方式           |
| `stroke`                             | `None`                       | 文字      | 字形描边                       |
| `shadow`                             | `None`                       | 文字/图片 | 阴影颜色、模糊和偏移           |

图片尺寸通过 `WatermarkSource::image(source, width, height)` 设置。

`color: None` 时使用当前主题前景色，最终透明度由 `opacity` 统一控制。图片源忽略文字专属字段。

## 长内容性能

组件不会为每一个水印创建 Dioxus 或 ArkUI 节点。文字只布局和栅格化一次，图片也只解码和缩放一次，结果缓存在一张透明纹理中，再由原生 `REPEAT` shader 用一次矩形绘制覆盖整个区域。因此内容变长时：

- 原生节点数量固定为 1 个水印绘制节点；
- 每次重绘只有 1 次纹理填充，不随水印数量增长；
- 文案和纹理样式没有变化时复用缓存纹理；
- 仅修改 `opacity`、`blend_mode` 或重复起点时不会重新栅格化；
- 单张纹理限制为最长边 2048px、最多 2,097,152 像素，极端文案或间距会自动降低栅格分辨率，避免无界内存分配。

水印节点使用 `hit_test_behavior: "none"`，不会影响下层交互。
