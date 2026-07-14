//! Proc-macro crate for `arkit_i18n`.
//!
//! Provides the `i18n!` macro: a compile-time `.ftl` resolver that generates a
//! typed `Locale` enum, an `I18n` helper, message-constructor functions, and a
//! `pub static CATALOG` for runtime translation. Fluent resources are parsed
//! as Fluent syntax (including selects, references, terms, and attributes),
//! and locale/key/variable parity is validated before code generation.

use fluent_syntax::ast;
use fluent_syntax::parser;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, parse_macro_input, LitStr, Token, Visibility};
use unic_langid::LanguageIdentifier;

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
    vars: BTreeSet<String>,
}

fn expand_i18n(input: I18nInput) -> syn::Result<TokenStream2> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|error| syn::Error::new(input.path.span(), error))?;
    let base = PathBuf::from(manifest_dir).join(input.path.value());
    let fallback_id = input.fallback.value();

    let mut locale_ids = Vec::new();
    let mut catalogs = Vec::new();
    let mut locale_paths = Vec::new();
    for locale in &input.locales {
        let locale_id = locale.value();
        locale_id.parse::<LanguageIdentifier>().map_err(|error| {
            syn::Error::new(
                locale.span(),
                format!("invalid Unicode locale identifier `{locale_id}`: {error}"),
            )
        })?;
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
        let tracked_path = path.canonicalize().unwrap_or(path);
        let tracked_path = tracked_path.to_str().ok_or_else(|| {
            syn::Error::new(
                locale.span(),
                "locale path is not valid UTF-8 and cannot be used by include_str!",
            )
        })?;
        locale_paths.push(LitStr::new(tracked_path, locale.span()));
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

    let declaration_span = input.fallback.span();
    let runtime = i18n_runtime_path(declaration_span)?;
    let visibility = input.visibility;
    let module = input.module;
    let locale_variants = locale_ids
        .iter()
        .map(|locale| locale_variant(locale))
        .collect::<Vec<_>>();
    ensure_unique_identifiers(
        &locale_ids,
        &locale_variants,
        "locale variants",
        declaration_span,
    )?;
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
    let function_keys = fallback_catalog.keys().cloned().collect::<Vec<_>>();
    let function_idents = function_keys
        .iter()
        .map(|key| message_function(key))
        .collect::<Vec<_>>();
    ensure_unique_identifiers(
        &function_keys,
        &function_idents,
        "message functions",
        declaration_span,
    )?;
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
                pub fn #function() -> #runtime::TypedMessage {
                    #runtime::TypedMessage::new(#key)
                }
            }
        } else {
            quote! {
                pub fn #function(
                    #(#params: impl Into<#runtime::I18nValue>),*
                ) -> #runtime::TypedMessage {
                    #runtime::TypedMessage::new(#key)
                        #(.with_arg(#names, #params))*
                }
            }
        }
    });

    let mut catalog_sources = locale_ids
        .iter()
        .zip(locale_paths.iter())
        .collect::<Vec<_>>();
    catalog_sources.sort_by_key(|(locale, _)| locale.as_str());
    let locale_catalogs = catalog_sources.iter().map(|(locale, path)| {
        quote! {
            #runtime::LocaleCatalog {
                id: #locale,
                source: include_str!(#path),
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

                pub fn tr(&self, message: #runtime::TypedMessage) -> String {
                    #runtime::translate(&CATALOG, self.locale.id(), message)
                }

                pub fn try_tr(
                    &self,
                    message: #runtime::TypedMessage,
                ) -> Result<String, #runtime::I18nError> {
                    #runtime::try_translate(&CATALOG, self.locale.id(), message)
                }
            }

            impl Default for I18n {
                fn default() -> Self {
                    Self::new(Locale::#fallback_variant)
                }
            }

            pub const FALLBACK_LOCALE: Locale = Locale::#fallback_variant;

            /// The static catalog for this i18n module. Public so dioxus context
            /// providers (e.g. `arkit_i18n::use_i18n_provider`) can reference it.
            pub static CATALOG: #runtime::Catalog = #runtime::Catalog {
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

#[derive(Debug, Clone, Default)]
struct PatternUsage {
    vars: BTreeSet<String>,
    references: Vec<ReferenceUsage>,
}

#[derive(Debug, Clone)]
struct ReferenceUsage {
    key: String,
    bound: BTreeSet<String>,
}

fn parse_ftl(source: &str) -> Result<BTreeMap<String, Template>, String> {
    let resource = parser::parse(source).map_err(|(_, errors)| {
        let errors = errors
            .into_iter()
            .map(|error| format!("{error:?}"))
            .collect::<Vec<_>>()
            .join("; ");
        format!("Fluent parse failed: {errors}")
    })?;
    let mut definitions = BTreeMap::<String, PatternUsage>::new();
    let mut public_keys = Vec::new();

    for entry in &resource.body {
        match entry {
            ast::Entry::Message(message) => {
                let id = message.id.name;
                if let Some(pattern) = &message.value {
                    insert_pattern(&mut definitions, id.to_string(), pattern)?;
                    public_keys.push(id.to_string());
                }
                for attribute in &message.attributes {
                    let key = format!("{id}.{}", attribute.id.name);
                    insert_pattern(&mut definitions, key.clone(), &attribute.value)?;
                    public_keys.push(key);
                }
            }
            ast::Entry::Term(term) => {
                let id = format!("-{}", term.id.name);
                insert_pattern(&mut definitions, id.clone(), &term.value)?;
                for attribute in &term.attributes {
                    insert_pattern(
                        &mut definitions,
                        format!("{id}.{}", attribute.id.name),
                        &attribute.value,
                    )?;
                }
            }
            ast::Entry::Junk { .. } => {
                return Err("Fluent resource contains an invalid junk entry".to_string());
            }
            ast::Entry::Comment(_)
            | ast::Entry::GroupComment(_)
            | ast::Entry::ResourceComment(_) => {}
        }
    }
    if public_keys.is_empty() {
        return Err("locale does not define any translatable messages".to_string());
    }

    let mut cache = BTreeMap::<String, BTreeSet<String>>::new();
    let mut messages = BTreeMap::new();
    for key in public_keys {
        let vars = resolve_vars(&key, &definitions, &mut cache, &mut BTreeSet::new())?;
        messages.insert(key, Template { vars });
    }
    Ok(messages)
}

fn insert_pattern(
    definitions: &mut BTreeMap<String, PatternUsage>,
    key: String,
    pattern: &ast::Pattern<&str>,
) -> Result<(), String> {
    let mut usage = PatternUsage::default();
    collect_pattern(pattern, &mut usage);
    if definitions.insert(key.clone(), usage).is_some() {
        return Err(format!("duplicate Fluent message or attribute `{key}`"));
    }
    Ok(())
}

fn resolve_vars(
    key: &str,
    definitions: &BTreeMap<String, PatternUsage>,
    cache: &mut BTreeMap<String, BTreeSet<String>>,
    visiting: &mut BTreeSet<String>,
) -> Result<BTreeSet<String>, String> {
    if let Some(vars) = cache.get(key) {
        return Ok(vars.clone());
    }
    if !visiting.insert(key.to_string()) {
        return Err(format!("cyclic Fluent reference involving `{key}`"));
    }
    let usage = definitions
        .get(key)
        .ok_or_else(|| format!("Fluent reference targets missing message or term `{key}`"))?;
    let mut vars = usage.vars.clone();
    for reference in &usage.references {
        let referenced = resolve_vars(&reference.key, definitions, cache, visiting)?;
        vars.extend(
            referenced
                .into_iter()
                .filter(|name| !reference.bound.contains(name)),
        );
    }
    visiting.remove(key);
    cache.insert(key.to_string(), vars.clone());
    Ok(vars)
}

fn collect_pattern(pattern: &ast::Pattern<&str>, usage: &mut PatternUsage) {
    for element in &pattern.elements {
        if let ast::PatternElement::Placeable { expression } = element {
            collect_expression(expression, usage);
        }
    }
}

fn collect_expression(expression: &ast::Expression<&str>, usage: &mut PatternUsage) {
    match expression {
        ast::Expression::Inline(expression) => collect_inline(expression, usage),
        ast::Expression::Select { selector, variants } => {
            collect_inline(selector, usage);
            for variant in variants {
                collect_pattern(&variant.value, usage);
            }
        }
    }
}

fn collect_inline(expression: &ast::InlineExpression<&str>, usage: &mut PatternUsage) {
    match expression {
        ast::InlineExpression::VariableReference { id } => {
            usage.vars.insert(id.name.to_string());
        }
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            collect_arguments(arguments, usage);
        }
        ast::InlineExpression::MessageReference { id, attribute } => {
            usage.references.push(ReferenceUsage {
                key: referenced_key(id.name, attribute.as_ref().map(|value| value.name), false),
                bound: BTreeSet::new(),
            });
        }
        ast::InlineExpression::TermReference {
            id,
            attribute,
            arguments,
        } => {
            let mut bound = BTreeSet::new();
            if let Some(arguments) = arguments {
                bound.extend(
                    arguments
                        .named
                        .iter()
                        .map(|argument| argument.name.name.to_string()),
                );
                collect_arguments(arguments, usage);
            }
            usage.references.push(ReferenceUsage {
                key: referenced_key(id.name, attribute.as_ref().map(|value| value.name), true),
                bound,
            });
        }
        ast::InlineExpression::Placeable { expression } => collect_expression(expression, usage),
        ast::InlineExpression::StringLiteral { .. }
        | ast::InlineExpression::NumberLiteral { .. } => {}
    }
}

fn collect_arguments(arguments: &ast::CallArguments<&str>, usage: &mut PatternUsage) {
    for argument in &arguments.positional {
        collect_inline(argument, usage);
    }
    for argument in &arguments.named {
        collect_inline(&argument.value, usage);
    }
}

fn referenced_key(id: &str, attribute: Option<&str>, term: bool) -> String {
    let mut key = if term {
        format!("-{id}")
    } else {
        id.to_string()
    };
    if let Some(attribute) = attribute {
        key.push('.');
        key.push_str(attribute);
    }
    key
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

fn ensure_unique_identifiers(
    sources: &[String],
    identifiers: &[Ident],
    kind: &str,
    span: Span,
) -> syn::Result<()> {
    let mut generated = BTreeMap::<String, &str>::new();
    for (source, identifier) in sources.iter().zip(identifiers) {
        let identifier = identifier.to_string();
        if let Some(previous) = generated.insert(identifier.clone(), source) {
            return Err(syn::Error::new(
                span,
                format!(
                    "{kind} `{previous}` and `{source}` both normalize to Rust identifier `{identifier}`"
                ),
            ));
        }
    }
    Ok(())
}

fn i18n_runtime_path(span: Span) -> syn::Result<TokenStream2> {
    if let Some(path) = dependency_path("arkit_i18n") {
        return Ok(path);
    }
    if let Some(framework) = dependency_path("arkit") {
        return Ok(quote!(#framework::i18n));
    }
    Err(syn::Error::new(
        span,
        "i18n! requires a direct dependency on `arkit_i18n` or the `arkit` facade with its `i18n` feature",
    ))
}

fn dependency_path(package: &str) -> Option<TokenStream2> {
    match crate_name(package).ok()? {
        FoundCrate::Itself => Some(quote!(crate)),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name.replace('-', "_"), Span::call_site());
            Some(quote!(::#ident))
        }
    }
}
