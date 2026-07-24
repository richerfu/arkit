---
title: GPU 终端
description: "嵌入 GPU 终端：组件只负责渲染，shell / SSH 由你自己接。"
---

# GPU 终端

`terminal` feature 提供一块 GPU 加速的嵌入式终端。VT 解析和栅格化交给 libghostty-vt（Zig），像素画在 ArkUI 的 XComponent / NativeWindow 上。

有一点要先说清楚：**本地 shell 和 SSH 会话由你自己管**。框架不帮你 fork 进程，也不内置伪终端管理器——组件只负责「画出来」和「把输入交出去」。

## 启用方式

```toml
[dependencies]
arkit = { version = "*", features = ["terminal"] }
```

```rust
use arkit::prelude::*;
use arkit::terminal::{Terminal, TerminalController, TerminalProps};
```

完整可跑的代码在 `examples/terminal`：本地 shell 回显和 SSH 两种宿主都有。UI 侧只接 `on_input` / `on_write_pty`，宿主输出再喂给 `controller.feed_vt`。

## 谁干什么

```text
  Terminal.on_input     ──►  应用 Host（LocalShell | SSH write）
  Host 输出字节流       ──►  controller.feed_vt
  Terminal.on_write_pty ──►  同一 Host write（DA/DSR 等设备应答）
```

| 层                   | 负责                                      | 不负责                    |
| -------------------- | ----------------------------------------- | ------------------------- |
| `Terminal` 组件      | 栅格、光标、选区、IME gutter、焦点与绘制  | fork/exec、SSH 握手、密钥 |
| `TerminalController` | `feed_vt`、resize、focus、选区/剪贴板协作 | 业务会话生命周期          |
| 应用 Host            | PTY/shell、SSH channel、编码与重连策略    | VT 语义与 GPU 帧提交      |

## 最小接入

```rust
let controller = TerminalController::new();

rsx! {
    Terminal {
        controller: controller.clone(),
        cols: 80,
        rows: 24,
        on_input: move |data: Vec<u8>| {
            // 写入本地 PTY 或 SSH channel
            let _ = data;
        },
        on_write_pty: move |data: Vec<u8>| {
            // 终端主动写出的设备应答
            let _ = data;
        },
    }
}

// 宿主收到输出后：
// controller.feed_vt(&bytes);
```

列/行应与布局区域匹配；示例采用接近卡片宽高比的 `40×24` 网格，避免单元格拉伸或底部空白带。

## 运行与依赖

- 构建目标需要可用的 Zig 工具链以编译 libghostty-vt 侧产物（见 `crates/arkit_terminal`）。
- 设备侧依赖 GPU/XComponent 路径；模拟器是否完整取决于平台实现。
- 打包进 HAP 时与其他 example 相同：一次只部署一个 `cdylib`，`moduleName` 与 `.so` 名称对齐。

```sh
cd examples/terminal
ohrs build --arch aarch
cd ../../
./app/run.sh terminal all
```

## 与其他能力的关系

- **不依赖** `shadcn` / `camera` / `chart`。可与 shadcn 页面并存，但 Terminal 自身不是 shadcn compound。
- WebView 用于嵌入网页；Terminal 用于 VT 会话。二者都走独立 native surface，不要混用同一 controller。
- 输入法：组件预留顶部 gutter（示例 `IME_TOP_GUTTER`），避免软键盘遮挡首行。

## 验收清单

- 本地 Host 模式下字符、退格、换行与滚动正确。
- SSH 模式下连接、断开、重连不泄漏 channel；切回本地模式后 UI 仍可输入。
- `feed_vt` 高频输出时帧率稳定，离开页面后 surface/worker 释放。
- resize（列行变化）后栅格与选区状态一致。
