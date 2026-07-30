---
title: Markdown
description: "原生渲染 Markdown（CommonMark / GFM），可选代码高亮。"
---

# Markdown

用原生节点渲染 CommonMark / GFM。需要围栏高亮时再打开 `code` feature。

## 性能模型

- 使用 `pulldown-cmark` 的 pull parser 流式消费事件，不先生成 HTML 或通用 DOM。
- 解析结果会压缩为面向 ArkUI 的 block/inline 快照；相邻且样式相同的文本会合并，减少原生节点数。
- `source` 与 `options` 未变化时复用 Dioxus memo。主题切换和链接回调更新只重绘，不重复解析 Markdown。
- 大段普通文本通常对应一个 native `text`；只有强调、代码和链接等真实样式边界才拆分节点。
- 启用 `code` 后，已知语言的围栏块在渲染期走 tree-sitter；未知语言或失败时回退普通等宽。

组件不主动创建滚动容器。页面应在外层放置 `scroll`，让 Markdown 可以参与已有页面布局。

## Props

| Prop                     | 类型                               | 说明                                                        |
| ------------------------ | ---------------------------------- | ----------------------------------------------------------- |
| `source`                 | `String`                           | Markdown 源文本                                             |
| `options`                | `MarkdownOptions`                  | GFM 扩展开关，默认启用表格、任务、删除线、脚注和 admonition |
| `style`                  | `Option<MarkdownStyle>`            | 完整样式覆盖；缺省时实时读取当前 shadcn theme               |
| `admonition_labels`      | `Option<MarkdownAdmonitionLabels>` | admonition 标题；默认随 i18n locale 切换                    |
| `show_admonition_labels` | `bool`                             | 是否显示 admonition 标题，默认 `true`                       |
| `on_link_click`          | `Option<EventHandler<String>>`     | 链接激活回调；未提供时链接保留样式但不执行跳转              |

默认标题支持 `en-US`/`zh-CN` 响应式切换。也可通过
`MarkdownAdmonitionLabels::new(...)` 固定覆盖 NOTE、TIP、IMPORTANT、
WARNING 和 CAUTION 文案；设置 `show_admonition_labels: false` 可完全隐藏标题。

`MarkdownStyle::from_theme(&theme)` 可生成完整样式，再覆盖颜色、正文/代码字号、块间距、代码 padding、图片高度或任务列表 marker 的选中/未选中配色。

```rust
let theme = arkit::shadcn::theme::use_theme();
let mut style = MarkdownStyle::from_theme(&theme);
style.body_font_size = 17.0;
style.image_height = 240.0;

rsx! { Markdown { source, style: Some(style) } }
```

## 代码高亮（依赖 `code`）

高亮能力属于 **`code` feature**（独立 `Code` 组件、registry、tree-sitter 依赖），不绑死在 Markdown 上。

| 项                                | 说明                                                      |
| --------------------------------- | --------------------------------------------------------- |
| `MarkdownOptions::code_highlight` | 仅当启用 `code` 时字段存在，默认 `true`                   |
| `MarkdownStyle::code_highlight`   | `CodeHighlightPalette`；`from_theme` 按 light/dark 选预设 |
| 渲染                              | 有 `code` 时委托给 `Code` 组件；否则纯等宽                |

内置语言、自定义 `register_language`、palette 细节见 [Code](../code/)。

关闭某一篇文档的高亮（需已启用 `code`）：

```rust
Markdown {
    source,
    options: MarkdownOptions {
        code_highlight: false,
        ..MarkdownOptions::default()
    },
}
```

## 扩展与安全边界

`MarkdownOptions` 分别控制 `tables`、`task_lists`、`strikethrough`、`footnotes`、`gfm_admonitions`、`smart_punctuation`，以及在启用 `code` 时的 `code_highlight`。智能标点会改变展示文本，因此默认关闭。

原始 HTML 和 metadata block 会被忽略。`Markdown` 是原生内容组件，不是 HTML 执行环境；需要完整 HTML/CSS/JavaScript 语义时应使用 WebView。
