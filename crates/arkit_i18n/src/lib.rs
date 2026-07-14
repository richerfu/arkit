//! Type-safe i18n support for Arkit applications, ported to dioxus 0.7.
//!
//! The compile-time `i18n!` macro (from [`arkit_i18n_macros`]) generates a
//! `Locale` enum, message-constructor functions, and a `pub static CATALOG`.
//! This crate provides the runtime translation machinery plus a dioxus context
//! layer so locale state lives in a cheap, shared `Signal<Rc<str>>`.
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
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use dioxus_hooks::{use_context, use_context_provider, use_signal};
use dioxus_signals::{Signal, WritableExt};
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use unic_langid::LanguageIdentifier;

pub use arkit_i18n_macros::i18n;

// ---------------------------------------------------------------------------
// Runtime translation machinery
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
    args: SmallVec<[I18nArg; 2]>,
}

impl TypedMessage {
    pub fn new(key: &'static str) -> Self {
        Self {
            key,
            args: SmallVec::new(),
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
pub struct LocaleCatalog {
    pub id: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Catalog {
    pub fallback: &'static str,
    /// Locale catalogs sorted by `id`. The `i18n!` macro guarantees this
    /// invariant so runtime lookup remains allocation-free.
    pub locales: &'static [LocaleCatalog],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum I18nError {
    MissingLocale(String),
    MissingMessage {
        locale: String,
        key: &'static str,
    },
    MissingValue {
        locale: String,
        key: &'static str,
    },
    InvalidLocale {
        locale: String,
        reason: Box<str>,
    },
    InvalidResource {
        locale: String,
        reason: Box<str>,
    },
    BundleResource {
        locale: String,
        reason: Box<str>,
    },
    Formatting {
        locale: String,
        key: &'static str,
        errors: Box<[Box<str>]>,
    },
}

impl Display for I18nError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingLocale(locale) => write!(f, "missing i18n locale `{locale}`"),
            Self::MissingMessage { locale, key } => {
                write!(f, "missing i18n message `{key}` for locale `{locale}`")
            }
            Self::MissingValue { locale, key } => {
                write!(f, "i18n message `{key}` for locale `{locale}` has no value")
            }
            Self::InvalidLocale { locale, reason } => {
                write!(f, "invalid i18n locale `{locale}`: {reason}")
            }
            Self::InvalidResource { locale, reason } => {
                write!(f, "invalid Fluent resource for `{locale}`: {reason}")
            }
            Self::BundleResource { locale, reason } => {
                write!(
                    f,
                    "failed to install Fluent resource for `{locale}`: {reason}"
                )
            }
            Self::Formatting {
                locale,
                key,
                errors,
            } => {
                write!(
                    f,
                    "failed to format Fluent message `{key}` for `{locale}`: {}",
                    errors
                        .iter()
                        .map(AsRef::as_ref)
                        .collect::<Vec<&str>>()
                        .join("; ")
                )
            }
        }
    }
}

impl Error for I18nError {}

pub fn translate(catalog: &'static Catalog, locale: &str, message: TypedMessage) -> String {
    let key = message.key();
    try_translate(catalog, locale, message).unwrap_or_else(|_| key.to_string())
}

pub fn try_translate(
    catalog: &'static Catalog,
    locale: &str,
    message: TypedMessage,
) -> Result<String, I18nError> {
    let (locale_index, locale_catalog) = find_locale(catalog, locale)
        .or_else(|| find_locale(catalog, catalog.fallback))
        .ok_or_else(|| I18nError::MissingLocale(locale.to_string()))?;
    match format_message(catalog, locale_index, locale_catalog, &message) {
        Err(I18nError::MissingMessage { .. }) if locale_catalog.id != catalog.fallback => {
            let (fallback_index, fallback) = find_locale(catalog, catalog.fallback)
                .ok_or_else(|| I18nError::MissingLocale(catalog.fallback.to_string()))?;
            format_message(catalog, fallback_index, fallback, &message)
        }
        result => result,
    }
}

fn find_locale(catalog: &'static Catalog, locale: &str) -> Option<(usize, &'static LocaleCatalog)> {
    catalog
        .locales
        .binary_search_by(|candidate| candidate.id.cmp(locale))
        .ok()
        .map(|index| (index, &catalog.locales[index]))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BundleKey {
    catalog: usize,
    locale: usize,
}

type RuntimeBundle = FluentBundle<FluentResource>;

thread_local! {
    static BUNDLE_CACHE: RefCell<FxHashMap<BundleKey, Rc<RuntimeBundle>>> =
        RefCell::new(FxHashMap::default());
}

fn format_message(
    catalog: &'static Catalog,
    locale_index: usize,
    locale: &'static LocaleCatalog,
    message: &TypedMessage,
) -> Result<String, I18nError> {
    let bundle = cached_bundle(catalog, locale_index, locale)?;
    let (message_id, attribute) = message
        .key()
        .split_once('.')
        .map_or((message.key(), None), |(message, attribute)| {
            (message, Some(attribute))
        });
    let fluent_message =
        bundle
            .get_message(message_id)
            .ok_or_else(|| I18nError::MissingMessage {
                locale: locale.id.to_string(),
                key: message.key(),
            })?;
    let pattern = match attribute {
        Some(attribute) => fluent_message
            .get_attribute(attribute)
            .map(|attribute| attribute.value()),
        None => fluent_message.value(),
    }
    .ok_or_else(|| I18nError::MissingValue {
        locale: locale.id.to_string(),
        key: message.key(),
    })?;
    let mut args = FluentArgs::new();
    for arg in message.args() {
        args.set(arg.name(), fluent_value(arg.value()));
    }
    let mut errors = Vec::new();
    let output = bundle
        .format_pattern(pattern, Some(&args), &mut errors)
        .into_owned();
    if errors.is_empty() {
        Ok(output)
    } else {
        Err(I18nError::Formatting {
            locale: locale.id.to_string(),
            key: message.key(),
            errors: errors
                .into_iter()
                .map(|error| error.to_string().into_boxed_str())
                .collect(),
        })
    }
}

fn cached_bundle(
    catalog: &'static Catalog,
    locale_index: usize,
    locale: &'static LocaleCatalog,
) -> Result<Rc<RuntimeBundle>, I18nError> {
    let key = BundleKey {
        catalog: std::ptr::from_ref(catalog).addr(),
        locale: locale_index,
    };
    if let Some(bundle) = BUNDLE_CACHE.with_borrow(|cache| cache.get(&key).cloned()) {
        return Ok(bundle);
    }
    let language =
        locale
            .id
            .parse::<LanguageIdentifier>()
            .map_err(|error| I18nError::InvalidLocale {
                locale: locale.id.to_string(),
                reason: error.to_string().into_boxed_str(),
            })?;
    let resource = FluentResource::try_new(locale.source.to_string()).map_err(|(_, errors)| {
        I18nError::InvalidResource {
            locale: locale.id.to_string(),
            reason: errors
                .into_iter()
                .map(|error| format!("{error:?}"))
                .collect::<Vec<_>>()
                .join("; ")
                .into_boxed_str(),
        }
    })?;
    let mut bundle = RuntimeBundle::new(vec![language]);
    // Preserve Arkit's historical plain-string output. Fluent's optional
    // FSI/PDI isolation is useful for rich bidi-aware renderers, while ArkUI
    // text callers expect exact display strings from this API.
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .map_err(|errors| I18nError::BundleResource {
            locale: locale.id.to_string(),
            reason: errors
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ")
                .into_boxed_str(),
        })?;
    let bundle = Rc::new(bundle);
    BUNDLE_CACHE.with_borrow_mut(|cache| {
        cache.insert(key, bundle.clone());
    });
    Ok(bundle)
}

fn fluent_value(value: &I18nValue) -> FluentValue<'_> {
    match value {
        I18nValue::String(value) => value.into(),
        I18nValue::I64(value) => value.into(),
        I18nValue::U64(value) => value.into(),
        I18nValue::F64(value) => value.into(),
        I18nValue::Bool(value) => {
            if *value {
                "true".into()
            } else {
                "false".into()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// dioxus context layer
// ---------------------------------------------------------------------------

/// A shared i18n context backed by an active-locale signal and a static,
/// compile-time-generated translation catalog.
///
/// Clone/Copy: the underlying `Signal` is `Copy` and shares state, so a context
/// obtained from [`use_context`] mutates the same shared locale.
#[derive(Clone, Copy)]
pub struct I18nContext {
    locale: Signal<Rc<str>>,
    catalog: &'static Catalog,
}

impl I18nContext {
    /// The active locale id (e.g. `"zh-CN"`).
    pub fn locale_id(&self) -> String {
        (self.locale)().to_string()
    }

    /// The static catalog backing this context.
    pub fn catalog(&self) -> &'static Catalog {
        self.catalog
    }

    /// Switch the active locale. Because `Signal` uses interior mutability,
    /// this takes `&self` and is safe to call from `Fn` event handlers.
    pub fn set_locale_id(&self, id: impl Into<String>) {
        let mut signal = self.locale;
        signal.set(Rc::from(id.into()));
    }

    /// Translate a typed message using the active locale (with fallback).
    pub fn tr(&self, message: TypedMessage) -> String {
        let locale = (self.locale)();
        translate(self.catalog, locale.as_ref(), message)
    }

    /// Translate a typed message, returning the raw error on failure.
    pub fn try_tr(&self, message: TypedMessage) -> Result<String, I18nError> {
        let locale = (self.locale)();
        try_translate(self.catalog, locale.as_ref(), message)
    }
}

/// Provide an [`I18nContext`] at the current scope (call once near the app root).
///
/// `catalog` is typically the `pub static CATALOG` generated by `i18n!`, and
/// `initial` is the starting locale id (e.g. `tr::FALLBACK_LOCALE.id()`).
#[must_use]
pub fn use_i18n_provider(catalog: &'static Catalog, initial: impl Into<String>) -> I18nContext {
    let locale = use_signal(move || Rc::from(initial.into()));
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
                source: "hello = Hello, { $name }!\ncount = { $count } items\nitems = { $count ->\n    [one] One item\n   *[other] { $count } items\n}",
            },
            LocaleCatalog {
                id: "zh-CN",
                source: "hello = 你好，{ $name }！",
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
    fn reports_fluent_formatting_errors() {
        let error = try_translate(&CATALOG, "en-US", TypedMessage::new("hello")).unwrap_err();

        assert!(matches!(error, I18nError::Formatting { key: "hello", .. }));
    }

    #[test]
    fn evaluates_fluent_select_expressions() {
        assert_eq!(
            translate(
                &CATALOG,
                "en-US",
                TypedMessage::new("items").with_arg("count", 1),
            ),
            "One item"
        );
        assert_eq!(
            translate(
                &CATALOG,
                "en-US",
                TypedMessage::new("items").with_arg("count", 3),
            ),
            "3 items"
        );
    }
}
