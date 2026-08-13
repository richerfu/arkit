//! Tree-sitter syntax highlighting for the standalone [`super::Code`] component
//! and for Markdown fenced blocks when both `markdown` and `code` are enabled.
//!
//! # Feature gate
//!
//! This entire module—including built-in highlighters, [`highlight_code`],
//! palette types, and custom registration ([`register_language`],
//! [`register_highlight_configuration`], and related helpers)—is compiled
//! **only** when the `code` Cargo feature is enabled. It does **not** require
//! `markdown`. Enable `markdown` + `code` (or the `markdown-highlight` alias)
//! to highlight fenced code inside Markdown documents.
//!
//! # Custom languages
//!
//! Built-ins cover common fence tags. Applications can register additional
//! grammars at startup with [`register_language`] (or
//! [`register_highlight_configuration`]). Aliases are matched
//! case-insensitively against the language / fence info string.
//!
//! Grammar crates must be compatible with the same `tree-sitter` major as
//! this crate (currently 0.25). Prefer converting via
//! [`Language::from`] / `.into()` on the grammar's `LANGUAGE` constant.
//! Capture names should align with [`HIGHLIGHT_NAMES`] so
//! [`CodeHighlightPalette`] can map tokens to colors.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, OnceLock, RwLock};

use tree_sitter_highlight::{Highlight, HighlightEvent, Highlighter};

/// Re-export so integrators can name the same type `register_language` expects.
pub use tree_sitter::Language;
/// Re-export for advanced registration via [`register_highlight_configuration`].
pub use tree_sitter_highlight::HighlightConfiguration;

/// Capture names recognized by every language configuration.
///
/// Order is significant: indices match [`Highlight`] values returned by
/// tree-sitter-highlight. Custom queries should use these capture names
/// (or their base prefixes, e.g. `function.method` → palette `function`).
pub const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "comment",
    "constant",
    "constant.builtin",
    "constructor",
    "embedded",
    "function",
    "function.builtin",
    "keyword",
    "module",
    "number",
    "operator",
    "property",
    "property.builtin",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

/// Theme colors for highlighted code tokens (ARGB).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeHighlightPalette {
    pub default: u32,
    pub comment: u32,
    pub keyword: u32,
    pub string: u32,
    pub number: u32,
    pub function: u32,
    pub type_name: u32,
    pub constant: u32,
    pub property: u32,
    pub operator: u32,
    pub punctuation: u32,
    pub variable: u32,
    pub attribute: u32,
    pub tag: u32,
}

impl CodeHighlightPalette {
    /// Palette tuned for light shadcn surfaces (zinc / neutral).
    pub const fn light() -> Self {
        Self {
            default: 0xFF18181B,
            comment: 0xFF71717A,
            keyword: 0xFF7C3AED,
            string: 0xFF16A34A,
            number: 0xFFEA580C,
            function: 0xFF2563EB,
            type_name: 0xFF0D9488,
            constant: 0xFFDB2777,
            property: 0xFF0284C7,
            operator: 0xFF52525B,
            punctuation: 0xFF71717A,
            variable: 0xFF18181B,
            attribute: 0xFFCA8A04,
            tag: 0xFFDC2626,
        }
    }

    /// Palette for dark shadcn surfaces.
    pub const fn dark() -> Self {
        Self {
            default: 0xFFFAFAFA,
            comment: 0xFFA1A1AA,
            keyword: 0xFFC4B5FD,
            string: 0xFF86EFAC,
            number: 0xFFFDBA74,
            function: 0xFF93C5FD,
            type_name: 0xFF5EEAD4,
            constant: 0xFFF9A8D4,
            property: 0xFF7DD3FC,
            operator: 0xFFD4D4D8,
            punctuation: 0xFFA1A1AA,
            variable: 0xFFFAFAFA,
            attribute: 0xFFFDE047,
            tag: 0xFFFCA5A5,
        }
    }

    fn color_for_name(self, name: &str) -> u32 {
        let base = name.split('.').next().unwrap_or(name);
        match base {
            "comment" => self.comment,
            "keyword" => self.keyword,
            "string" => self.string,
            "number" => self.number,
            "function" | "constructor" => self.function,
            "type" => self.type_name,
            "constant" => self.constant,
            "property" => self.property,
            "operator" => self.operator,
            "punctuation" => self.punctuation,
            "variable" | "module" | "embedded" => self.variable,
            "attribute" => self.attribute,
            "tag" => self.tag,
            _ => self.default,
        }
    }

    fn color_for_highlight(self, highlight: Highlight) -> u32 {
        HIGHLIGHT_NAMES
            .get(highlight.0)
            .map(|name| self.color_for_name(name))
            .unwrap_or(self.default)
    }
}

impl Default for CodeHighlightPalette {
    fn default() -> Self {
        Self::light()
    }
}

/// One colored span of source text (no embedded newlines).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    pub text: Arc<str>,
    pub color: u32,
}

/// One visual line of highlighted code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightLine {
    pub spans: Vec<HighlightSpan>,
}

/// Failure while installing a custom language into the highlight registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterLanguageError {
    /// `aliases` was empty after normalization.
    EmptyAliases,
    /// An alias was empty or whitespace-only.
    InvalidAlias,
    /// Highlight query / configuration failed to load.
    Config(String),
    /// Registry lock was poisoned.
    LockPoisoned,
}

impl fmt::Display for RegisterLanguageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAliases => write!(f, "at least one language alias is required"),
            Self::InvalidAlias => write!(f, "language alias must be non-empty"),
            Self::Config(message) => write!(f, "highlight configuration failed: {message}"),
            Self::LockPoisoned => write!(f, "language registry lock poisoned"),
        }
    }
}

impl std::error::Error for RegisterLanguageError {}

/// Highlight `source` for `language` (fence info string, e.g. `rust`, `js`).
///
/// Returns `None` when the language is unknown or highlighting fails so the
/// caller can render plain monospace text. Looks up both built-in and
/// [`register_language`]-installed aliases (and built-ins).
pub fn highlight_code(
    language: Option<&str>,
    source: &str,
    palette: CodeHighlightPalette,
) -> Option<Vec<HighlightLine>> {
    let language = language?;
    let config = language_config(language)?;
    let mut highlighter = Highlighter::new();
    let events = highlighter
        .highlight(config.as_ref(), source.as_bytes(), None, |_| None)
        .ok()?;
    Some(events_to_lines(source, events, palette))
}

/// Sorted list of every registered fence alias (built-in + custom).
pub fn supported_languages() -> Vec<String> {
    match registry().read() {
        Ok(guard) => guard.aliases(),
        Err(_) => Vec::new(),
    }
}

/// Whether `language` resolves to a highlight configuration.
pub fn is_language_registered(language: &str) -> bool {
    language_config(language).is_some()
}

/// Register a grammar for fenced-code highlighting.
///
/// Requires the `code` feature (this symbol is not present otherwise).
/// Registration is process-global and feeds [`highlight_code`], the
/// standalone [`super::Code`] component, and Markdown fenced blocks when
/// both `markdown` and `code` are enabled.
///
/// `aliases` are fence info strings (e.g. `["zig", "zon"]`), matched
/// case-insensitively. Existing aliases are overwritten with the new config
/// (shared across all listed aliases).
///
/// The configuration is built with the provided queries and then configured
/// against [`HIGHLIGHT_NAMES`] so token colors match [`CodeHighlightPalette`].
///
/// # Example
///
/// ```ignore
/// // Cargo: arkit = { features = ["code"] }
/// use arkit::shadcn::components::register_language;
///
/// register_language(
///     &["toml"],
///     tree_sitter_toml::LANGUAGE.into(),
///     "toml",
///     tree_sitter_toml::HIGHLIGHTS_QUERY,
///     "",
///     "",
/// )?;
/// ```
pub fn register_language(
    aliases: &[&str],
    language: Language,
    name: &str,
    highlights_query: &str,
    injections_query: &str,
    locals_query: &str,
) -> Result<(), RegisterLanguageError> {
    let config = HighlightConfiguration::new(
        language,
        name,
        highlights_query,
        injections_query,
        locals_query,
    )
    .map_err(|error| RegisterLanguageError::Config(error.to_string()))?;
    register_highlight_configuration(aliases, config)
}

/// Register a pre-built [`HighlightConfiguration`].
///
/// Always re-applies [`HIGHLIGHT_NAMES`] so palette indices stay consistent
/// with built-in languages. Prefer [`register_language`] unless you need
/// custom setup before install.
pub fn register_highlight_configuration(
    aliases: &[&str],
    mut config: HighlightConfiguration,
) -> Result<(), RegisterLanguageError> {
    let aliases = normalize_aliases(aliases)?;
    config.configure(HIGHLIGHT_NAMES);
    let config = Arc::new(config);
    let mut guard = registry()
        .write()
        .map_err(|_| RegisterLanguageError::LockPoisoned)?;
    for alias in aliases {
        guard.by_alias.insert(alias, Arc::clone(&config));
    }
    Ok(())
}

/// Remove a single fence alias from the registry.
///
/// Returns `true` if the alias existed. Other aliases that share the same
/// configuration are left intact. Built-in aliases can be removed; call
/// [`reset_language_registry`] to restore defaults.
pub fn unregister_language(alias: &str) -> bool {
    let key = alias.trim().to_ascii_lowercase();
    if key.is_empty() {
        return false;
    }
    match registry().write() {
        Ok(mut guard) => guard.by_alias.remove(&key).is_some(),
        Err(_) => false,
    }
}

/// Drop all custom entries and reinstall the built-in language set.
pub fn reset_language_registry() {
    if let Ok(mut guard) = registry().write() {
        *guard = LanguageRegistry::with_builtins();
    }
}

fn events_to_lines(
    source: &str,
    events: impl Iterator<Item = Result<HighlightEvent, tree_sitter_highlight::Error>>,
    palette: CodeHighlightPalette,
) -> Vec<HighlightLine> {
    let bytes = source.as_bytes();
    let mut lines: Vec<HighlightLine> = Vec::new();
    let mut current_spans: Vec<HighlightSpan> = Vec::new();
    let mut style_stack: Vec<Highlight> = Vec::new();

    let push_text = |spans: &mut Vec<HighlightSpan>, text: &str, color: u32| {
        if text.is_empty() {
            return;
        }
        if let Some(last) = spans.last_mut() {
            if last.color == color {
                let mut combined = last.text.as_ref().to_owned();
                combined.push_str(text);
                last.text = Arc::from(combined);
                return;
            }
        }
        spans.push(HighlightSpan {
            text: Arc::from(text),
            color,
        });
    };

    let flush_line = |lines: &mut Vec<HighlightLine>, spans: &mut Vec<HighlightSpan>| {
        if spans.is_empty() {
            // Preserve blank lines as a single space so row height remains.
            spans.push(HighlightSpan {
                text: Arc::from(" "),
                color: palette.default,
            });
        }
        lines.push(HighlightLine {
            spans: std::mem::take(spans),
        });
    };

    for event in events.flatten() {
        match event {
            HighlightEvent::HighlightStart(highlight) => style_stack.push(highlight),
            HighlightEvent::HighlightEnd => {
                let _ = style_stack.pop();
            }
            HighlightEvent::Source { start, end } => {
                let color = style_stack
                    .last()
                    .map(|highlight| palette.color_for_highlight(*highlight))
                    .unwrap_or(palette.default);
                let chunk = std::str::from_utf8(&bytes[start..end]).unwrap_or("");
                for (piece_index, piece) in chunk.split('\n').enumerate() {
                    if piece_index > 0 {
                        flush_line(&mut lines, &mut current_spans);
                    }
                    push_text(&mut current_spans, piece, color);
                }
            }
        }
    }

    if !current_spans.is_empty() || lines.is_empty() {
        flush_line(&mut lines, &mut current_spans);
    }

    // Drop the synthetic blank line introduced solely by a trailing newline.
    if source.ends_with('\n') {
        if let Some(last) = lines.last() {
            if last.spans.len() == 1 && last.spans[0].text.as_ref() == " " {
                lines.pop();
            }
        }
    }

    lines
}

fn language_config(language: &str) -> Option<Arc<HighlightConfiguration>> {
    let key = language.trim().to_ascii_lowercase();
    if key.is_empty() {
        return None;
    }
    registry().read().ok()?.by_alias.get(&key).cloned()
}

fn normalize_aliases(aliases: &[&str]) -> Result<Vec<String>, RegisterLanguageError> {
    let mut normalized = Vec::with_capacity(aliases.len());
    for alias in aliases {
        let key = alias.trim().to_ascii_lowercase();
        if key.is_empty() {
            return Err(RegisterLanguageError::InvalidAlias);
        }
        if !normalized.iter().any(|existing| existing == &key) {
            normalized.push(key);
        }
    }
    if normalized.is_empty() {
        return Err(RegisterLanguageError::EmptyAliases);
    }
    Ok(normalized)
}

fn registry() -> &'static RwLock<LanguageRegistry> {
    static REGISTRY: OnceLock<RwLock<LanguageRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(LanguageRegistry::with_builtins()))
}

struct LanguageRegistry {
    by_alias: HashMap<String, Arc<HighlightConfiguration>>,
}

impl LanguageRegistry {
    fn with_builtins() -> Self {
        let mut registry = Self {
            by_alias: HashMap::new(),
        };

        let mut insert = |aliases: &[&str], config: Option<HighlightConfiguration>| {
            let Some(config) = config else {
                return;
            };
            let config = Arc::new(config);
            for alias in aliases {
                registry
                    .by_alias
                    .insert((*alias).to_string(), Arc::clone(&config));
            }
        };

        insert(
            &["rust", "rs"],
            configure(
                tree_sitter_rust::LANGUAGE.into(),
                "rust",
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                tree_sitter_rust::INJECTIONS_QUERY,
                "",
            ),
        );

        insert(
            &["javascript", "js", "jsx"],
            configure(
                tree_sitter_javascript::LANGUAGE.into(),
                "javascript",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::INJECTIONS_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            ),
        );

        insert(
            &["typescript", "ts"],
            configure(
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                "typescript",
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
                "",
                tree_sitter_typescript::LOCALS_QUERY,
            ),
        );

        insert(
            &["tsx"],
            configure(
                tree_sitter_typescript::LANGUAGE_TSX.into(),
                "tsx",
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
                "",
                tree_sitter_typescript::LOCALS_QUERY,
            ),
        );

        insert(
            &["python", "py"],
            configure(
                tree_sitter_python::LANGUAGE.into(),
                "python",
                tree_sitter_python::HIGHLIGHTS_QUERY,
                "",
                "",
            ),
        );

        insert(
            &["json"],
            configure(
                tree_sitter_json::LANGUAGE.into(),
                "json",
                tree_sitter_json::HIGHLIGHTS_QUERY,
                "",
                "",
            ),
        );

        insert(
            &["bash", "sh", "shell", "zsh"],
            configure(
                tree_sitter_bash::LANGUAGE.into(),
                "bash",
                tree_sitter_bash::HIGHLIGHT_QUERY,
                "",
                "",
            ),
        );

        insert(
            &["go", "golang"],
            configure(
                tree_sitter_go::LANGUAGE.into(),
                "go",
                tree_sitter_go::HIGHLIGHTS_QUERY,
                "",
                "",
            ),
        );

        insert(
            &["c", "h"],
            configure(
                tree_sitter_c::LANGUAGE.into(),
                "c",
                tree_sitter_c::HIGHLIGHT_QUERY,
                "",
                "",
            ),
        );

        registry
    }

    fn aliases(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.by_alias.keys().cloned().collect();
        keys.sort();
        keys
    }
}

fn configure(
    language: Language,
    name: &str,
    highlights: &str,
    injections: &str,
    locals: &str,
) -> Option<HighlightConfiguration> {
    let mut config =
        match HighlightConfiguration::new(language, name, highlights, injections, locals) {
            Ok(config) => config,
            Err(error) => {
                ohos_hilog_binding::warn(format!(
                    "arkit_shadcn: highlight config for {name} failed: {error}"
                ));
                return None;
            }
        };
    config.configure(HIGHLIGHT_NAMES);
    Some(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_highlight_produces_colored_spans() {
        let lines = highlight_code(
            Some("rust"),
            "fn main() {\n    let x = 1;\n}\n",
            CodeHighlightPalette::light(),
        )
        .expect("rust should highlight");
        assert!(lines.len() >= 2);
        let has_non_default = lines.iter().flat_map(|line| line.spans.iter()).any(|span| {
            span.color != CodeHighlightPalette::light().default && !span.text.trim().is_empty()
        });
        assert!(
            has_non_default,
            "expected at least one non-default color span"
        );
    }

    #[test]
    fn unknown_language_returns_none() {
        assert!(highlight_code(Some("brainfuck"), "++++", CodeHighlightPalette::light()).is_none());
        assert!(highlight_code(None, "fn main() {}", CodeHighlightPalette::light()).is_none());
    }

    #[test]
    fn json_highlight_works() {
        let lines = highlight_code(
            Some("json"),
            r#"{ "ok": true }"#,
            CodeHighlightPalette::dark(),
        )
        .expect("json should highlight");
        assert_eq!(lines.len(), 1);
        assert!(!lines[0].spans.is_empty());
    }

    #[test]
    fn register_custom_alias_uses_shared_grammar() {
        // Global registry: use a unique alias and clean up so other tests stay stable.
        let alias = "arkit-test-rust-alias";
        let _ = unregister_language(alias);

        register_language(
            &[alias],
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        )
        .expect("register");

        assert!(is_language_registered(alias));
        assert!(supported_languages().iter().any(|name| name == alias));

        let lines = highlight_code(Some(alias), "fn main() {}\n", CodeHighlightPalette::light())
            .expect("custom alias should highlight");
        assert!(!lines.is_empty());

        assert!(unregister_language(alias));
        assert!(!is_language_registered(alias));
    }

    #[test]
    fn register_rejects_empty_aliases() {
        assert_eq!(
            register_language(
                &[],
                tree_sitter_rust::LANGUAGE.into(),
                "rust",
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                "",
                "",
            ),
            Err(RegisterLanguageError::EmptyAliases)
        );
        assert_eq!(
            register_language(
                &["  "],
                tree_sitter_rust::LANGUAGE.into(),
                "rust",
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                "",
                "",
            ),
            Err(RegisterLanguageError::InvalidAlias)
        );
    }

    #[test]
    fn reset_restores_builtins_and_drops_custom() {
        let alias = "arkit-test-reset-alias";
        register_language(
            &[alias],
            tree_sitter_json::LANGUAGE.into(),
            "json",
            tree_sitter_json::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .expect("register");
        assert!(is_language_registered(alias));

        reset_language_registry();

        assert!(!is_language_registered(alias));
        assert!(is_language_registered("rust"));
        assert!(is_language_registered("JSON")); // case-insensitive lookup
    }
}
