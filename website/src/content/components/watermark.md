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
use arkit::shadcn::components::WatermarkFontStyle;

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
        ..WatermarkStyle::default()
    },
    column {
        width: "100%",
        // 多段原生文档内容
    }
}
```

## 图片水印源

图片水印支持 SVG 以及 PNG、WebP 等已编码图片数据。图片在生成水印纹理时解码一次，按指定尺寸绘制后由同一个重复 shader 覆盖内容。

```rust
use arkit::ArkImageSource;
use arkit::shadcn::components::{Watermark, WatermarkSource, WatermarkStyle};

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
| `style`    | `WatermarkStyle`  | 默认样式 | 字体、颜色、旋转和间距 |
| `children` | `Element`         | 必填     | 被水印覆盖的内容       |

`WatermarkStyle` 提供：

- 通用样式：`opacity`、`rotation_degrees`、`gap_x`、`gap_y`；
- 文字样式：`color`、`font_size`、`font_weight`、`font_style`、`font_family`；
- 图片尺寸：通过 `WatermarkSource::image(source, width, height)` 设置。

`color: None` 时使用当前主题前景色，最终透明度由 `opacity` 统一控制。图片源忽略文字专属字段。

## 长内容性能

组件不会为每一个水印创建 Dioxus 或 ArkUI 节点。文字只布局和栅格化一次，图片也只解码和缩放一次，结果缓存在一张透明纹理中，再由原生 `REPEAT` shader 用一次矩形绘制覆盖整个区域。因此内容变长时：

- 原生节点数量固定为 1 个水印绘制节点；
- 每次重绘只有 1 次纹理填充，不随水印数量增长；
- 文案和样式没有变化时复用缓存纹理；
- 单张纹理限制为最长边 2048px、最多 2,097,152 像素，极端文案或间距会自动降低栅格分辨率，避免无界内存分配。

水印节点使用 `hit_test_behavior: "none"`，不会影响下层交互。
