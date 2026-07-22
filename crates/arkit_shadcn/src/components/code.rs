//! Standalone syntax-highlighted code block.
//!
//! Enabled via the `code` feature. Renders source as native ArkUI monospace
//! text, optionally token-colored through the shared tree-sitter pipeline in
//! [`super::code_highlight`]. Does not depend on Markdown.

use arkit_prelude::*;

use crate::theme::{spacing, typography, use_theme, Theme, ThemeMode};

use super::code_highlight::{highlight_code, CodeHighlightPalette};

/// Visual tokens for [`Code`].
///
/// Use [`CodeStyle::from_theme`] and override individual fields when needed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeStyle {
    pub foreground: u32,
    pub muted_foreground: u32,
    pub background: u32,
    pub font_size: f32,
    pub line_height: f32,
    pub padding: f32,
    pub radius: f32,
    pub show_language_label: bool,
    /// Token colors for tree-sitter highlighting.
    pub highlight: CodeHighlightPalette,
}

impl CodeStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            foreground: theme.colors.foreground,
            muted_foreground: theme.colors.muted_foreground,
            background: theme.colors.muted,
            font_size: typography::SM,
            line_height: 20.0,
            padding: spacing::LG,
            radius: theme.radii.md,
            show_language_label: true,
            highlight: match theme.mode {
                ThemeMode::Dark => CodeHighlightPalette::dark(),
                ThemeMode::Light => CodeHighlightPalette::light(),
            },
        }
    }
}

impl Default for CodeStyle {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

/// Props for [`Code`].
#[derive(Props, Clone, PartialEq)]
pub struct CodeProps {
    /// Source text to display (typically a single snippet, not a whole file).
    pub source: String,
    /// Language id for highlighting (fence-style alias, e.g. `rust`, `js`).
    /// Unknown or missing languages fall back to plain monospace.
    #[props(default)]
    pub language: Option<String>,
    /// When `true` (default), attempt tree-sitter highlighting.
    #[props(default = true)]
    pub highlight: bool,
    /// Complete style override. Omit to track the active shadcn theme.
    #[props(default)]
    pub style: Option<CodeStyle>,
}

/// Render a syntax-highlighted code block as native ArkUI nodes.
///
/// Independent of Markdown: use this for settings screens, docs snippets, or
/// any UI that shows code without a surrounding CommonMark document.
#[component]
pub fn Code(props: CodeProps) -> Element {
    let theme = use_theme();
    let style = props.style.unwrap_or_else(|| CodeStyle::from_theme(&theme));
    let language = props.language.as_deref();
    let language_label = language
        .filter(|_| style.show_language_label)
        .map(|value| value.to_string());
    let body = render_code_body(language, &props.source, &style, props.highlight);

    rsx! {
        column {
            percent_width: 1.0,
            align_items: "start",
            padding_top: style.padding,
            padding_right: style.padding,
            padding_bottom: style.padding,
            padding_left: style.padding,
            background_color: style.background,
            border_radius: style.radius,
            clip: true,
            if let Some(language) = language_label {
                text {
                    content: language,
                    font_size: 11.0,
                    font_weight: 600_i32,
                    font_color: style.muted_foreground,
                    line_height: 16.0,
                    margin_bottom: spacing::SM,
                }
            }
            {body}
        }
    }
}

/// Shared body renderer used by [`Code`] and Markdown fenced blocks.
pub(crate) fn render_code_body(
    language: Option<&str>,
    content: &str,
    style: &CodeStyle,
    highlight: bool,
) -> Element {
    if highlight {
        if let Some(lines) = highlight_code(language, content, style.highlight) {
            let rows = lines
                .into_iter()
                .enumerate()
                .map(|(line_index, line)| {
                    let spans = line
                        .spans
                        .into_iter()
                        .enumerate()
                        .map(|(span_index, span)| {
                            let key = format!("{line_index}-{span_index}");
                            rsx! {
                                text {
                                    key: "{key}",
                                    content: span.text.to_string(),
                                    font_size: style.font_size,
                                    font_family: "monospace",
                                    font_color: span.color,
                                    line_height: style.line_height,
                                    text_align: 0_i32,
                                }
                            }
                        })
                        .collect::<Vec<_>>();
                    rsx! {
                        row {
                            key: "{line_index}",
                            percent_width: 1.0,
                            align_items: "baseline",
                            justify_content: "start",
                            height: style.line_height,
                            {spans.into_iter()}
                        }
                    }
                })
                .collect::<Vec<_>>();
            return rsx! {
                column {
                    percent_width: 1.0,
                    align_items: "start",
                    {rows.into_iter()}
                }
            };
        }
    }

    rsx! {
        text {
            content: content.to_string(),
            percent_width: 1.0,
            font_size: style.font_size,
            font_family: "monospace",
            font_color: style.foreground,
            line_height: style.line_height,
            text_align: 0_i32,
        }
    }
}

/// Build a [`CodeStyle`] from Markdown code-related tokens (same feature set).
#[cfg(feature = "markdown")]
pub(crate) fn code_style_from_markdown(
    foreground: u32,
    muted_foreground: u32,
    background: u32,
    font_size: f32,
    line_height: f32,
    padding: f32,
    radius: f32,
    highlight: CodeHighlightPalette,
) -> CodeStyle {
    CodeStyle {
        foreground,
        muted_foreground,
        background,
        font_size,
        line_height,
        padding,
        radius,
        show_language_label: true,
        highlight,
    }
}
