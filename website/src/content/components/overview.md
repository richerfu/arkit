---
title: 组件库介绍
description: "安装、导入、组件边界与完整索引。"
---

# 组件库介绍

`arkit_shadcn` 是基于 Dioxus、ArkUI element、Arkit Animation 与 OverlayRoot 实现的原生组件库。启用 `shadcn` feature 后即可使用，且会自动启用 `animation` 与 `icon`。

`Markdown` 是独立的可选能力。使用该组件时启用 `markdown` feature；它会自动启用 `shadcn`，并且仅在此时引入 Markdown 解析依赖。

## 安装与导入

```toml
[dependencies]
arkit = { version = "*", features = ["shadcn"] }
```

仅使用 Markdown 时可以直接写：

```toml
arkit = { version = "*", features = ["markdown"] }
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

## 推荐入口

1. 在应用 root 安装 ThemeProvider。
2. 阅读“状态模型”，确定受控或非受控用法。
3. 按名称打开具体组件页，查看公开 Props、示例和生命周期。
4. 完整示例在 `examples/shadcn_showcase`。

## Chart 的边界

组件库 `Chart` 是轻量主题化展示组件，不等同 `arkit::echarts::ECharts`。需要 22 类 series、Action、DataZoom、增量更新和导出时，使用顶部“图表”文档。
