---
title: Markdown
description: "高性能原生 CommonMark/GFM 渲染。"
---

# Markdown

`Markdown` 把 CommonMark/GFM 内容直接渲染为原生 ArkUI 节点，不经过 HTML 或 WebView。默认支持标题、段落、强调、链接、图片、引用、嵌套列表、代码块、分隔线、表格、任务列表和脚注。

Markdown 解析器及其辅助数据结构由独立的 `markdown` feature 引入；只启用 `shadcn` 不会链接这些依赖。

```toml
[dependencies]
arkit = { version = "*", features = ["markdown"] }
```

```rust
Markdown {
    source: r#"
# Release notes

- [x] Parse once
- [x] Render natively

See the [migration guide](https://example.com).
"#.to_string(),
    on_link_click: move |url| open_url(url),
}
```

## 性能模型

- 使用 `pulldown-cmark` 的 pull parser 流式消费事件，不先生成 HTML 或通用 DOM。
- 解析结果会压缩为面向 ArkUI 的 block/inline 快照；相邻且样式相同的文本会合并，减少原生节点数。
- `source` 与 `options` 未变化时复用 Dioxus memo。主题切换和链接回调更新只重绘，不重复解析 Markdown。
- 大段普通文本通常对应一个 native `text`；只有强调、代码和链接等真实样式边界才拆分节点。

组件不主动创建滚动容器。页面应在外层放置 `scroll`，让 Markdown 可以参与已有页面布局。

## Props

| Prop            | 类型                           | 说明                                                        |
| --------------- | ------------------------------ | ----------------------------------------------------------- |
| `source`        | `String`                       | Markdown 源文本                                             |
| `options`       | `MarkdownOptions`              | GFM 扩展开关，默认启用表格、任务、删除线、脚注和 admonition |
| `style`         | `Option<MarkdownStyle>`        | 完整样式覆盖；缺省时实时读取当前 shadcn theme               |
| `on_link_click` | `Option<EventHandler<String>>` | 链接激活回调；未提供时链接保留样式但不执行跳转              |

`MarkdownStyle::from_theme(&theme)` 可生成完整样式，再覆盖颜色、正文/代码字号、块间距、代码 padding、图片高度或任务列表 marker 的选中/未选中配色。

```rust
let theme = arkit::shadcn::theme::use_theme();
let mut style = MarkdownStyle::from_theme(&theme);
style.body_font_size = 17.0;
style.image_height = 240.0;

rsx! { Markdown { source, style: Some(style) } }
```

## 扩展与安全边界

`MarkdownOptions` 分别控制 `tables`、`task_lists`、`strikethrough`、`footnotes`、`gfm_admonitions` 和 `smart_punctuation`。智能标点会改变展示文本，因此默认关闭。

原始 HTML 和 metadata block 会被忽略。`Markdown` 是原生内容组件，不是 HTML 执行环境；需要完整 HTML/CSS/JavaScript 语义时应使用 WebView。
