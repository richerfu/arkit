//! Native, theme-aware Markdown rendering.
//!
//! Parsing is intentionally separated from rendering. `pulldown-cmark`
//! streams CommonMark events into a compact ArkUI-oriented snapshot, and the
//! component memoizes that snapshot by source and parser options. Theme and
//! callback changes therefore do not repeat Markdown parsing.

use std::sync::Arc;

use arkit_prelude::*;
use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use smallvec::SmallVec;

use crate::theme::{spacing, typography, use_theme, Theme};

use super::ARKUI_BORDER_STYLE_SOLID;

const TEXT_DECORATION_NONE: i32 = 0;
const TEXT_DECORATION_UNDERLINE: i32 = 1;
const TEXT_DECORATION_LINE_THROUGH: i32 = 3;
const MAX_RENDER_DEPTH: usize = 32;
const LIST_MARKER_WIDTH: f32 = 28.0;
const TASK_MARKER_SIZE: f32 = 18.0;
const TASK_MARKER_ICON_SIZE: f32 = 14.0;
const TASK_MARKER_RADIUS: f32 = 4.0;
const ALIGN_CENTER: i32 = 4;

/// CommonMark extension switches used by [`Markdown`].
///
/// The default is the practical GitHub-flavored subset that has a native
/// renderer in this component. Smart punctuation remains opt-in because it
/// changes the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkdownOptions {
    pub tables: bool,
    pub task_lists: bool,
    pub strikethrough: bool,
    pub footnotes: bool,
    pub gfm_admonitions: bool,
    pub smart_punctuation: bool,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            tables: true,
            task_lists: true,
            strikethrough: true,
            footnotes: true,
            gfm_admonitions: true,
            smart_punctuation: false,
        }
    }
}

impl MarkdownOptions {
    fn parser_options(self) -> Options {
        let mut options = Options::empty();
        if self.tables {
            options.insert(Options::ENABLE_TABLES);
        }
        if self.task_lists {
            options.insert(Options::ENABLE_TASKLISTS);
        }
        if self.strikethrough {
            options.insert(Options::ENABLE_STRIKETHROUGH);
        }
        if self.footnotes {
            options.insert(Options::ENABLE_FOOTNOTES);
        }
        if self.gfm_admonitions {
            options.insert(Options::ENABLE_GFM);
        }
        if self.smart_punctuation {
            options.insert(Options::ENABLE_SMART_PUNCTUATION);
        }
        options
    }
}

/// Visual tokens for [`Markdown`].
///
/// Use [`MarkdownStyle::from_theme`] to derive a complete style and then
/// override individual fields. When the prop is omitted, the active shadcn
/// theme is resolved on every render.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkdownStyle {
    pub foreground: u32,
    pub muted_foreground: u32,
    pub link: u32,
    pub code_background: u32,
    pub quote_border: u32,
    pub table_border: u32,
    pub table_header_background: u32,
    pub task_checked_background: u32,
    pub task_checked_foreground: u32,
    pub task_unchecked_background: u32,
    pub task_border: u32,
    pub body_font_size: f32,
    pub body_line_height: f32,
    pub code_font_size: f32,
    pub code_line_height: f32,
    pub block_spacing: f32,
    pub list_item_spacing: f32,
    pub code_padding: f32,
    pub image_height: f32,
    pub radius: f32,
}

impl MarkdownStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            foreground: theme.colors.foreground,
            muted_foreground: theme.colors.muted_foreground,
            link: theme.colors.primary,
            code_background: theme.colors.muted,
            quote_border: theme.colors.border,
            table_border: theme.colors.border,
            table_header_background: theme.colors.muted,
            task_checked_background: theme.colors.primary,
            task_checked_foreground: theme.colors.primary_foreground,
            task_unchecked_background: theme.colors.background,
            task_border: theme.colors.border,
            body_font_size: typography::MD,
            body_line_height: 26.0,
            code_font_size: typography::SM,
            code_line_height: 20.0,
            block_spacing: spacing::LG,
            list_item_spacing: spacing::SM,
            code_padding: spacing::LG,
            image_height: 200.0,
            radius: theme.radii.md,
        }
    }
}

impl Default for MarkdownStyle {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

/// Props for [`Markdown`].
#[derive(Props, Clone, PartialEq)]
pub struct MarkdownProps {
    /// CommonMark source. The parsed snapshot is rebuilt only when this value
    /// or `options` changes.
    pub source: String,
    #[props(default)]
    pub options: MarkdownOptions,
    /// Complete style override. Omit it to track the active shadcn theme.
    #[props(default)]
    pub style: Option<MarkdownStyle>,
    /// Receives the destination of an activated Markdown link. Without a
    /// handler, links remain visually distinct but are inert.
    #[props(default)]
    pub on_link_click: Option<EventHandler<String>>,
}

/// Render CommonMark directly to native ArkUI nodes.
///
/// Raw HTML and metadata blocks are intentionally ignored: this is a native
/// renderer, not an HTML execution surface. CommonMark blocks, nested lists,
/// quotes, fenced code, links, images, tables, tasks, and footnotes are
/// rendered without using a WebView.
#[component]
pub fn Markdown(props: MarkdownProps) -> Element {
    let theme = use_theme();
    let style = props
        .style
        .unwrap_or_else(|| MarkdownStyle::from_theme(&theme));
    let source = props.source;
    let options = props.options;
    let document = use_memo(use_reactive((&source, &options), |(source, options)| {
        MarkdownDocument::parse(&source, options)
    }));
    let document = document.read();
    let blocks = render_blocks(&document.blocks, &style, props.on_link_click, "markdown", 0);

    rsx! {
        column {
            percent_width: 1.0,
            align_items: "start",
            {blocks.into_iter()}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MarkdownDocument {
    blocks: Vec<Block>,
}

impl MarkdownDocument {
    fn parse(source: &str, options: MarkdownOptions) -> Self {
        DocumentBuilder::new().parse(source, options)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Block {
    Paragraph(Inlines),
    Heading {
        level: u8,
        content: Inlines,
    },
    Quote {
        kind: Option<BlockQuoteKind>,
        blocks: Vec<Block>,
    },
    Code {
        language: Option<Arc<str>>,
        content: Arc<str>,
    },
    List {
        start: Option<u64>,
        items: Vec<ListItem>,
    },
    Table {
        alignments: Vec<Alignment>,
        header: Vec<Inlines>,
        rows: Vec<Vec<Inlines>>,
    },
    Footnote {
        label: Arc<str>,
        blocks: Vec<Block>,
    },
    Rule,
}

#[derive(Debug, Clone, PartialEq)]
struct ListItem {
    checked: Option<bool>,
    blocks: Vec<Block>,
}

type Inlines = SmallVec<[InlineNode; 4]>;

#[derive(Debug, Clone, PartialEq)]
enum InlineNode {
    Text {
        content: Arc<str>,
        style: InlineStyle,
        link: Option<Arc<str>>,
    },
    Image {
        source: Arc<str>,
        title: Arc<str>,
        alt: Arc<str>,
        link: Option<Arc<str>>,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct InlineStyle(u8);

impl InlineStyle {
    const STRONG: u8 = 1 << 0;
    const EMPHASIS: u8 = 1 << 1;
    const STRIKETHROUGH: u8 = 1 << 2;
    const CODE: u8 = 1 << 3;
    const SUPERSCRIPT: u8 = 1 << 4;
    const SUBSCRIPT: u8 = 1 << 5;

    fn with(mut self, flag: u8) -> Self {
        self.0 |= flag;
        self
    }

    fn contains(self, flag: u8) -> bool {
        self.0 & flag != 0
    }
}

#[derive(Debug)]
enum InlineDraft {
    Text {
        content: String,
        style: InlineStyle,
        link: Option<Arc<str>>,
    },
    Image {
        source: Arc<str>,
        title: Arc<str>,
        alt: Arc<str>,
        link: Option<Arc<str>>,
    },
}

#[derive(Debug, Default)]
struct InlineBuilder {
    nodes: SmallVec<[InlineDraft; 4]>,
}

impl InlineBuilder {
    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn push_text(&mut self, value: &str, style: InlineStyle, link: Option<Arc<str>>) {
        if value.is_empty() {
            return;
        }
        if let Some(InlineDraft::Text {
            content,
            style: previous_style,
            link: previous_link,
        }) = self.nodes.last_mut()
        {
            if *previous_style == style && *previous_link == link {
                content.push_str(value);
                return;
            }
        }
        self.nodes.push(InlineDraft::Text {
            content: value.to_owned(),
            style,
            link,
        });
    }

    fn push_image(
        &mut self,
        source: Arc<str>,
        title: Arc<str>,
        alt: Arc<str>,
        link: Option<Arc<str>>,
    ) {
        self.nodes.push(InlineDraft::Image {
            source,
            title,
            alt,
            link,
        });
    }

    fn finish(self) -> Inlines {
        self.nodes
            .into_iter()
            .map(|node| match node {
                InlineDraft::Text {
                    content,
                    style,
                    link,
                } => InlineNode::Text {
                    content: Arc::from(content),
                    style,
                    link,
                },
                InlineDraft::Image {
                    source,
                    title,
                    alt,
                    link,
                } => InlineNode::Image {
                    source,
                    title,
                    alt,
                    link,
                },
            })
            .collect()
    }
}

#[derive(Debug)]
struct ImageCapture {
    source: Arc<str>,
    title: Arc<str>,
    alt: String,
    link: Option<Arc<str>>,
}

#[derive(Debug)]
enum Frame {
    Root {
        blocks: Vec<Block>,
    },
    Paragraph(InlineBuilder),
    Heading {
        level: u8,
        content: InlineBuilder,
    },
    BlockQuote {
        kind: Option<BlockQuoteKind>,
        blocks: Vec<Block>,
    },
    Code {
        language: Option<Arc<str>>,
        content: String,
    },
    List {
        start: Option<u64>,
        items: Vec<ListItem>,
    },
    Item {
        checked: Option<bool>,
        blocks: Vec<Block>,
        inline: InlineBuilder,
    },
    Table {
        alignments: Vec<Alignment>,
        header: Vec<Inlines>,
        rows: Vec<Vec<Inlines>>,
    },
    TableHead {
        cells: Vec<Inlines>,
    },
    TableRow {
        cells: Vec<Inlines>,
    },
    TableCell(InlineBuilder),
    Footnote {
        label: Arc<str>,
        blocks: Vec<Block>,
    },
    Ignored,
}

struct DocumentBuilder {
    frames: Vec<Frame>,
    inline_style: InlineStyle,
    style_stack: Vec<InlineStyle>,
    current_link: Option<Arc<str>>,
    link_stack: Vec<Option<Arc<str>>>,
    image: Option<ImageCapture>,
}

impl DocumentBuilder {
    fn new() -> Self {
        Self {
            frames: vec![Frame::Root { blocks: Vec::new() }],
            inline_style: InlineStyle::default(),
            style_stack: Vec::new(),
            current_link: None,
            link_stack: Vec::new(),
            image: None,
        }
    }

    fn parse(mut self, source: &str, options: MarkdownOptions) -> MarkdownDocument {
        for event in Parser::new_ext(source, options.parser_options()) {
            self.consume(event);
        }
        debug_assert!(self.style_stack.is_empty());
        debug_assert!(self.link_stack.is_empty());
        debug_assert!(self.image.is_none());
        debug_assert_eq!(self.frames.len(), 1);
        let frame = self
            .frames
            .pop()
            .expect("document builder always owns a root frame");
        let Frame::Root { blocks } = frame else {
            unreachable!("pulldown-cmark guarantees balanced events");
        };
        MarkdownDocument { blocks }
    }

    fn consume(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.push_text(&text),
            Event::Code(code) => {
                let style = self.inline_style.with(InlineStyle::CODE);
                self.push_styled_text(&code, style);
            }
            Event::InlineMath(math) => {
                let value = format!("${math}$");
                let style = self.inline_style.with(InlineStyle::CODE);
                self.push_styled_text(&value, style);
            }
            Event::DisplayMath(math) => self.add_block(Block::Code {
                language: Some(Arc::from("math")),
                content: Arc::from(math.into_string()),
            }),
            Event::Html(_) | Event::InlineHtml(_) => {}
            Event::FootnoteReference(label) => {
                let value = format!("[{}]", label.as_ref());
                let style = self.inline_style.with(InlineStyle::SUPERSCRIPT);
                self.push_styled_text(&value, style);
            }
            Event::SoftBreak => self.push_text(" "),
            Event::HardBreak => self.push_text("\n"),
            Event::Rule => self.add_block(Block::Rule),
            Event::TaskListMarker(checked) => {
                if let Some(Frame::Item {
                    checked: item_checked,
                    ..
                }) = self
                    .frames
                    .iter_mut()
                    .rev()
                    .find(|frame| matches!(frame, Frame::Item { .. }))
                {
                    *item_checked = Some(checked);
                }
            }
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        if matches!(
            &tag,
            Tag::Paragraph
                | Tag::Heading { .. }
                | Tag::BlockQuote(_)
                | Tag::CodeBlock(_)
                | Tag::HtmlBlock
                | Tag::List(_)
                | Tag::FootnoteDefinition(_)
                | Tag::Table(_)
                | Tag::MetadataBlock(_)
                | Tag::DefinitionList
                | Tag::DefinitionListTitle
                | Tag::DefinitionListDefinition
        ) {
            self.flush_item_inline();
        }
        match tag {
            Tag::Paragraph => self.frames.push(Frame::Paragraph(InlineBuilder::default())),
            Tag::Heading { level, .. } => self.frames.push(Frame::Heading {
                level: heading_level(level),
                content: InlineBuilder::default(),
            }),
            Tag::BlockQuote(kind) => self.frames.push(Frame::BlockQuote {
                kind,
                blocks: Vec::new(),
            }),
            Tag::CodeBlock(kind) => self.frames.push(Frame::Code {
                language: code_language(kind),
                content: String::new(),
            }),
            Tag::HtmlBlock | Tag::MetadataBlock(_) => self.frames.push(Frame::Ignored),
            Tag::List(start) => self.frames.push(Frame::List {
                start,
                items: Vec::new(),
            }),
            Tag::Item => self.frames.push(Frame::Item {
                checked: None,
                blocks: Vec::new(),
                inline: InlineBuilder::default(),
            }),
            Tag::FootnoteDefinition(label) => self.frames.push(Frame::Footnote {
                label: Arc::from(label.into_string()),
                blocks: Vec::new(),
            }),
            Tag::Table(alignments) => self.frames.push(Frame::Table {
                alignments,
                header: Vec::new(),
                rows: Vec::new(),
            }),
            Tag::TableHead => self.frames.push(Frame::TableHead { cells: Vec::new() }),
            Tag::TableRow => self.frames.push(Frame::TableRow { cells: Vec::new() }),
            Tag::TableCell => self.frames.push(Frame::TableCell(InlineBuilder::default())),
            Tag::Emphasis => self.push_style(InlineStyle::EMPHASIS),
            Tag::Strong => self.push_style(InlineStyle::STRONG),
            Tag::Strikethrough => self.push_style(InlineStyle::STRIKETHROUGH),
            Tag::Superscript => self.push_style(InlineStyle::SUPERSCRIPT),
            Tag::Subscript => self.push_style(InlineStyle::SUBSCRIPT),
            Tag::Link { dest_url, .. } => {
                self.link_stack.push(self.current_link.take());
                self.current_link = Some(Arc::from(dest_url.into_string()));
            }
            Tag::Image {
                dest_url, title, ..
            } => {
                debug_assert!(self.image.is_none(), "Markdown images cannot nest");
                self.image = Some(ImageCapture {
                    source: Arc::from(dest_url.into_string()),
                    title: Arc::from(title.into_string()),
                    alt: String::new(),
                    link: self.current_link.clone(),
                });
            }
            Tag::DefinitionList | Tag::DefinitionListTitle | Tag::DefinitionListDefinition => {
                self.frames.push(Frame::Ignored)
            }
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                let Frame::Paragraph(content) = self.pop_frame() else {
                    unreachable!("paragraph end must close a paragraph frame");
                };
                self.add_block(Block::Paragraph(content.finish()));
            }
            TagEnd::Heading(_) => {
                let Frame::Heading { level, content } = self.pop_frame() else {
                    unreachable!("heading end must close a heading frame");
                };
                self.add_block(Block::Heading {
                    level,
                    content: content.finish(),
                });
            }
            TagEnd::BlockQuote(_) => {
                let Frame::BlockQuote { kind, blocks } = self.pop_frame() else {
                    unreachable!("quote end must close a quote frame");
                };
                self.add_block(Block::Quote { kind, blocks });
            }
            TagEnd::CodeBlock => {
                let Frame::Code { language, content } = self.pop_frame() else {
                    unreachable!("code end must close a code frame");
                };
                self.add_block(Block::Code {
                    language,
                    content: Arc::from(content),
                });
            }
            TagEnd::HtmlBlock | TagEnd::MetadataBlock(_) => self.pop_ignored(),
            TagEnd::List(_) => {
                let Frame::List { start, items } = self.pop_frame() else {
                    unreachable!("list end must close a list frame");
                };
                self.add_block(Block::List { start, items });
            }
            TagEnd::Item => {
                let Frame::Item {
                    checked,
                    mut blocks,
                    inline,
                } = self.pop_frame()
                else {
                    unreachable!("item end must close an item frame");
                };
                if !inline.is_empty() {
                    blocks.push(Block::Paragraph(inline.finish()));
                }
                let Some(Frame::List { items, .. }) = self.frames.last_mut() else {
                    unreachable!("list item must be owned by a list");
                };
                items.push(ListItem { checked, blocks });
            }
            TagEnd::FootnoteDefinition => {
                let Frame::Footnote { label, blocks } = self.pop_frame() else {
                    unreachable!("footnote end must close a footnote frame");
                };
                self.add_block(Block::Footnote { label, blocks });
            }
            TagEnd::Table => {
                let Frame::Table {
                    alignments,
                    header,
                    rows,
                } = self.pop_frame()
                else {
                    unreachable!("table end must close a table frame");
                };
                self.add_block(Block::Table {
                    alignments,
                    header,
                    rows,
                });
            }
            TagEnd::TableHead => {
                let Frame::TableHead { cells } = self.pop_frame() else {
                    unreachable!("table head end must close a table head frame");
                };
                let Some(Frame::Table {
                    header: table_header,
                    ..
                }) = self.frames.last_mut()
                else {
                    unreachable!("table head must be owned by a table");
                };
                *table_header = cells;
            }
            TagEnd::TableRow => {
                let Frame::TableRow { cells } = self.pop_frame() else {
                    unreachable!("table row end must close a table row frame");
                };
                match self.frames.last_mut() {
                    Some(Frame::Table { rows, .. }) => rows.push(cells),
                    _ => unreachable!("table body row must be owned by a table"),
                }
            }
            TagEnd::TableCell => {
                let Frame::TableCell(content) = self.pop_frame() else {
                    unreachable!("table cell end must close a table cell frame");
                };
                let content = content.finish();
                match self.frames.last_mut() {
                    Some(Frame::TableHead { cells }) | Some(Frame::TableRow { cells }) => {
                        cells.push(content);
                    }
                    _ => unreachable!("table cell must be owned by a table row or table head"),
                }
            }
            TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript => {
                self.inline_style = self
                    .style_stack
                    .pop()
                    .expect("inline style events are balanced");
            }
            TagEnd::Link => {
                self.current_link = self.link_stack.pop().expect("link events are balanced");
            }
            TagEnd::Image => {
                let image = self.image.take().expect("image events are balanced");
                self.current_inlines().push_image(
                    image.source,
                    image.title,
                    Arc::from(image.alt),
                    image.link,
                );
            }
            TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition => self.pop_ignored(),
        }
    }

    fn pop_frame(&mut self) -> Frame {
        self.frames
            .pop()
            .expect("pulldown-cmark emits balanced container events")
    }

    fn pop_ignored(&mut self) {
        let frame = self.pop_frame();
        debug_assert!(matches!(frame, Frame::Ignored));
    }

    fn push_style(&mut self, flag: u8) {
        self.style_stack.push(self.inline_style);
        self.inline_style = self.inline_style.with(flag);
    }

    fn push_text(&mut self, value: &str) {
        if let Some(Frame::Code { content, .. }) = self.frames.last_mut() {
            content.push_str(value);
            return;
        }
        if let Some(image) = self.image.as_mut() {
            image.alt.push_str(value);
            return;
        }
        self.push_styled_text(value, self.inline_style);
    }

    fn push_styled_text(&mut self, value: &str, style: InlineStyle) {
        let link = self.current_link.clone();
        self.current_inlines().push_text(value, style, link);
    }

    fn current_inlines(&mut self) -> &mut InlineBuilder {
        match self.frames.last_mut() {
            Some(Frame::Paragraph(content))
            | Some(Frame::TableCell(content))
            | Some(Frame::Heading { content, .. })
            | Some(Frame::Item {
                inline: content, ..
            }) => content,
            _ => unreachable!("inline content must be owned by an inline container"),
        }
    }

    fn add_block(&mut self, block: Block) {
        match self.frames.last_mut() {
            Some(Frame::Root { blocks })
            | Some(Frame::BlockQuote { blocks, .. })
            | Some(Frame::Item { blocks, .. })
            | Some(Frame::Footnote { blocks, .. }) => blocks.push(block),
            Some(Frame::Ignored) => {}
            _ => unreachable!("block must be owned by a block container"),
        }
    }

    fn flush_item_inline(&mut self) {
        let Some(Frame::Item { blocks, inline, .. }) = self.frames.last_mut() else {
            return;
        };
        if inline.is_empty() {
            return;
        }
        let content = std::mem::take(inline).finish();
        blocks.push(Block::Paragraph(content));
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn code_language(kind: CodeBlockKind<'_>) -> Option<Arc<str>> {
    match kind {
        CodeBlockKind::Indented => None,
        CodeBlockKind::Fenced(info) => info
            .split_whitespace()
            .next()
            .filter(|language| !language.is_empty())
            .map(Arc::from),
    }
}

#[derive(Clone, Copy)]
struct TextMetrics {
    font_size: f32,
    line_height: f32,
    font_weight: i32,
    color: u32,
}

#[derive(Clone, Copy)]
enum InlineAlignment {
    Start,
    Center,
    End,
}

impl InlineAlignment {
    fn justify_content(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

fn render_blocks(
    blocks: &[Block],
    style: &MarkdownStyle,
    on_link_click: Option<EventHandler<String>>,
    key_prefix: &str,
    depth: usize,
) -> Vec<Element> {
    blocks
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let key = format!("{key_prefix}-{index}");
            let content = if depth >= MAX_RENDER_DEPTH {
                render_depth_limit(block, style)
            } else {
                render_block(block, style, on_link_click, key.as_str(), depth)
            };
            rsx! {
                column {
                    key: "{key}",
                    percent_width: 1.0,
                    align_items: "start",
                    margin_top: if index == 0 { 0.0 } else { style.block_spacing },
                    {content}
                }
            }
        })
        .collect()
}

fn render_block(
    block: &Block,
    style: &MarkdownStyle,
    on_link_click: Option<EventHandler<String>>,
    key_prefix: &str,
    depth: usize,
) -> Element {
    match block {
        Block::Paragraph(content) => render_inlines(
            content,
            TextMetrics {
                font_size: style.body_font_size,
                line_height: style.body_line_height,
                font_weight: 400,
                color: style.foreground,
            },
            style,
            on_link_click,
            key_prefix,
            InlineAlignment::Start,
        ),
        Block::Heading { level, content } => {
            let (font_size, line_height, font_weight) = heading_metrics(*level);
            render_inlines(
                content,
                TextMetrics {
                    font_size,
                    line_height,
                    font_weight,
                    color: style.foreground,
                },
                style,
                on_link_click,
                key_prefix,
                InlineAlignment::Start,
            )
        }
        Block::Quote { kind, blocks } => {
            let nested_key = format!("{key_prefix}-quote");
            let nested = render_blocks(blocks, style, on_link_click, &nested_key, depth + 1);
            let label = kind.and_then(admonition_label);
            rsx! {
                row {
                    percent_width: 1.0,
                    align_items: "start",
                    row {
                        width: 3.0,
                        align_self: "stretch",
                        background_color: style.quote_border,
                        border_radius: 1.5,
                    }
                    column {
                        layout_weight: 1.0,
                        margin_left: spacing::LG,
                        align_items: "start",
                        if let Some(label) = label {
                            text {
                                content: label,
                                font_size: typography::SM,
                                font_weight: 700_i32,
                                font_color: style.muted_foreground,
                                line_height: 18.0,
                                margin_bottom: spacing::XS,
                            }
                        }
                        {nested.into_iter()}
                    }
                }
            }
        }
        Block::Code { language, content } => {
            let language = language.as_ref().map(|value| value.to_string());
            let content = content.to_string();
            rsx! {
                column {
                    percent_width: 1.0,
                    align_items: "start",
                    padding_top: style.code_padding,
                    padding_right: style.code_padding,
                    padding_bottom: style.code_padding,
                    padding_left: style.code_padding,
                    background_color: style.code_background,
                    border_radius: style.radius,
                    clip: true,
                    if let Some(language) = language {
                        text {
                            content: language,
                            font_size: 11.0,
                            font_weight: 600_i32,
                            font_color: style.muted_foreground,
                            line_height: 16.0,
                            margin_bottom: spacing::SM,
                        }
                    }
                    text {
                        content,
                        percent_width: 1.0,
                        font_size: style.code_font_size,
                        font_family: "monospace",
                        font_color: style.foreground,
                        line_height: style.code_line_height,
                        text_align: 0_i32,
                    }
                }
            }
        }
        Block::List { start, items } => {
            let rows = items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let marker = render_list_marker(*start, index, item.checked, style);
                    let nested_key = format!("{key_prefix}-item-{index}");
                    let nested =
                        render_blocks(&item.blocks, style, on_link_click, &nested_key, depth + 1);
                    rsx! {
                        row {
                            key: "{nested_key}",
                            percent_width: 1.0,
                            align_items: "start",
                            margin_top: if index == 0 { 0.0 } else { style.list_item_spacing },
                            {marker}
                            column {
                                layout_weight: 1.0,
                                align_items: "start",
                                {nested.into_iter()}
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();
            rsx! {
                column {
                    percent_width: 1.0,
                    align_items: "start",
                    {rows.into_iter()}
                }
            }
        }
        Block::Table {
            alignments,
            header,
            rows,
        } => render_table(alignments, header, rows, style, on_link_click, key_prefix),
        Block::Footnote { label, blocks } => {
            let nested_key = format!("{key_prefix}-footnote");
            let nested = render_blocks(blocks, style, on_link_click, &nested_key, depth + 1);
            rsx! {
                row {
                    percent_width: 1.0,
                    align_items: "start",
                    text {
                        content: format!("[{}]", label.as_ref()),
                        width: 36.0,
                        font_size: typography::SM,
                        font_weight: 600_i32,
                        font_color: style.muted_foreground,
                        line_height: style.body_line_height,
                    }
                    column {
                        layout_weight: 1.0,
                        align_items: "start",
                        {nested.into_iter()}
                    }
                }
            }
        }
        Block::Rule => rsx! {
            row {
                percent_width: 1.0,
                height: 1.0,
                background_color: style.table_border,
            }
        },
    }
}

fn render_table(
    alignments: &[Alignment],
    header: &[Inlines],
    rows: &[Vec<Inlines>],
    style: &MarkdownStyle,
    on_link_click: Option<EventHandler<String>>,
    key_prefix: &str,
) -> Element {
    let header_cells = header
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            let key = format!("{key_prefix}-head-{index}");
            let content = render_table_cell(
                cell,
                alignments.get(index).copied().unwrap_or(Alignment::None),
                true,
                style,
                on_link_click,
                &key,
            );
            rsx! {
                row {
                    key: "{key}",
                    layout_weight: 1.0,
                    padding_top: spacing::SM,
                    padding_right: spacing::SM,
                    padding_bottom: spacing::SM,
                    padding_left: spacing::SM,
                    background_color: style.table_header_background,
                    {content}
                }
            }
        })
        .collect::<Vec<_>>();
    let body_rows = rows
        .iter()
        .enumerate()
        .map(|(row_index, row)| {
            let row_key = format!("{key_prefix}-row-{row_index}");
            let cells = row
                .iter()
                .enumerate()
                .map(|(cell_index, cell)| {
                    let cell_key = format!("{row_key}-cell-{cell_index}");
                    let content = render_table_cell(
                        cell,
                        alignments
                            .get(cell_index)
                            .copied()
                            .unwrap_or(Alignment::None),
                        false,
                        style,
                        on_link_click,
                        &cell_key,
                    );
                    rsx! {
                        row {
                            key: "{cell_key}",
                            layout_weight: 1.0,
                            padding_top: spacing::SM,
                            padding_right: spacing::SM,
                            padding_bottom: spacing::SM,
                            padding_left: spacing::SM,
                            {content}
                        }
                    }
                })
                .collect::<Vec<_>>();
            rsx! {
                row {
                    key: "{row_key}",
                    percent_width: 1.0,
                    align_items: "start",
                    border_width: if row_index + 1 == rows.len() { "0,0,0,0" } else { "0,0,1,0" },
                    border_color: style.table_border,
                    {cells.into_iter()}
                }
            }
        })
        .collect::<Vec<_>>();

    rsx! {
        column {
            percent_width: 1.0,
            border_width: 1.0,
            border_color: style.table_border,
            border_radius: style.radius,
            clip: true,
            if !header_cells.is_empty() {
                row {
                    percent_width: 1.0,
                    align_items: "start",
                    border_width: if body_rows.is_empty() { "0,0,0,0" } else { "0,0,1,0" },
                    border_color: style.table_border,
                    {header_cells.into_iter()}
                }
            }
            {body_rows.into_iter()}
        }
    }
}

fn render_table_cell(
    content: &[InlineNode],
    alignment: Alignment,
    header: bool,
    style: &MarkdownStyle,
    on_link_click: Option<EventHandler<String>>,
    key_prefix: &str,
) -> Element {
    render_inlines(
        content,
        TextMetrics {
            font_size: typography::SM,
            line_height: 20.0,
            font_weight: if header { 600 } else { 400 },
            color: style.foreground,
        },
        style,
        on_link_click,
        key_prefix,
        match alignment {
            Alignment::Center => InlineAlignment::Center,
            Alignment::Right => InlineAlignment::End,
            Alignment::None | Alignment::Left => InlineAlignment::Start,
        },
    )
}

fn render_inlines(
    content: &[InlineNode],
    metrics: TextMetrics,
    markdown_style: &MarkdownStyle,
    on_link_click: Option<EventHandler<String>>,
    key_prefix: &str,
    alignment: InlineAlignment,
) -> Element {
    if content.is_empty() {
        return rsx! { row { height: metrics.line_height } };
    }

    let nodes = content
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let key = format!("{key_prefix}-inline-{index}");
            match node {
                InlineNode::Text {
                    content,
                    style,
                    link,
                } => render_text_span(
                    content,
                    *style,
                    link.as_ref(),
                    metrics,
                    markdown_style,
                    on_link_click,
                    key,
                ),
                InlineNode::Image {
                    source,
                    title,
                    alt,
                    link,
                } => render_image_span(
                    source,
                    title,
                    alt,
                    link.as_ref(),
                    markdown_style,
                    on_link_click,
                    key,
                ),
            }
        })
        .collect::<Vec<_>>();

    rsx! {
        flex {
            percent_width: 1.0,
            flex_direction: "row",
            flex_wrap: "wrap",
            align_items: "baseline",
            justify_content: alignment.justify_content(),
            {nodes.into_iter()}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_text_span(
    content: &Arc<str>,
    inline_style: InlineStyle,
    link: Option<&Arc<str>>,
    metrics: TextMetrics,
    markdown_style: &MarkdownStyle,
    on_link_click: Option<EventHandler<String>>,
    key: String,
) -> Element {
    let content = content.to_string();
    let is_code = inline_style.contains(InlineStyle::CODE);
    let is_link = link.is_some();
    let font_size = if inline_style.contains(InlineStyle::SUPERSCRIPT)
        || inline_style.contains(InlineStyle::SUBSCRIPT)
    {
        metrics.font_size * 0.8
    } else {
        metrics.font_size
    };
    let font_weight = if inline_style.contains(InlineStyle::STRONG) {
        700
    } else {
        metrics.font_weight
    };
    let font_style = i32::from(inline_style.contains(InlineStyle::EMPHASIS));
    let text_decoration = if inline_style.contains(InlineStyle::STRIKETHROUGH) {
        TEXT_DECORATION_LINE_THROUGH
    } else if is_link {
        TEXT_DECORATION_UNDERLINE
    } else {
        TEXT_DECORATION_NONE
    };
    let color = if is_link {
        markdown_style.link
    } else {
        metrics.color
    };

    match (is_code, link, on_link_click) {
        (true, Some(link), Some(handler)) => {
            let destination = link.to_string();
            rsx! {
                text {
                    key: "{key}",
                    content,
                    font_size,
                    font_family: "monospace",
                    font_weight,
                    font_style,
                    font_color: color,
                    line_height: metrics.line_height,
                    text_decoration,
                    background_color: markdown_style.code_background,
                    padding_top: 2.0,
                    padding_right: 4.0,
                    padding_bottom: 2.0,
                    padding_left: 4.0,
                    border_radius: 4.0,
                    onclick: move |_| handler.call(destination.clone()),
                }
            }
        }
        (true, _, _) => rsx! {
            text {
                key: "{key}",
                content,
                font_size,
                font_family: "monospace",
                font_weight,
                font_style,
                font_color: color,
                line_height: metrics.line_height,
                text_decoration,
                background_color: markdown_style.code_background,
                padding_top: 2.0,
                padding_right: 4.0,
                padding_bottom: 2.0,
                padding_left: 4.0,
                border_radius: 4.0,
            }
        },
        (false, Some(link), Some(handler)) => {
            let destination = link.to_string();
            rsx! {
                text {
                    key: "{key}",
                    content,
                    font_size,
                    font_weight,
                    font_style,
                    font_color: color,
                    line_height: metrics.line_height,
                    text_decoration,
                    onclick: move |_| handler.call(destination.clone()),
                }
            }
        }
        (false, _, _) => rsx! {
            text {
                key: "{key}",
                content,
                font_size,
                font_weight,
                font_style,
                font_color: color,
                line_height: metrics.line_height,
                text_decoration,
            }
        },
    }
}

fn render_image_span(
    source: &Arc<str>,
    _title: &Arc<str>,
    _alt: &Arc<str>,
    link: Option<&Arc<str>>,
    style: &MarkdownStyle,
    on_link_click: Option<EventHandler<String>>,
    key: String,
) -> Element {
    let source = source.to_string();
    if let (Some(link), Some(handler)) = (link, on_link_click) {
        let destination = link.to_string();
        rsx! {
            column {
                key: "{key}",
                percent_width: 1.0,
                height: style.image_height,
                border_radius: style.radius,
                clip: true,
                onclick: move |_| handler.call(destination.clone()),
                image {
                    src: source,
                    percent_width: 1.0,
                    percent_height: 1.0,
                    object_fit: 1_i32,
                }
            }
        }
    } else {
        rsx! {
            column {
                key: "{key}",
                percent_width: 1.0,
                height: style.image_height,
                border_radius: style.radius,
                clip: true,
                image {
                    src: source,
                    percent_width: 1.0,
                    percent_height: 1.0,
                    object_fit: 1_i32,
                }
            }
        }
    }
}

fn render_depth_limit(block: &Block, style: &MarkdownStyle) -> Element {
    rsx! {
        text {
            content: block_plain_text(block),
            percent_width: 1.0,
            font_size: style.body_font_size,
            font_color: style.foreground,
            line_height: style.body_line_height,
        }
    }
}

fn block_plain_text(block: &Block) -> String {
    let mut output = String::new();
    let mut pending = vec![block];
    while let Some(block) = pending.pop() {
        match block {
            Block::Paragraph(content) | Block::Heading { content, .. } => {
                collect_inline_text(content, &mut output)
            }
            Block::Quote { blocks, .. } | Block::Footnote { blocks, .. } => {
                for child in blocks.iter().rev() {
                    pending.push(child);
                }
                output.push('\n');
            }
            Block::Code { content, .. } => output.push_str(content),
            Block::List { items, .. } => {
                for item in items.iter().rev() {
                    for child in item.blocks.iter().rev() {
                        pending.push(child);
                    }
                }
                output.push('\n');
            }
            Block::Table { header, rows, .. } => {
                for cell in header.iter().chain(rows.iter().flatten()) {
                    collect_inline_text(cell, &mut output);
                    output.push(' ');
                }
            }
            Block::Rule => output.push_str("---"),
        }
    }
    output
}

fn collect_inline_text(content: &[InlineNode], output: &mut String) {
    for node in content {
        match node {
            InlineNode::Text { content, .. } => output.push_str(content),
            InlineNode::Image { alt, .. } => output.push_str(alt),
        }
    }
}

fn heading_metrics(level: u8) -> (f32, f32, i32) {
    match level {
        1 => (32.0, 38.0, 700),
        2 => (26.0, 32.0, 700),
        3 => (22.0, 28.0, 600),
        4 => (18.0, 24.0, 600),
        5 => (16.0, 22.0, 600),
        _ => (14.0, 20.0, 600),
    }
}

fn admonition_label(kind: BlockQuoteKind) -> Option<String> {
    Some(
        match kind {
            BlockQuoteKind::Note => "NOTE",
            BlockQuoteKind::Tip => "TIP",
            BlockQuoteKind::Important => "IMPORTANT",
            BlockQuoteKind::Warning => "WARNING",
            BlockQuoteKind::Caution => "CAUTION",
        }
        .to_owned(),
    )
}

fn render_list_marker(
    start: Option<u64>,
    index: usize,
    checked: Option<bool>,
    style: &MarkdownStyle,
) -> Element {
    if let Some(checked) = checked {
        let border_color = if checked {
            style.task_checked_background
        } else {
            style.task_border
        };
        let background_color = if checked {
            style.task_checked_background
        } else {
            style.task_unchecked_background
        };
        return rsx! {
            row {
                width: LIST_MARKER_WIDTH,
                height: style.body_line_height,
                align_items: "center",
                justify_content: "start",
                stack {
                    width: TASK_MARKER_SIZE,
                    height: TASK_MARKER_SIZE,
                    alignment: ALIGN_CENTER,
                    border_width: 1.0,
                    border_style: ARKUI_BORDER_STYLE_SOLID,
                    border_color,
                    border_radius: TASK_MARKER_RADIUS,
                    background_color,
                    clip: true,
                    hit_test_behavior: 2_i32,
                    if checked {
                        {arkit_icon::icon(
                            "check",
                            TASK_MARKER_ICON_SIZE,
                            style.task_checked_foreground,
                        )}
                    }
                }
            }
        };
    }

    let marker = start
        .map(|start| format!("{}.", start.saturating_add(index as u64)))
        .unwrap_or_else(|| "•".to_owned());
    rsx! {
        text {
            content: marker,
            width: LIST_MARKER_WIDTH,
            font_size: style.body_font_size,
            font_color: style.muted_foreground,
            line_height: style.body_line_height,
            text_align: 0_i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_of(content: &[InlineNode]) -> String {
        let mut text = String::new();
        collect_inline_text(content, &mut text);
        text
    }

    #[test]
    fn parses_commonmark_and_gfm_blocks() {
        let source = concat!(
            "# Heading\n\n",
            "A **strong** and [linked](https://example.com) paragraph.\n\n",
            "- [x] parsed once\n",
            "  - nested item\n",
            "- [ ] rendered natively\n\n",
            "| Name | Value |\n",
            "| --- | ---: |\n",
            "| mode | fast |\n\n",
            "```rust\nlet answer = 42;\n```\n",
        );
        let document = MarkdownDocument::parse(source, MarkdownOptions::default());

        assert!(matches!(
            document.blocks[0],
            Block::Heading { level: 1, .. }
        ));
        let Block::Paragraph(paragraph) = &document.blocks[1] else {
            panic!("expected paragraph");
        };
        assert_eq!(text_of(paragraph), "A strong and linked paragraph.");
        assert_eq!(paragraph.len(), 5, "style boundaries stay compact");

        let Block::List { items, .. } = &document.blocks[2] else {
            panic!("expected task list");
        };
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].checked, Some(true));
        assert_eq!(items[1].checked, Some(false));
        assert!(matches!(items[0].blocks[0], Block::Paragraph(_)));
        assert!(matches!(items[0].blocks[1], Block::List { .. }));

        let Block::Table {
            header,
            rows,
            alignments,
        } = &document.blocks[3]
        else {
            panic!("expected table");
        };
        assert_eq!(header.len(), 2);
        assert_eq!(rows.len(), 1);
        assert_eq!(alignments[1], Alignment::Right);

        let Block::Code { language, content } = &document.blocks[4] else {
            panic!("expected code block");
        };
        assert_eq!(language.as_deref(), Some("rust"));
        assert_eq!(content.as_ref(), "let answer = 42;\n");
    }

    #[test]
    fn merges_adjacent_text_with_the_same_presentation() {
        let document = MarkdownDocument::parse(
            "plain &amp; text with a soft\nbreak",
            MarkdownOptions::default(),
        );
        let Block::Paragraph(content) = &document.blocks[0] else {
            panic!("expected paragraph");
        };

        assert_eq!(content.len(), 1);
        assert_eq!(text_of(content), "plain & text with a soft break");
    }

    #[test]
    fn extension_options_change_the_parse_snapshot() {
        let source = "| A | B |\n| - | - |\n| 1 | 2 |";
        let enabled = MarkdownDocument::parse(source, MarkdownOptions::default());
        let disabled = MarkdownDocument::parse(
            source,
            MarkdownOptions {
                tables: false,
                ..MarkdownOptions::default()
            },
        );

        assert!(matches!(enabled.blocks[0], Block::Table { .. }));
        assert!(disabled
            .blocks
            .iter()
            .all(|block| !matches!(block, Block::Table { .. })));
    }

    #[test]
    fn captures_links_images_and_footnotes_without_html() {
        let source = concat!(
            "[![alt text](image.png \"title\")](https://example.com) note[^n].\n\n",
            "[^n]: Footnote body.\n\n",
            "<script>ignored()</script>\n",
        );
        let document = MarkdownDocument::parse(source, MarkdownOptions::default());
        let Block::Paragraph(content) = &document.blocks[0] else {
            panic!("expected paragraph");
        };
        let InlineNode::Image {
            source,
            title,
            alt,
            link,
        } = &content[0]
        else {
            panic!("expected image");
        };
        assert_eq!(source.as_ref(), "image.png");
        assert_eq!(title.as_ref(), "title");
        assert_eq!(alt.as_ref(), "alt text");
        assert_eq!(link.as_deref(), Some("https://example.com"));
        assert!(document
            .blocks
            .iter()
            .any(|block| matches!(block, Block::Footnote { .. })));
        assert!(!document
            .blocks
            .iter()
            .any(|block| block_plain_text(block).contains("script")));
    }

    #[test]
    fn large_plain_documents_keep_one_inline_node_per_paragraph() {
        let mut source = String::new();
        for index in 0..1_000 {
            source.push_str(&format!("Paragraph {index} with ordinary text.\n\n"));
        }
        let document = MarkdownDocument::parse(&source, MarkdownOptions::default());

        assert_eq!(document.blocks.len(), 1_000);
        assert!(document
            .blocks
            .iter()
            .all(|block| { matches!(block, Block::Paragraph(content) if content.len() == 1) }));
    }
}
