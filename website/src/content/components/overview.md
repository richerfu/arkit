---
title: 组件库介绍
description: "shadcn 风格原生组件库怎么装、怎么 import，以及和 Markdown / 终端的关系。"
---

# 组件库介绍

`arkit_shadcn` 是一套跑在 ArkUI 上的原生组件，交互语义尽量对齐 [shadcn/ui](https://ui.shadcn.com)，但不是 Web 套壳，也不会再起一个 VirtualDom。打开 `shadcn` feature 就能用，并会顺带启用 `animation`、`i18n` 和 `icon`。

`Markdown` 和 `Code` 是另外两个可选能力：

- `markdown`：原生渲染 CommonMark / GFM（会自动带上 `shadcn`）
- `code`：独立代码高亮，底层 tree-sitter，可以不依赖 Markdown
- 两个都开时，Markdown 围栏会复用 Code 管线；嫌麻烦可以用 `markdown-highlight`

`Barcode` 走单独的 `barcode` feature，不强制 `shadcn`。GPU 终端在文档区的 [Terminal](../docs/terminal/)，不属于这套 compound 组件。

## 安装与导入

```toml
[dependencies]
arkit = { version = "*", features = ["shadcn"] }
```

仅 Markdown：

```toml
arkit = { version = "*", features = ["markdown"] }
```

仅代码高亮组件：

```toml
arkit = { version = "*", features = ["code"] }
```

Markdown 围栏也要高亮：

```toml
arkit = { version = "*", features = ["markdown", "code"] }
# 或: features = ["markdown-highlight"]
```

```rust
use arkit::prelude::*;
use arkit::shadcn::components::*;
use arkit::shadcn::theme::*;
```

## 文档结构

本区分为两类文档：

- 通用指南：主题、状态所有权、布局/浮层、交互与可访问性。
- 组件 API：每个公开组件族单独一页，compound primitive 记录在所属组件页，例如 CardHeader 属于 Card，TabsTrigger 属于 Tabs。

## 运行模型

组件不是 Web shadcn wrapper，也不创建第二个 VirtualDom。它们直接组合 ArkUI 原生节点；Dialog、Menu、Sonner 等通过应用唯一 OverlayRoot 发布；动画共用 root AnimationHost。

## 内置文案与 i18n

Calendar、Chart、Combobox、Command、DatePicker、Markdown、Pagination 和
Select 的内置文案自带 `en-US`、`zh-CN` 资源。应用 root 安装
`use_i18n_provider` 后，这些组件会读取同一个响应式 locale；调用
`I18nContext::set_locale_id` 会同步刷新组件默认文案。未安装 provider
或 locale 不受支持时回退到 `en-US`。

业务显式传入的 placeholder、label 或 labels 始终优先，可用于领域文案
或更多语言。组件库只复用应用的 locale 状态，不要求业务 catalog 重复
声明 shadcn 的消息 key。

## 推荐入口

1. 在应用 root 安装 `ThemeProvider`（及需要时的 `use_i18n_provider`）。
2. 阅读「状态模型」，确定受控或非受控用法。
3. 按名称打开具体组件页，查看公开 Props、示例和生命周期。
4. 完整示例在 `examples/shadcn_showcase`（含 Guide 等新组件）。
5. 需要 ECharts 级图表时离开本区，打开顶部「图表」文档。

## Chart 的边界

组件库 `Chart` 是轻量主题化展示组件，不等同 `arkit::echarts::ECharts`。需要 22 类 series、Action、DataZoom、增量更新和导出时，使用顶部“图表”文档。
