---
title: 架构与 crate 边界
---

# 架构与 crate 边界

Arkit 的唯一 UI 运行模型是 Dioxus：

```text
#[entry] root
    → Dioxus VirtualDom + scheduler
    → WriteMutations
    → arkit_arkui HostTree
    → deterministic projection
    → ArkUI native nodes
```

## Crate 所有权

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
| `arkit_shadcn`                     | 主题 tokens、业务组件和浮层组合                                             |

domain crate 不依赖 facade；facade 组合 domain API。`arkit_prelude` 避免 shadcn/icon/animation 反向依赖 `arkit` 形成环。

## HostTree 投影

Dioxus logical tree 是真实来源。HostTree 保存 template path、placeholder、text child、ElementId、listener 和 native ownership。ArkUI tree 是当前 logical tree 的投影。

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
- VirtualNodeAdapter 管理 attach/detach/reload 与 item wrapper。
- Embedded WebView attach 失败立即 dispose ArkTS controller。
- renderer dispose subtree 前先注销 event/gesture callback。
- UI-only native 对象保持 thread-local ownership，不跨线程析构。

返回错误不能遗留不可达 native handle。

## Feature 与依赖边界

核心 facade 不自动链接领域栈。`chart` 依赖 animation；`router` 依赖 animation；`shadcn` 依赖 animation + icon；`markdown` 依赖 shadcn，并单独启用 `arkit_shadcn` 的 Markdown 解析依赖。这些关系只在 root `Cargo.toml`/crate manifest 中声明，业务 crate 用 `arkit` features 选择。

第三方/internal dependency version 集中在 workspace `Cargo.toml`，成员 crate 使用 `workspace = true`，确保 ArkUI binding/sys 类型身份一致。

## 何时使用哪个层

| 需求                     | 层                              |
| ------------------------ | ------------------------------- |
| 普通页面和组件           | `arkit::prelude` + RSX          |
| 领域功能                 | facade feature + 对应 namespace |
| 布局观测/adapter/WebView | `arkit_hooks` / facade hooks    |
| 自定义 native item       | `NodeBuilder`                   |
| 框架贡献：渲染 mutation  | `arkit_arkui`                   |
| 框架贡献：调度/窗口      | `arkit_runtime`                 |
| 平台无关动画算法         | `arkit_animation_core`          |

业务不要直接依赖内部 renderer tree 或重新实现 host context。公开 facade 是应用兼容边界。
