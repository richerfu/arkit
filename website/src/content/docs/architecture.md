---
title: 架构与 crate 边界
description: "VirtualDom、HostTree 和各个 crate 各管什么，依赖边界在哪里。"
---

# 架构与 crate 边界

Arkit 的 UI 只走 Dioxus 这一条路：业务写出组件树，运行时 diff 后投影到 ArkUI 原生节点，中间没有第二套 UI 状态。

## 各 crate 管什么

| Crate                              | 责任                                                                        |
| ---------------------------------- | --------------------------------------------------------------------------- |
| `arkit`                            | public facade、prelude、feature gates、入口 root wrapper                    |
| `arkit_derive`                     | `#[entry]` N-API lifecycle proc macro                                       |
| `arkit_runtime`                    | VirtualDom、OpenHarmony loop、event queue、window metrics、embedded WebView |
| `arkit_arkui`                      | HostTree、projection、attributes、events/gestures、image、virtual adapter   |
| `arkit_elements`                   | `rsx!` 使用的 ArkUI element/attribute/event registry                        |
| `arkit_prelude`                    | Dioxus primitives 与 elements 的无环共享 prelude                            |
| `arkit_hooks`                      | native node、layout、overlay、safe area、virtualization hooks               |
| `arkit_animation_core`             | 无 ArkUI 依赖的 resolve/compile/sample/state engine                         |
| `arkit_animation`                  | root frame driver、ArkUI/Drawing adapter、native lowering、交互             |
| `arkit_chart`                      | option/parser、series render、hit test、ECharts component                   |
| `arkit_router`                     | dioxus-router 的 ArkUI Link/back/transition 集成                            |
| `arkit_i18n` / `arkit_i18n_macros` | runtime locale 与编译期 Fluent catalog                                      |
| `arkit_icon`                       | embedded SVG catalog、raster source 与有界 cache                            |
| `arkit_lottie`                     | ThorVG worker、XComponent/NativeWindow Lottie 渲染                          |
| `arkit_camera`                     | CameraKit 预览、拍照；可选 scan 解码                                        |
| `arkit_barcode`                    | 独立条码/二维码生成（rxing），无相机依赖                                    |
| `arkit_terminal`                   | libghostty-vt + GPU surface；会话 I/O 由应用托管                            |
| `arkit_shadcn`                     | 主题 tokens、业务组件和浮层组合                                             |

领域 crate 不反向依赖 facade；facade 只是把它们拼起来对外。`arkit_prelude` 用来打断 shadcn / icon / animation 对 `arkit` 的环。

## HostTree 投影

逻辑上以 Dioxus 树为准。HostTree 记着模板路径、占位、文本子节点、ElementId、监听器和原生所有权；ArkUI 树上的节点只是这份逻辑树的投影。

典型规则：

- 动态 text child 更新父 ArkUI TextContent。
- native child reorder 先 detach 再插入目标位置，不留下 stale tail。
- native insert 失败不会把 logical host 误绑定到原 index 的 sibling。
- subtree dispose 释放 listener、gesture、image、virtual adapter 和 arena slot。
- native callback 使用 active token，拒绝注销后晚到的事件。

## 调度与事件

runtime 把 `VirtualDom::wait_for_work` waker 接到 OpenHarmony loop。每个 tick：

1. 执行已排队 UI-loop effects。
2. 把 owned native events 交给 Dioxus runtime。
3. drain scheduler ready work。
4. `render_immediate` 输出 mutation。
5. renderer 同步 ArkUI projection。
6. 重新注册 wait。

没有固定次数轮询。ArkUI callback 只复制 payload、入队、wake；禁止同步借用 VirtualDom 或重入 render。

## Window 与 Overlay

一个 root 只有一个 `WindowMetricsHandle`。所有 avoid areas 先与 XComponent content rect 求交，再转换 vp。

OverlayRoot 与 safe business subtree 是 root stack 的 sibling：

- backdrop 可以覆盖完整 window。
- panel/floating content 消费 safe viewport。
- edge-to-edge 只改变 business subtree policy。
- 每个 `use_overlay` token 独立，scope drop 只清理自己的 entry。

## 动画热路径

Animation core 使用 dense/generational ID、预编译 plan、typed values、复用 FrameBatch 和 dirty compare。resolution 阶段一次读取 target/schema/layout/window/baseline；正常帧不做文件 I/O、目录遍历、字符串 property lookup 或排序。

callback 与 controls command 通过 queue 隔离；adapter commit 成功后才发布 render/terminal events。普通帧不使 Dioxus scope 重渲染。

## 图表热路径

Chart model 是受控 snapshot。render transition 共享 `Rc` option，hit regions 来自实际绘制结果，文字排版按 style 使用有界 cache。Custom canvas/Drawing escape hatch 被封装在 `ECharts` 内，业务只传 props/controller。

## 原生所有权

OpenHarmony binding 中部分 node/adapter handle 没有隐式 Drop：

- `NodeBuilder` 在 build 前清理 early-error path。
- VirtualNodeAdapter 管理 attach/detach/reload、item wrapper，以及可见 RSX subtree 的 runtime owner。
- Embedded WebView attach 失败立即 dispose ArkTS controller。
- renderer dispose subtree 前先注销 event/gesture callback。
- UI-only native 对象保持 thread-local ownership，不跨线程析构。

返回错误不能遗留不可达 native handle。

## Feature 与依赖边界

核心 facade 不自动链接领域栈。主要 feature 边：

- `chart` / `router` → `animation`
- `shadcn` → `animation` + `i18n` + `icon`
- `markdown` / `code` → `shadcn`（`markdown-highlight` = 二者组合）
- `camera-scan` → `animation` + `camera` + scan decoder
- `lottie-network` → `lottie` + HTTP 栈；`lottie-expressions` 可选
- `barcode`、`terminal`、`camera`、`lottie` 各自独立可选

这些关系只在 root `Cargo.toml`/crate manifest 中声明，业务 crate 用 `arkit` features 选择。

第三方/internal dependency version 集中在 workspace `Cargo.toml`，成员 crate 使用 `workspace = true`，确保 ArkUI binding/sys 类型身份一致。

## 何时使用哪个层

| 需求                     | 层                              |
| ------------------------ | ------------------------------- |
| 普通页面和组件           | `arkit::prelude` + RSX          |
| 领域功能                 | facade feature + 对应 namespace |
| 虚拟 RSX 列表            | `use_virtual_node_adapter_rsx`  |
| 布局观测/adapter/WebView | `arkit_hooks` / facade hooks    |
| 自定义 native item       | `NodeBuilder`                   |
| 框架贡献：渲染 mutation  | `arkit_arkui`                   |
| 框架贡献：调度/窗口      | `arkit_runtime`                 |
| 平台无关动画算法         | `arkit_animation_core`          |

业务不要直接依赖内部 renderer tree 或重新实现 host context。应用只依赖公开 facade。
