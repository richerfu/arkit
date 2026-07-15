---
title: 图标概览
---

# 图标概览

启用 `icon` 后，`arkit_icon` 提供编译进二进制的 Lucide SVG。运行时按名称查找、应用颜色/尺寸/描边配置，再生成 ArkUI `image` 可持有的资源。

## 基础用法

```rust
use arkit::prelude::*;

rsx! {
    row {
        {icon("settings", 24.0, 0xFF334155)}
        text { "设置" }
    }
}
```

`icon` 返回完整 `Element`；业务组件不要把它当成 image source，也不要假设任意字符串名称都存在。

## 查询目录

```rust
assert!(has_icon("settings"));

for name in icon_names() {
    // 用于图标浏览器或开发期搜索
}
```

`icon_names()` 适合工具页，不建议在每次业务 render 中扫描完整目录。常用图标名集中成模块常量，可以减少拼写错误。

## 与组件库的关系

`shadcn` 自动启用 `icon`，组件内部使用同一套资源和缓存。业务可以直接使用原生图标，也可以把图标作为 Button/Menu 等组件的内容。渲染参数、缓存边界和性能注意事项见下一章。
