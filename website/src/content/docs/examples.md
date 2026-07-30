---
title: 示例索引
description: "仓库里有哪些示例、各自练什么，以及怎么编进真机验证。"
---

# 示例索引

仓库里目前有 **14** 个可在设备上跑的示例（见 workspace 的 `members`）。它们既是上手材料，也是公开 API 是否还能编译、能否上真机的活合同。

## 一览

| 示例              | 覆盖能力                                                                  | 关键入口                              |
| ----------------- | ------------------------------------------------------------------------- | ------------------------------------- |
| `counter`         | `#[entry]`、RSX、signal、click                                            | `examples/counter/src/lib.rs`         |
| `async_task`      | `use_resource`、Tokio timer、UI wake                                      | `examples/async_task/src/lib.rs`      |
| `animation`       | timeline、easing、controls、layout/presence、drag/scroll、lowering        | `examples/animation/src/lib.rs`       |
| `camera`          | CameraKit 拍照/扫码双模式、可配置工具栏、分辨率与完整控制项               | `examples/camera/src/lib.rs`          |
| `canvas`          | W3C Canvas 2D 完整绘制面：Path2D、paint、文字、图片与像素                 | `examples/canvas/src/lib.rs`          |
| `barcode`         | 独立二维码/条形码生成、`Barcode` / `use_barcode`、PNG 导出                | `examples/barcode/src/lib.rs`         |
| `chart`           | 22 series、realtime option、actions、events、appendData、coordinate query | `examples/chart/src/lib.rs`           |
| `complex_cases`   | 10,000 item List/Grid/WaterFlow、单项动态更新与变高重排                   | `examples/complex_cases/src/lib.rs`   |
| `i18n`            | Fluent macro、typed message、locale switch、Cargo rename                  | `examples/i18n/src/lib.rs`            |
| `lottie`          | ThorVG worker、XComponent 帧同步、播放控制、fit/repeat/speed              | `examples/lottie/src/lib.rs`          |
| `router`          | typed routes、ArkUI Link、dynamic param、route transition                 | `examples/router/src/lib.rs`          |
| `shadcn_showcase` | themes、表单、导航、浮层、反馈、Guide、数据展示组件                       | `examples/shadcn_showcase/src/lib.rs` |
| `terminal`        | GPU 终端、本地 shell / SSH Host、`feed_vt` 与 IME gutter                  | `examples/terminal/src/lib.rs`        |
| `webview`         | embedded mount、URL、title callback、reload/focus/zoom                    | `examples/webview/src/lib.rs`         |

## 怎么编、怎么装

在对应 example 目录里执行：

```sh
cd examples/counter
ohrs build --arch aarch
```

把 `counter` 换成你要的示例名。`--arch aarch` 对应常见的 arm64 设备；别的 ABI 按你本机 ohrs / 工具链配置来。

建议一次只装一个 example 的最新产物，并保证 app 壳里的 `moduleName` 和 `.so` 名字对得上。

```sh
# 从仓库根目录把构建产物装进 app 并运行（脚本名以仓库为准）
./app/run.sh counter all
```

## counter

最小端到端基线。确认：

- N-API init/render/destroy 正常。
- Text child 随 signal 更新。
- click 经 runtime queue 派发。
- 退出 Ability 后 runtime/native tree 清理。

## async_task

验证 future 在无用户输入时完成也会唤醒 UI。等待期间界面应保持响应，完成后自动更新。它区分 OpenHarmony scheduler wiring 是否正确，而不仅是 Rust future 是否能运行。

## animation

包含多个 lab：

- easing：内建/irregular/spring 和 typed value。
- timeline：position、label、nested、iterations、alternate。
- interaction：drag、scroll、seek。
- lifecycle：play/pause/reverse/cancel/reset/revert/finished。
- orchestration：layout、presence、stagger、scope、lowering report。

这是动画更改的主要目标回归入口。

## chart

展示全部 series family 与图表 instance operations。除画面外检查 tooltip/hit-test、legend/dataZoom、selection state、realtime transition、appendData、coordinate conversion 和图片导出。

## canvas

通过 `canvas` feature 使用持久化 ArkUI Custom + Native Drawing backing store，不创建额外 XComponent。示例验证高 DPI logical pixel、gradient/pattern/shadow、roundRect/arcTo/ellipse、SVG Path2D、dash、affine transform、ImageData/drawImage、fill/stroke text 与完整文字测量。

## camera

使用全屏 XComponent 预览并覆盖 Rust ArkUI 控件，验证：

- CameraKit session 与 XComponent Surface 的生命周期绑定。
- 后置/前置相机切换、暂停和恢复。
- JPEG 拍照数据从 ImageNative/NativeBuffer 安全复制到 `CapturedPhoto`。
- 权限拒绝、无相机设备和 native error 的可观察状态。
- 启用扫码路径时（`camera-scan`）解码结果与配置项。

相机画面与拍照必须在带 CameraKit 相机设备的真机上验收。没有虚拟相机的模拟器仍可验证 HAP 加载、权限请求、Surface 建立和 `Unavailable` 错误路径。

## barcode

与相机扫码解耦的**生成**路径（`barcode` feature）。确认：

- `encode_barcode` / `use_barcode` 对 QR 与常见 1D 格式产出位图。
- UI 线程不直接跑同步重编码；hook 路径走异步导出。
- PNG/base64 导出与组件展示一致。

```sh
cd examples/barcode && ohrs build --arch aarch
./app/run.sh barcode all
```

## complex_cases

三个 host 同时保留，通过 visibility/height 切换：

- List：固定高 10,000 item。
- Grid：两列。
- WaterFlow：五种循环高度与固定 auto-fill track。
- 三种容器都可选择目标 index 并执行 `reload_items(index, 1)`；revision、颜色立即变化，WaterFlow 同时改变该项高度。

它验证 NodeAdapter attach、wrapper kind、RSX item scope、item recycle、局部 signal、局部失效和变高重新测量。点击可见 item 会更新它独立的 `taps` signal；点击“更新单项”时，业务数据先更新，再同步 reload 目标 item，用于验证 subtree 卸载、重建与 native wrapper 生命周期。

## lottie

通过独立 `lottie` feature 引入 ThorVG 和 NativeWindow 依赖。示例包含持续旋转、缩放、渐变与多轨道图层，验证播放/暂停/停止、seek、倍速、循环/往返和 contain/cover/fill。进入后台或让组件完全不可见后，frame callback 不再提交渲染；恢复可见时从原帧继续。

```sh
cd examples/lottie
ohrs build --arch aarch
cd ../../
./app/run.sh lottie all
```

## i18n

使用 `framework = { package = "arkit", ... }` rename，验证 proc macro 不硬编码 `::arkit`。修改 locale 资源后重新编译，可检查 key/args consistency diagnostics。

## router

包含 Home、Settings、`Users { id }`。验证 Link push、系统返回、dynamic param 和 route mount transition。

## shadcn_showcase

组件库覆盖面最大，真机回归应分组检查：

- theme preset 与 light/dark。
- controlled/uncontrolled inputs。
- Dialog/Sheet/Drawer/Popover/Menu safe-area。
- Tabs/Accordion/Carousel navigation。
- Guide 分步引导。
- Sonner timer、action 与 swipe dismiss。
- 离开页面后的 overlay 和 timer cleanup。

## terminal

GPU 终端 + 应用自管 Host：

- 本地 shell 回显与基础行编辑。
- SSH 连接/断开/切回本地。
- `on_input` / `on_write_pty` 与 `controller.feed_vt` 数据通路。
- IME 与顶部 gutter、resize 后栅格一致。

```sh
cd examples/terminal
ohrs build --arch aarch
cd ../../
./app/run.sh terminal all
```

详见 [GPU 终端](../terminal/)。

## webview

需要有效网络和 Ability helper。测试：

- 首次 mount 与 layout frame。
- URL 输入和显式 load。
- reload、focus、zoom。
- title callback 经 `queue_ui_loop` 更新。
- 页面卸载、返回与 resize。

## Workspace 验证

文档/纯 Rust 变更至少运行：

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

涉及 ArkUI/N-API/runtime 的变更还必须选择受影响 example 执行 `ohrs build` 和真机交互。host cargo 命令无法验证目标 symbols、HAP packaging 或设备行为。

站点（Astro）变更：

```sh
pnpm install
pnpm run check
pnpm run website:build
```
