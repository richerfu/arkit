---
title: 组件库介绍
description: "安装、导入、组件边界与完整索引。"
---

# 组件库介绍

`arkit_shadcn` 是基于 Dioxus、ArkUI element、Arkit Animation 与 OverlayRoot 实现的原生组件库。启用 `shadcn` feature 后即可使用，且会自动启用 `animation`、`i18n` 与 `icon`。

`Markdown` 与 `Code` 是独立的可选能力：

- `markdown`：CommonMark/GFM 原生渲染（自动启用 `shadcn`）
- `code`：独立语法高亮代码块 + tree-sitter 注册（可不依赖 Markdown）
- 两者同时启用时，Markdown 围栏块复用 Code 管线；`markdown-highlight` 是二者的便捷别名

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

1. 在应用 root 安装 ThemeProvider。
2. 阅读“状态模型”，确定受控或非受控用法。
3. 按名称打开具体组件页，查看公开 Props、示例和生命周期。
4. 完整示例在 `examples/shadcn_showcase`。

## Chart 的边界

组件库 `Chart` 是轻量主题化展示组件，不等同 `arkit::echarts::ECharts`。需要 22 类 series、Action、DataZoom、增量更新和导出时，使用顶部“图表”文档。
