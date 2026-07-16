---
title: 安装与第一个应用
description: "创建 crate、声明入口并完成构建。"
---

# 安装与第一个应用

本章建立一个最小 Rust native 模块。完整可运行基线位于 `examples/counter`。

## 前置条件

- Rust 1.88 或更高版本。
- 已安装并配置 OpenHarmony SDK/NDK。
- `ohrs` 构建工具可用。
- 一个可加载 native `.so` 的 OpenHarmony 应用壳。

站点本身使用 Node.js 22、pnpm 10.24；它们不是运行 Arkit 应用的依赖。

## Cargo 配置

业务 crate 必须输出 `cdylib`，并依赖 facade 与 N-API toolchain：

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

仓库内 example 使用 workspace dependencies；外部项目直接依赖已发布版本。领域能力通过 `arkit` feature 启用：

```toml
arkit = {
  version = "*",
  features = ["animation", "camera", "router", "i18n", "icon"]
}
```

每个领域能力及其 native 依赖都跟随对应 feature。未启用 `camera` 时不会引入 `arkit_camera`、CameraKit 及该领域新增的 native surface/image 依赖边；基础 renderer 已经使用的共享 ArkUI 绑定不受此规则影响。

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
            percent_width: 1.0,
            percent_height: 1.0,
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

`#[entry]` 的函数必须无参数、非 `async`，返回 `Element`。宏会生成：

- `init`：安装 Ability init context 与 resource manager。
- `render`：保存 ArkTS helper/main-thread env，并把一个 `VirtualDom` 挂到 `NodeContent`。
- `destroy`：卸载 renderer，释放 runtime 和 native tree。
- `on_back_press_intercept`：把系统返回键转交当前 handler stack。

框架 root 会自动安装 ArkHost、WindowMetrics、安全区、OverlayRoot；启用 animation 时还会安装 root-owned AnimationHost。业务入口不要重复安装这些 provider。

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
