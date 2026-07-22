---
title: Barcode
description: "二维码 / 条形码生成（独立 barcode feature）。"
---

# Barcode

`Barcode` 与 `encode_barcode` / `use_barcode` 提供二维码与条形码**生成**能力，基于 rxing（ZXing Rust 移植）。与 `camera-scan` 扫码完全解耦：不需要相机权限，也不依赖 CameraKit。

```toml
[dependencies]
arkit = { version = "*", features = ["barcode"] }
```

## 三层 API

| 层     | API                                | 场景                                  |
| ------ | ---------------------------------- | ------------------------------------- |
| 纯函数 | `encode_barcode` → `BarcodeBitmap` | 批处理、单测、**调用方自己的 worker** |
| Hook   | `use_barcode(contents, options)`   | 内容变化自动重编码 + 异步导出         |
| 组件   | `Barcode { … }`                    | 声明式预览                            |

### 线程模型（重要）

| 路径                                                             | 线程                             |
| ---------------------------------------------------------------- | -------------------------------- |
| `Barcode` / `use_barcode` 编码                                   | Tokio **blocking pool**（非 UI） |
| handle `png_bytes_async` / `base64_png_async` / `save_png_async` | blocking pool，回调回 UI         |
| 纯函数 `encode_barcode` / `to_png_bytes`                         | **同步**，禁止在 UI 线程直接调用 |

UI 侧状态含 `BarcodePhase::Encoding`，用于占位，避免主线程卡住。

### 展示（默认）

编码结果优先转 **SVG → `ArkImageSource`**，与 icon 同一套 image 管线，无临时文件。

### 导出（按需，异步）

```rust
// UI：走 handle 异步 API
code.base64_png_async(|result| { /* UI 回调 */ });
code.save_png_async(path, |result| { /* … */ });

// 非 UI / 自管线程：
let bmp = encode_barcode(&BarcodeRequest::qr(url, 256))?;
let png = bmp.to_png_bytes()?;
```

## 组件

```rust
Barcode {
    contents: pay_url(),
    format: BarcodeFormat::QrCode,
    size: 220.0,
}
```

| Prop       | 说明                       |
| ---------- | -------------------------- |
| `contents` | 载荷                       |
| `format`   | 默认同 QR                  |
| `size`     | 矩阵边长 / 一维码宽度      |
| `height`   | 一维码高度（可选）         |
| `options`  | 覆盖 margin / 颜色 / EC 等 |
| `on_error` | 编码失败回调               |

## Hook

```rust
let mut text = use_signal(|| "https://example.com".to_string());
let mut options = use_signal(|| BarcodeOptions::qr(256));
let code = use_barcode(text, options);

// code.image() → Option<ArkImageSource>
// code.phase() → Empty | Encoding | Ready | Error
// code.save_png_async(path, |result| { … });
```

## 格式

`QrCode`、`DataMatrix`、`Aztec`、`Pdf417`、`Code128`、`Code93`、`Code39`、`Codabar`、`Ean13`、`Ean8`、`UpcA`、`UpcE`、`Itf`。

与 `CameraScanFormat` 同构，便于「扫到再生成」手写映射。

## 与扫码的关系

|      | `barcode`      | `camera-scan`              |
| ---- | -------------- | -------------------------- |
| 方向 | 生成           | 识别                       |
| 依赖 | rxing encoders | CameraKit + rxing decoders |
| UI   | `Barcode`      | `CameraView` Scan 模式     |

## 示例

```bash
cd examples/barcode && ohrs build --arch aarch
./app/run.sh barcode all
```
