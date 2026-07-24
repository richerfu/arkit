---
title: 安装与第一个应用
description: "从空 crate 走到能在设备上跑起来的第一个 Arkit 应用。"
---

# 安装与第一个应用

这一页带你搭一个最小可运行的 Arkit 模块。如果想直接看完整代码，仓库里的 `examples/counter` 就是参考基线。

## 前置条件

- Rust 1.88 或更高版本。
- 已安装并配置 OpenHarmony SDK/NDK。
- `ohrs` 构建工具可用。
- 一个可加载 native `.so` 的 OpenHarmony 应用壳。

文档站点自己用 Node.js 22 和 pnpm 10.24，跟跑 Arkit 应用无关。

## Cargo 配置

业务 crate 需要输出 `cdylib`，并依赖 Arkit facade 和 N-API 工具链：

```toml
[package]
name = "counter"
version = "0.1.0"
edition = "2021"
rust-version = "1.88"

[lib]
crate-type = ["cdylib"]

[dependencies]
arkit = { version = "*" }
napi-ohos = "1.1"
napi-derive-ohos = "1.1"

[build-dependencies]
napi-build-ohos = "1.1"
```

仓库里的 example 走 workspace 依赖；你在自己的项目里直接依赖发布版本即可。额外能力用 `arkit` 的 feature 打开：

```toml
arkit = {
  version = "*",
  features = ["animation", "router", "i18n", "icon", "shadcn"]
}
```

常用可选 feature（完整表见首页与 [架构](architecture/)）：

| Feature                          | 作用                         |
| -------------------------------- | ---------------------------- |
| `animation` / `router` / `chart` | 动画与依赖动画的路由、图表   |
| `i18n` / `icon`                  | Fluent 文案与 Lucide 图标    |
| `shadcn` / `markdown` / `code`   | 业务组件、Markdown、语法高亮 |
| `camera` / `camera-scan`         | 预览拍照 / 扫码              |
| `barcode`                        | 无相机的码生成               |
| `lottie` / `lottie-network`      | Lottie 渲染 / 网络源         |
| `terminal`                       | GPU 终端组件                 |
| `full`                           | 打开全部领域能力             |

feature 会连同它的 native 依赖一起拉进来。比如没开 `camera`，就不会带上 `arkit_camera` 和 CameraKit；基础渲染用到的共享 ArkUI 绑定不受影响。

`build.rs` 只初始化 N-API 构建：

```rust
fn main() {
    napi_build_ohos::setup();
}
```

## 编写入口

```rust
use arkit::prelude::*;

#[entry]
fn app() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        column {
            width: "100%",
            height: "100%",
            align_items: "center",
            justify_content: "center",

            text {
                font_size: 28.0,
                "count = {count}"
            }
            button {
                margin_top: 12.0,
                onclick: move |_| count += 1,
                "increment"
            }
        }
    }
}
```

`#[entry]` 函数不能带参数，也不能是 `async`，返回值是 `Element`。宏会帮你生成这些导出：

- `init`：安装 Ability init context 与 resource manager。
- `render`：保存 ArkTS helper/main-thread env，并把一个 `VirtualDom` 挂到 `NodeContent`。
- `destroy`：卸载 renderer，释放 runtime 和 native tree。
- `on_back_press_intercept`：把系统返回键转交当前 handler stack。

框架 root 会自动装好 ArkHost、窗口度量、安全区和 OverlayRoot；开了 animation 还会挂上 AnimationHost。业务入口里不用再装一遍。

## 安全区策略

默认入口把业务内容放在 visual safe area 内：

```rust
#[entry]
fn app() -> Element {
    // ...
}
```

沉浸式内容使用：

```rust
#[entry(edge_to_edge)]
fn app() -> Element {
    // ...
}
```

edge-to-edge 只取消业务 root 的默认 padding。`use_window_metrics()`、`use_safe_area()` 和框架浮层避让仍然有效；ArkTS 宿主也必须配置对应的 window 模式。

## 构建

从具体 example 目录执行目标构建：

```sh
cd examples/counter
ohrs build --arch aarch
```

host 侧检查可尽早发现 Rust 类型错误：

```sh
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

但 OpenHarmony 依赖、N-API export、ArkUI symbols 与目标链接只能由 `ohrs build` 验证。

## 打包与运行

仓库 `app/run.sh` 按一个 example 一次完成 native artifact 拷贝、hvigor 打包和设备安装。多个 example 都输出 native 库，不要在同一次运行中混用陈旧产物。

排查顺序：

1. 确认 Rust crate 名和最终 `lib<name>.so`。
2. 确认 ArkTS `moduleName` 使用裸名称，不含 `lib` 与 `.so`。
3. 确认 target ABI 与设备一致。
4. 确认 `build.rs` 调用了 `napi_build_ohos::setup()`。
5. 先看 `ohrs` 链接错误，再看设备 hilog 中的 N-API/ArkUI 错误。
