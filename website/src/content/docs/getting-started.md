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
napi-ohos = "1.2.0"
napi-derive-ohos = "1.2.0"

[build-dependencies]
napi-build-ohos = "1.2.0"
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

`#[entry]` 函数不能是 `async`，返回值是 `Element`；函数本身不带参数，或者带一个 `OpenHarmonyApp` 句柄参数（见下文"自定义桥接插件"）。宏会帮你生成这些导出：

- `init`：安装 Ability-session bridge 与 init context，并返回 Rust 插件声明。
- `render`：按 render-owner token 把 native XComponent 和 `VirtualDom` 挂到 `NodeContent`。
- `disposeRender` / `disposeAllRenders`：卸载 renderer 与 native tree。
- `disposeBridge`：只释放匹配 owner 的 Ability-session transport。
- `onBackPressIntercept`：把系统返回键转交当前 handler stack。

框架 root 会自动装好 `RuntimeHandle`、窗口度量和安全区；开了 animation 还会挂上 AnimationHost。Portal 由 renderer 原生投影，业务入口里不用安装额外 host/provider。

## 自定义桥接插件

应用可以注册自己的 openharmony-ability `BridgePlugin` facade（与框架内置的 `ohos.webview` 同一机制），有**两种可组合的写法**：

1. **声明式列表** —— `#[entry(plugins = [MyPlugin, UrlBridgePlugin])]`。宏在生成的 `init` 里、框架插件之后逐个注册（此时尚未投递任何 ArkTS 插件事件）。每一项是表达式，在模块作用域解析，单元类型或构造调用都行；注册失败只记 hilog，不会中断初始化。
2. **App 句柄参数** —— entry 函数接收一个 `OpenHarmonyApp` 克隆，在 render（Ability-session bridge 安装之后）传入，函数体内任意 `handle.register_plugin(...)`。晚注册是安全的：注册器会对 `REQUIRED_CONTEXTS` 已就绪的插件重放有界生命周期历史。

```rust
use arkit::prelude::*;
use openharmony_ability::{AsyncBridge, BridgePlugin, OpenHarmonyApp, PluginLifecycleEvent};

struct MyPlugin;

impl BridgePlugin for MyPlugin {
    type Mode = AsyncBridge;
    const ID: &'static str = "myapp.config";

    fn on_lifecycle(&self, event: &PluginLifecycleEvent) -> napi_ohos::Result<()> {
        // 可选：Ability 创建 / UI 上下文就绪等生命周期
        Ok(())
    }
}

#[entry(plugins = [MyPlugin])]
fn app(handle: OpenHarmonyApp) -> Element {
    // 第二种方式：entry 函数体内手动注册
    // let _ = handle.register_plugin(MyPlugin);
    rsx! { text { "hello" } }
}
```

需要 ArkTS 侧实现的插件（如 `@ohos-rs/ability-plugin-url` 的 `UrlPlugin`），要把它的实例加进宿主 ability 的 `bridgePlugins` 数组（`app/entry/src/main/ets/entryability/EntryAbility.ets`）——这一侧需要手工维护，`#[entry]` 是 Rust 侧 proc-macro，无法触碰 ArkTS 源码。

## 安全区策略

业务内容默认铺满挂载表面（edge-to-edge），框架不做安全区避让。需要避让时在业务组件里按需取 insets 自己应用：

```rust
#[entry]
fn app() -> Element {
    // 默认：内容铺满，无安全区 padding
    let safe = use_safe_area(); // 需要时取 insets（vp）
    rsx! {
        column {
            padding_top: safe.top,   // 自行决定是否避让状态栏
            // ...
        }
    }
}
```

`use_safe_area()` 返回当前安全区 insets，`SafeArea` 组件可以按边选择应用；`use_window_metrics()` 等窗口度量始终可用。显式挂载时也可用 `mount_entry_with_policy(..., SafeAreaPolicy::Safe)` 恢复框架 root 的自动 padding。

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
