//! Type-safe i18n support for Arkit applications, ported to dioxus 0.7.
//!
//! The compile-time `i18n!` macro (from [`arkit_i18n_macros`]) generates a
//! `Locale` enum, message-constructor functions, and a `pub static CATALOG`.
//! This crate provides the runtime translation machinery plus a dioxus context
//! layer so locale state lives in a `Signal<String>` shared across the tree.
//!
//! ```ignore
//! arkit_i18n::i18n! {
//!     pub mod tr {
//!         path: "locales",
//!         fallback: "zh-CN",
//!         locales: ["zh-CN", "en-US"],
//!     }
//! }
//!
//! #[entry]
//! fn app() -> Element {
//!     arkit_i18n::use_i18n_provider(tr::CATALOG, tr::FALLBACK_LOCALE.id());
//!     rsx! { text { "{t!(tr::app_title())}" } }
//! }
//! ```

extern crate self as arkit_i18n;

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};

use dioxus_hooks::{use_context, use_context_provider, use_signal};
use dioxus_signals::{Signal, WritableExt};

pub use arkit_i18n_macros::i18n;

// ---------------------------------------------------------------------------
// Runtime translation machinery (preserved from legacy)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum I18nValue {
    String(String),
    I64(i64),
    U64(u64),
    F64(f64),
    Bool(bool),
}

impl Display for I18nValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => f.write_str(value),
            Self::I64(value) => Display::fmt(value, f),
            Self::U64(value) => Display::fmt(value, f),
            Self::F64(value) => Display::fmt(value, f),
            Self::Bool(value) => Display::fmt(value, f),
        }
    }
}

impl From<String> for I18nValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for I18nValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<Cow<'_, str>> for I18nValue {
    fn from(value: Cow<'_, str>) -> Self {
        Self::String(value.into_owned())
    }
}

impl From<bool> for I18nValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

macro_rules! impl_i18n_signed {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for I18nValue {
                fn from(value: $ty) -> Self {
                    Self::I64(value as i64)
                }
            }
        )*
    };
}

macro_rules! impl_i18n_unsigned {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for I18nValue {
                fn from(value: $ty) -> Self {
                    Self::U64(value as u64)
                }
            }
        )*
    };
}

impl_i18n_signed!(i8, i16, i32, i64, isize);
impl_i18n_unsigned!(u8, u16, u32, u64, usize);

impl From<f32> for I18nValue {
    fn from(value: f32) -> Self {
        Self::F64(value as f64)
    }
}

impl From<f64> for I18nValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct I18nArg {
    name: &'static str,
    value: I18nValue,
}

impl I18nArg {
    pub fn new(name: &'static str, value: impl Into<I18nValue>) -> Self {
        Self {
            name,
            value: value.into(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn value(&self) -> &I18nValue {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedMessage {
    key: &'static str,
    args: Vec<I18nArg>,
}

impl TypedMessage {
    pub fn new(key: &'static str) -> Self {
        Self {
            key,
            args: Vec::new(),
        }
    }

    pub fn with_arg(mut self, name: &'static str, value: impl Into<I18nValue>) -> Self {
        self.args.push(I18nArg::new(name, value));
        self
    }

    pub fn key(&self) -> &'static str {
        self.key
    }

    pub fn args(&self) -> &[I18nArg] {
        &self.args
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Message {
    pub key: &'static str,
    pub pattern: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocaleCatalog {
    pub id: &'static str,
    pub messages: &'static [Message],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Catalog {
    pub fallback: &'static str,
    pub locales: &'static [LocaleCatalog],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum I18nError {
    MissingLocale(String),
    MissingMessage { locale: String, key: &'static str },
    MissingArgument { key: &'static str, argument: String },
    UnterminatedArgument { key: &'static str },
}

impl Display for I18nError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingLocale(locale) => write!(f, "missing i18n locale `{locale}`"),
            Self::MissingMessage { locale, key } => {
                write!(f, "missing i18n message `{key}` for locale `{locale}`")
            }
            Self::MissingArgument { key, argument } => {
                write!(
                    f,
                    "missing i18n argument `{{${argument}}}` for message `{key}`"
                )
            }
            Self::UnterminatedArgument { key } => {
                write!(f, "unterminated i18n argument in message `{key}`")
            }
        }
    }
}

impl Error for I18nError {}

pub fn translate(catalog: &'static Catalog, locale: &str, message: TypedMessage) -> String {
    try_translate(catalog, locale, message.clone()).unwrap_or_else(|_| message.key().to_string())
}

pub fn try_translate(
    catalog: &'static Catalog,
    locale: &str,
    message: TypedMessage,
) -> Result<String, I18nError> {
    let locale_catalog = find_locale(catalog, locale)
        .or_else(|| find_locale(catalog, catalog.fallback))
        .ok_or_else(|| I18nError::MissingLocale(locale.to_string()))?;
    let pattern = find_message(locale_catalog, message.key())
        .or_else(|| {
            find_locale(catalog, catalog.fallback)
                .and_then(|fallback| find_message(fallback, message.key()))
        })
        .ok_or_else(|| I18nError::MissingMessage {
            locale: locale_catalog.id.to_string(),
            key: message.key(),
        })?;

    render_pattern(message.key(), pattern, message.args())
}

fn find_locale(catalog: &'static Catalog, locale: &str) -> Option<&'static LocaleCatalog> {
    catalog
        .locales
        .iter()
        .find(|candidate| candidate.id == locale)
}

fn find_message(locale: &'static LocaleCatalog, key: &str) -> Option<&'static str> {
    locale
        .messages
        .iter()
        .find(|message| message.key == key)
        .map(|message| message.pattern)
}

fn render_pattern(key: &'static str, pattern: &str, args: &[I18nArg]) -> Result<String, I18nError> {
    let mut output = String::with_capacity(pattern.len());
    let mut rest = pattern;

    while let Some(start) = rest.find("{$") {
        output.push_str(&rest[..start]);
        let name_start = start + 2;
        let Some(relative_end) = rest[name_start..].find('}') else {
            return Err(I18nError::UnterminatedArgument { key });
        };
        let name_end = name_start + relative_end;
        let name = &rest[name_start..name_end];
        let arg = args.iter().find(|arg| arg.name() == name).ok_or_else(|| {
            I18nError::MissingArgument {
                key,
                argument: name.to_string(),
            }
        })?;
        output.push_str(&arg.value().to_string());
        rest = &rest[name_end + 1..];
    }

    output.push_str(rest);
    Ok(output)
}

// ---------------------------------------------------------------------------
// dioxus context layer
// ---------------------------------------------------------------------------

/// A shared i18n context backed by a `Signal<String>` (the active locale id) and
/// a `&'static Catalog` (the compile-time-generated translation table).
///
/// Clone/Copy: the underlying `Signal` is `Copy` and shares state, so a context
/// obtained from [`use_context`] mutates the same shared locale.
#[derive(Clone, Copy)]
pub struct I18nContext {
    locale: Signal<String>,
    catalog: &'static Catalog,
}

impl I18nContext {
    /// The active locale id (e.g. `"zh-CN"`).
    pub fn locale_id(&self) -> String {
        (self.locale)()
    }

    /// The static catalog backing this context.
    pub fn catalog(&self) -> &'static Catalog {
        self.catalog
    }

    /// Switch the active locale. Because `Signal` uses interior mutability,
    /// this takes `&self` and is safe to call from `Fn` event handlers.
    pub fn set_locale_id(&self, id: impl Into<String>) {
        let mut signal = self.locale;
        signal.set(id.into());
    }

    /// Translate a typed message using the active locale (with fallback).
    pub fn tr(&self, message: TypedMessage) -> String {
        let locale = (self.locale)();
        translate(self.catalog, &locale, message)
    }

    /// Translate a typed message, returning the raw error on failure.
    pub fn try_tr(&self, message: TypedMessage) -> Result<String, I18nError> {
        let locale = (self.locale)();
        try_translate(self.catalog, &locale, message)
    }
}

/// Provide an [`I18nContext`] at the current scope (call once near the app root).
///
/// `catalog` is typically the `pub static CATALOG` generated by `i18n!`, and
/// `initial` is the starting locale id (e.g. `tr::FALLBACK_LOCALE.id()`).
#[must_use]
pub fn use_i18n_provider(catalog: &'static Catalog, initial: impl Into<String>) -> I18nContext {
    let locale = use_signal(move || initial.into());
    use_context_provider(move || I18nContext { locale, catalog })
}

/// Read the shared [`I18nContext`] provided by an ancestor (or the same scope).
#[must_use]
pub fn use_i18n() -> I18nContext {
    use_context::<I18nContext>()
}

/// Compile-time string resolver: translates a typed message against the current
/// [`I18nContext`]. Expands to a `use_i18n()` read + `translate`, so call it in
/// component bodies / `rsx!` expressions.
///
/// ```ignore
/// rsx! { text { "{t!(tr::welcome_user(\"Arkit\"))}" } }
/// ```
#[macro_export]
macro_rules! t {
    ($message:expr) => {{
        let __ctx = $crate::use_i18n();
        __ctx.tr($message)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    static CATALOG: Catalog = Catalog {
        fallback: "en-US",
        locales: &[
            LocaleCatalog {
                id: "en-US",
                messages: &[
                    Message {
                        key: "hello",
                        pattern: "Hello, {$name}!",
                    },
                    Message {
                        key: "count",
                        pattern: "{$count} items",
                    },
                ],
            },
            LocaleCatalog {
                id: "zh-CN",
                messages: &[Message {
                    key: "hello",
                    pattern: "你好，{$name}！",
                }],
            },
        ],
    };

    #[test]
    fn translates_with_arguments() {
        let value = translate(
            &CATALOG,
            "en-US",
            TypedMessage::new("hello").with_arg("name", "Ada"),
        );

        assert_eq!(value, "Hello, Ada!");
    }

    #[test]
    fn falls_back_to_default_locale_message() {
        let value = translate(
            &CATALOG,
            "zh-CN",
            TypedMessage::new("count").with_arg("count", 3),
        );

        assert_eq!(value, "3 items");
    }

    #[test]
    fn reports_missing_argument() {
        let error = try_translate(&CATALOG, "en-US", TypedMessage::new("hello")).unwrap_err();

        assert_eq!(
            error,
            I18nError::MissingArgument {
                key: "hello",
                argument: "name".to_string()
            }
        );
    }
}
