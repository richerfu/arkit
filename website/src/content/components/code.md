---
title: Code
description: "独立的语法高亮代码块，底层用 tree-sitter。"
---

# Code

单独的代码高亮块，底层是 tree-sitter。可以脱离 Markdown 使用，也可以给 Markdown 围栏复用。

## Feature 关系

| Feature              | 作用                                  |
| -------------------- | ------------------------------------- |
| `code`               | `Code` 组件、高亮引擎、语言注册 API   |
| `markdown`           | CommonMark 文档渲染（围栏默认同等宽） |
| `markdown` + `code`  | Markdown 围栏块走 `Code` 管线         |
| `markdown-highlight` | 便捷别名 = `markdown` + `code`        |

## Props

| Prop        | 类型                | 说明                                            |
| ----------- | ------------------- | ----------------------------------------------- |
| `source`    | `String`            | 源码文本                                        |
| `language`  | `Option<String>`    | 语言别名（如 `rust`、`js`）；未知则回退普通等宽 |
| `highlight` | `bool`              | 是否尝试 tree-sitter 高亮，默认 `true`          |
| `style`     | `Option<CodeStyle>` | 完整样式；缺省跟 shadcn theme                   |

`CodeStyle::from_theme` 可覆盖背景、字号、行高、padding、圆角、是否显示 language label，以及 `CodeHighlightPalette` 令牌色。

## 内置语言与注册

内置别名：`rust`/`rs`，`javascript`/`js`/`jsx`，`typescript`/`ts`，`tsx`，`python`/`py`，`json`，`bash`/`sh`/`shell`/`zsh`，`go`/`golang`，`c`/`h`。

自定义 grammar 与 `Markdown` 共用同一全局 registry：

```rust
use arkit::shadcn::components::register_language;

register_language(
    &["toml"],
    tree_sitter_toml::LANGUAGE.into(),
    "toml",
    tree_sitter_toml::HIGHLIGHTS_QUERY,
    "",
    "",
)?;
```

| API                                               | 说明                    |
| ------------------------------------------------- | ----------------------- |
| `register_language`                               | Language + queries 安装 |
| `register_highlight_configuration`                | 已构建 config 安装      |
| `unregister_language` / `reset_language_registry` | 删除别名 / 恢复内置     |
| `highlight_code`                                  | 无 UI，只返回着色 span  |
| `components::code_highlight`                      | 子模块入口              |

grammar 需与 arkit 的 `tree-sitter` 主版本一致（当前 0.25）；capture 对齐 `HIGHLIGHT_NAMES` 才能套用 palette。
