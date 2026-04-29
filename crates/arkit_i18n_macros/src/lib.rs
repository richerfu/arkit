use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, parse_macro_input, LitStr, Token, Visibility};

#[proc_macro]
pub fn i18n(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as I18nInput);
    match expand_i18n(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

struct I18nInput {
    visibility: Visibility,
    module: Ident,
    path: LitStr,
    fallback: LitStr,
    locales: Vec<LitStr>,
}

impl Parse for I18nInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let visibility = input.parse()?;
        input.parse::<Token![mod]>()?;
        let module: Ident = input.parse()?;
        let content;
        braced!(content in input);

        let mut path = None;
        let mut fallback = None;
        let mut locales: Option<Vec<LitStr>> = None;

        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![:]>()?;
            match key.to_string().as_str() {
                "path" => path = Some(content.parse()?),
                "fallback" => fallback = Some(content.parse()?),
                "locales" => {
                    let items;
                    bracketed!(items in content);
                    let parsed = Punctuated::<LitStr, Token![,]>::parse_terminated(&items)?;
                    locales = Some(parsed.into_iter().collect());
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "expected `path`, `fallback`, or `locales`",
                    ));
                }
            }

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        let path = path.ok_or_else(|| syn::Error::new(module.span(), "missing `path`"))?;
        let fallback =
            fallback.ok_or_else(|| syn::Error::new(module.span(), "missing `fallback`"))?;
        let locales = locales.ok_or_else(|| syn::Error::new(module.span(), "missing `locales`"))?;

        if locales.is_empty() {
            return Err(syn::Error::new(module.span(), "`locales` cannot be empty"));
        }

        Ok(Self {
            visibility,
            module,
            path,
            fallback,
            locales,
        })
    }
}

#[derive(Debug, Clone)]
struct Template {
    pattern: String,
    vars: BTreeSet<String>,
}

fn expand_i18n(input: I18nInput) -> syn::Result<TokenStream2> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|error| syn::Error::new(input.path.span(), error))?;
    let base = PathBuf::from(manifest_dir).join(input.path.value());
    let fallback_id = input.fallback.value();

    let mut locale_ids = Vec::new();
    let mut catalogs = Vec::new();
    for locale in &input.locales {
        let locale_id = locale.value();
        let path = base.join(format!("{locale_id}.ftl"));
        let source = fs::read_to_string(&path).map_err(|error| {
            syn::Error::new(
                locale.span(),
                format!(
                    "failed to read locale `{}` at {}: {error}",
                    locale_id,
                    path.display()
                ),
            )
        })?;
        let messages = parse_ftl(&source).map_err(|message| {
            syn::Error::new(
                locale.span(),
                format!("invalid locale `{locale_id}`: {message}"),
            )
        })?;
        locale_ids.push(locale_id);
        catalogs.push(messages);
    }

    let fallback_index = locale_ids
        .iter()
        .position(|locale| locale == &fallback_id)
        .ok_or_else(|| syn::Error::new(input.fallback.span(), "`fallback` must be in `locales`"))?;
    validate_catalogs(
        &locale_ids,
        &catalogs,
        fallback_index,
        input.fallback.span(),
    )?;

    let visibility = input.visibility;
    let module = input.module;
    let locale_variants = locale_ids
        .iter()
        .map(|locale| locale_variant(locale))
        .collect::<Vec<_>>();
    let fallback_variant = &locale_variants[fallback_index];
    let locale_match_arms = locale_variants
        .iter()
        .zip(locale_ids.iter())
        .map(|(variant, locale)| quote! { Self::#variant => #locale });
    let locale_from_arms = locale_variants
        .iter()
        .zip(locale_ids.iter())
        .map(|(variant, locale)| quote! { #locale => Ok(Self::#variant) });
    let locale_display_arms = locale_variants
        .iter()
        .zip(locale_ids.iter())
        .map(|(variant, locale)| quote! { Self::#variant => f.write_str(#locale) });
    let all_locale_values = locale_variants
        .iter()
        .map(|variant| quote! { Locale::#variant });

    let fallback_catalog = &catalogs[fallback_index];
    let functions = fallback_catalog.iter().map(|(key, template)| {
        let function = message_function(key);
        let params = template
            .vars
            .iter()
            .map(|name| argument_ident(name))
            .collect::<Vec<_>>();
        let names = template.vars.iter().collect::<Vec<_>>();

        if params.is_empty() {
            quote! {
                pub fn #function() -> ::arkit_i18n::TypedMessage {
                    ::arkit_i18n::TypedMessage::new(#key)
                }
            }
        } else {
            quote! {
                pub fn #function(
                    #(#params: impl Into<::arkit_i18n::I18nValue>),*
                ) -> ::arkit_i18n::TypedMessage {
                    ::arkit_i18n::TypedMessage::new(#key)
                        #(.with_arg(#names, #params))*
                }
            }
        }
    });

    let locale_catalogs = locale_ids
        .iter()
        .zip(catalogs.iter())
        .map(|(locale, catalog)| {
            let messages = catalog.iter().map(|(key, template)| {
                let pattern = &template.pattern;
                quote! {
                    ::arkit_i18n::Message {
                        key: #key,
                        pattern: #pattern,
                    }
                }
            });

            quote! {
                ::arkit_i18n::LocaleCatalog {
                    id: #locale,
                    messages: &[#(#messages),*],
                }
            }
        });

    Ok(quote! {
        #visibility mod #module {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum Locale {
                #(#locale_variants),*
            }

            impl Locale {
                pub const fn id(self) -> &'static str {
                    match self {
                        #(#locale_match_arms),*
                    }
                }

                pub const fn all() -> &'static [Self] {
                    &[#(#all_locale_values),*]
                }
            }

            impl std::fmt::Display for Locale {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#locale_display_arms),*
                    }
                }
            }

            impl std::str::FromStr for Locale {
                type Err = String;

                fn from_str(value: &str) -> Result<Self, Self::Err> {
                    match value {
                        #(#locale_from_arms),*,
                        _ => Err(format!("unsupported locale `{value}`")),
                    }
                }
            }

            #[derive(Debug, Clone)]
            pub struct I18n {
                locale: Locale,
            }

            impl I18n {
                pub const fn new(locale: Locale) -> Self {
                    Self { locale }
                }

                pub const fn locale(&self) -> Locale {
                    self.locale
                }

                pub fn set_locale(&mut self, locale: Locale) {
                    self.locale = locale;
                }

                pub const fn available_locales(&self) -> &'static [Locale] {
                    Locale::all()
                }

                pub fn tr(&self, message: ::arkit_i18n::TypedMessage) -> String {
                    ::arkit_i18n::translate(&CATALOG, self.locale.id(), message)
                }

                pub fn try_tr(
                    &self,
                    message: ::arkit_i18n::TypedMessage,
                ) -> Result<String, ::arkit_i18n::I18nError> {
                    ::arkit_i18n::try_translate(&CATALOG, self.locale.id(), message)
                }
            }

            impl Default for I18n {
                fn default() -> Self {
                    Self::new(Locale::#fallback_variant)
                }
            }

            pub const FALLBACK_LOCALE: Locale = Locale::#fallback_variant;

            static CATALOG: ::arkit_i18n::Catalog = ::arkit_i18n::Catalog {
                fallback: #fallback_id,
                locales: &[#(#locale_catalogs),*],
            };

            #(#functions)*
        }
    })
}

fn validate_catalogs(
    locale_ids: &[String],
    catalogs: &[BTreeMap<String, Template>],
    fallback_index: usize,
    span: Span,
) -> syn::Result<()> {
    let fallback = &catalogs[fallback_index];

    for (index, catalog) in catalogs.iter().enumerate() {
        let locale = &locale_ids[index];

        for key in fallback.keys() {
            if !catalog.contains_key(key) {
                return Err(syn::Error::new(
                    span,
                    format!("locale `{locale}` is missing message `{key}`"),
                ));
            }
        }

        for key in catalog.keys() {
            if !fallback.contains_key(key) {
                return Err(syn::Error::new(
                    span,
                    format!("locale `{locale}` has extra message `{key}`"),
                ));
            }
        }

        for (key, template) in fallback {
            let vars = &catalog.get(key).expect("key checked above").vars;
            if vars != &template.vars {
                return Err(syn::Error::new(
                    span,
                    format!(
                        "locale `{locale}` message `{key}` has variables {:?}; expected {:?}",
                        vars, template.vars
                    ),
                ));
            }
        }
    }

    Ok(())
}

fn parse_ftl(source: &str) -> Result<BTreeMap<String, Template>, String> {
    let mut messages: BTreeMap<String, Template> = BTreeMap::new();
    let mut current_key: Option<String> = None;

    for (line_index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if line.starts_with(' ') || line.starts_with('\t') {
            let Some(key) = current_key.as_ref() else {
                return Err(format!(
                    "line {} has a continuation without a message",
                    line_index + 1
                ));
            };
            let template = messages
                .get_mut(key)
                .expect("current key points to an existing message");
            template.pattern.push('\n');
            template.pattern.push_str(trimmed);
            template.vars = extract_vars(&template.pattern)?;
            continue;
        }

        let Some((raw_key, raw_value)) = line.split_once('=') else {
            return Err(format!("line {} is not a message", line_index + 1));
        };
        let key = raw_key.trim();
        if key.is_empty() {
            return Err(format!("line {} has an empty message id", line_index + 1));
        }
        if !is_valid_message_id(key) {
            return Err(format!(
                "line {} has invalid message id `{key}`",
                line_index + 1
            ));
        }
        if messages.contains_key(key) {
            return Err(format!(
                "line {} duplicates message `{key}`",
                line_index + 1
            ));
        }

        let pattern = raw_value.trim().to_string();
        let vars = extract_vars(&pattern)?;
        messages.insert(key.to_string(), Template { pattern, vars });
        current_key = Some(key.to_string());
    }

    if messages.is_empty() {
        return Err("locale does not define any messages".to_string());
    }

    Ok(messages)
}

fn extract_vars(pattern: &str) -> Result<BTreeSet<String>, String> {
    let mut vars = BTreeSet::new();
    let mut rest = pattern;

    while let Some(start) = rest.find("{$") {
        let name_start = start + 2;
        let Some(relative_end) = rest[name_start..].find('}') else {
            return Err("unterminated variable placeholder".to_string());
        };
        let name = &rest[name_start..name_start + relative_end];
        if name.is_empty() || !is_valid_arg_name(name) {
            return Err(format!("invalid variable name `{name}`"));
        }
        vars.insert(name.to_string());
        rest = &rest[name_start + relative_end + 1..];
    }

    Ok(vars)
}

fn is_valid_message_id(value: &str) -> bool {
    value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
}

fn is_valid_arg_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn message_function(key: &str) -> Ident {
    rust_ident(&key.replace(['-', '.'], "_"))
}

fn argument_ident(key: &str) -> Ident {
    rust_ident(key)
}

fn rust_ident(value: &str) -> Ident {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() || out.starts_with(|ch: char| ch.is_ascii_digit()) {
        out.insert(0, '_');
    }
    if is_rust_keyword(&out) {
        out.push('_');
    }
    format_ident!("{out}")
}

fn locale_variant(locale: &str) -> Ident {
    let mut out = String::new();
    let mut uppercase = true;
    for ch in locale.chars() {
        if ch.is_ascii_alphanumeric() {
            if uppercase {
                out.push(ch.to_ascii_uppercase());
                uppercase = false;
            } else {
                out.push(ch.to_ascii_lowercase());
            }
        } else {
            uppercase = true;
        }
    }
    if out.is_empty() || out.starts_with(|ch: char| ch.is_ascii_digit()) {
        out.insert(0, 'L');
    }
    if is_rust_keyword(&out) {
        out.push('_');
    }
    format_ident!("{out}")
}

fn is_rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
    )
}
