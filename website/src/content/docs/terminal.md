---
title: GPU 终端
description: "嵌入 GPU 终端：Arkit 管理窗口与布局，应用接入真实 PTY 或 SSH。"
---

# GPU 终端

`terminal` feature 提供一块 GPU 加速的嵌入式终端。VT 解析交给 **rio-vt**（纯 Rust，不依赖 Zig），栅格在 GPU 上展开：顶点着色器按 `instance_index` 走格子，片元着色器采样字形图集并画光标/下划线。像素画在 ArkUI 的 XComponent / NativeWindow 上。

有一点要先说清楚：**本地 shell 和 SSH 会话由你自己管**。框架不帮你 fork 进程，也不内置伪终端管理器——组件只负责「画出来」和「把输入交出去」。

ANSI 状态机是顺序的，所以 **VT 解析留在 CPU**。这和 Ghostty / Rio / Alacritty 一样。GPU 负责的是格子布局、装饰和字形采样，而不是把整屏 CPU 栅格化后再 blit。

## 启用方式

```toml
[dependencies]
arkit = { version = "*", features = ["terminal"] }
```

```rust
use arkit::prelude::*;
use arkit::terminal::{Terminal, TerminalController, TerminalProps};
```

完整代码在 `examples/terminal`，由 `examples/demos` 的入口编译到 HAP。示例包含真实本地 PTY、内置演示命令和 SSH。UI 侧接 `on_input` / `on_write_pty`，宿主输出再喂给 `controller.feed_vt`。

## 谁干什么

```text
  Terminal.on_input     ──►  应用 Host（PTY | SSH | 演示命令）
  Host 输出字节流       ──►  controller.feed_vt
  Terminal.on_write_pty ──►  同一 Host write（DA/DSR 等设备应答）
  Arkit 布局变化        ──►  Terminal.config + Host 的 PTY 尺寸
```

| 层                   | 负责                                     | 不负责                    |
| -------------------- | ---------------------------------------- | ------------------------- |
| Arkit                | 窗口状态、键盘避让数据、原生布局订阅     | shell 进程与命令执行      |
| `Terminal` 组件      | VT 栅格、光标、输入编码、焦点与 GPU 绘制 | fork/exec、SSH 握手、密钥 |
| `TerminalController` | `feed_vt`、`reconfigure`、键盘显示与隐藏 | 业务会话生命周期          |
| 应用 Host            | PTY/shell、SSH channel、编码与重连策略   | VT 语义与 GPU 帧提交      |

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

列/行应与布局区域匹配。示例把 `use_native_element_ref` 绑定到终端容器，通过 `use_layout_size` 获取物理像素尺寸，再按 `use_window_metrics` 的比例换算行列数。布局变化同时更新 `Terminal.config`、本地 PTY 的 `TIOCSWINSZ` 和 SSH 的 `window_change`。组件也会应用挂载后的配置变化，不需要手动尺寸按钮。

## 真实本地 PTY

点击 `Local PTY` 会打开 OS 的 master/slave 伪终端，并启动 `/system/bin/sh -i`。后端通过 `setsid` 和 `TIOCSCTTY` 设置控制终端，用非阻塞 master 和有界队列传输原始字节。尺寸变化会通知前台进程组；断开或离开页面会关闭 master 并回收 shell。

OHOS 下的初始目录与 `HOME` 都是应用自己的 `/data/storage/el2/base/files/terminal`。普通应用不能列出沙箱根目录 `/`：在该目录执行 `ls` 会出现 `Permission denied`，即使命令已经正常启动。应用目录支持普通文件操作；系统目录和用户目录仍遵循各自的访问权限。

`PTY probe` 依次检查真实 tty、当前目录、`ls -la` 和 `printf | cat` 管道，全部成功后才输出 `ARKIT_REAL_PTY`。PTY 打开失败会显示实际错误，输入也不会切到内置演示命令。

## 运行与依赖

- 构建目标只需要 Rust 工具链（`rio-vt`，无 Zig）。
- 设备侧依赖 GPU/XComponent 路径；模拟器是否完整取决于平台实现。
- `examples/terminal` 是页面库；`examples/demos` 是包含入口的 `cdylib`，对应 `libdemos.so`。

```sh
cd examples/demos
ohrs build --arch aarch
cd ../../
HDC_TARGET="127.0.0.1:10090" ./app/run.sh
```

将 `HDC_TARGET` 换成 `hdc list targets` 中的目标设备地址，打开整合示例中的 Terminal 页面，再点击 `Local PTY`。如果目标模拟器允许安装 unsigned HAP，可使用：

```sh
hdc -t 127.0.0.1:10090 install -r app/entry/build/default/outputs/default/entry-default-unsigned.hap
hdc -t 127.0.0.1:10090 shell aa start -a EntryAbility -b com.arkit.example
```

本机后端测试无需链接 OHOS UI 库，使用 Python 3.11+ 和 Rust 即可运行：

```sh
python3 scripts/test-terminal-host.py --toolchain 1.97.1 --offline \
  --target-dir /tmp/arkit-terminal-host-target
```

测试直接使用生产代码，覆盖真实 tty、尺寸变化、Ctrl-C、退出状态、输出背压下的子进程回收，以及键盘/鼠标编码。

## 与其他能力的关系

- **不依赖** `shadcn` / `camera` / `chart`。可与 shadcn 页面并存，但 Terminal 自身不是 shadcn compound。
- WebView 用于嵌入网页；Terminal 用于 VT 会话。二者都走独立 native surface，不要混用同一 controller。
- 输入法：示例使用 Arkit 的窗口指标计算键盘避让；终端组件处理 IME 会话与输入字节。

## 验收清单

- 真实 PTY 下字符、退格、换行、Ctrl-C、退出状态与滚动正确。
- SSH 模式下连接、断开、重连不泄漏 channel；切回本地模式后 UI 仍可输入。
- `feed_vt` 高频输出时帧率稳定，离开页面后 surface/worker 释放。
- resize（列行变化）后栅格与选区状态一致。
- 在可读的应用目录中，`ls -la` 和实际 shell 管道正常执行。
- 输出队列满时断开会话仍能回收 shell；实体键盘不把 `KEYCODE_*` 标签写入 PTY。
