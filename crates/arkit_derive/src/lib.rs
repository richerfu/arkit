use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::{BTreeMap, BTreeSet};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, ItemFn, LitStr, Type};

/// Mark a function as the application entry point.
///
/// Generates OpenHarmony NAPI bindings (init / render / destroy lifecycle)
/// that call the entry function. The entry function must take no arguments
/// and return `Element`.
///
/// ## How it works (for IDE / rust-analyzer)
///
/// The generated NAPI module is `pub` (with `#[doc(hidden)]`), so its
/// `render()` function is reachable from the crate root. Since `render()`
/// calls the user's entry function, rust-analyzer can trace the entire
/// call chain — no `#[allow(dead_code)]` needed.
#[proc_macro_attribute]
pub fn entry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    if !input.sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            &input.sig.inputs,
            "#[entry] function must not have arguments",
        )
        .to_compile_error()
        .into();
    }

    if input.sig.asyncness.is_some() {
        return syn::Error::new_spanned(
            &input.sig.asyncness,
            "#[entry] function must not be async",
        )
        .to_compile_error()
        .into();
    }

    let fn_name = input.sig.ident.clone();

    let expanded = quote! {
        #input

        #[doc(hidden)]
        pub mod __arkit_entry_mod {
            use super::*;
            use std::cell::RefCell;
            use std::sync::LazyLock;

            static APP: LazyLock<::arkit::openharmony_ability::OpenHarmonyApp> =
                LazyLock::new(::arkit::openharmony_ability::OpenHarmonyApp::new);

            thread_local! {
                static RUNTIME: RefCell<Option<Box<dyn ::arkit::MountedEntryHandle>>> = RefCell::new(None);
            }

            #[::arkit::napi_derive_ohos::napi]
            pub fn on_back_press_intercept() -> bool {
                (*APP).get_back_press_interceptor()
            }

            #[::arkit::napi_derive_ohos::napi]
            pub fn init<'a>(
                env: &'a ::arkit::napi_ohos::Env,
                #[napi(ts_arg_type = "AbilityInitContext")]
                context: Option<::arkit::napi_ohos::bindgen_prelude::Object<'a>>,
            ) -> ::arkit::napi_ohos::Result<::arkit::openharmony_ability::ApplicationLifecycle<'a>> {
                let init_context =
                    ::arkit::openharmony_ability::AbilityInitContext::from_object(context.as_ref())?;
                let resource_manager =
                    ::arkit::openharmony_ability::ResourceManager::from_init_context(*env, context.as_ref())?;

                (*APP).set_init_context(init_context);
                (*APP).set_resource_manager(resource_manager);

                ::arkit::openharmony_ability::create_lifecycle_handle(env, (*APP).clone())
            }

            #[::arkit::napi_derive_ohos::napi]
            pub fn render<'a>(
                env: &'a ::arkit::napi_ohos::Env,
                helper: ::arkit::napi_ohos::bindgen_prelude::ObjectRef,
                #[napi(ts_arg_type = "NodeContent")] slot: ::arkit::ohos_arkui_binding::common::handle::ArkUIHandle,
            ) -> ::arkit::napi_ohos::Result<()> {
                ::arkit::openharmony_ability::set_helper(helper);
                ::arkit::openharmony_ability::set_main_thread_env(*env);
                let _ = ::arkit::openharmony_ability::create_permission_request_tsfn(env);

                RUNTIME.with(|state| -> ::arkit::napi_ohos::Result<()> {
                    let mut runtime_state = state.borrow_mut();
                    if runtime_state.is_some() {
                        // Already mounted — the OHOS entrypoint only mounts once.
                        Ok(())
                    } else {
                        let runtime = ::arkit::mount_entry(slot, (*APP).clone(), #fn_name())?;
                        runtime_state.replace(runtime);
                        Ok(())
                    }
                })
            }

            #[::arkit::napi_derive_ohos::napi]
            pub fn destroy() -> ::arkit::napi_ohos::Result<()> {
                RUNTIME.with(|state| {
                    if let Some(runtime) = state.borrow_mut().take() {
                        runtime.unmount()?;
                    }
                    Ok(())
                })
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(StructuredRoute, attributes(route))]
pub fn derive_structured_route(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    match expand_structured_route(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[derive(Default)]
struct RouteAttrs {
    path: Option<LitStr>,
    name: Option<LitStr>,
    param: Option<LitStr>,
}

struct RouteField {
    ident: Ident,
    ty: Type,
    param: String,
}

fn expand_structured_route(input: DeriveInput) -> syn::Result<TokenStream2> {
    let attrs = parse_route_attrs(&input.attrs)?;
    let path_lit = attrs.path.ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            "StructuredRoute requires #[route(path = \"...\")]",
        )
    })?;
    let path = normalize_route_path(&path_lit.value())
        .map_err(|message| syn::Error::new_spanned(&path_lit, message))?;
    let path_lit = LitStr::new(&path, path_lit.span());
    let route_params =
        route_path_params(&path).map_err(|message| syn::Error::new_spanned(&path_lit, message))?;

    let fields = match input.data {
        Data::Struct(data) => parse_route_fields(&data.fields)?,
        _ => {
            return Err(syn::Error::new_spanned(
                input.ident,
                "StructuredRoute can only be derived for structs",
            ));
        }
    };

    validate_route_fields(&input.ident, &fields, &route_params)?;

    let ident = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let path_build = build_path_tokens(&path, &fields);
    let definition = if let Some(name) = attrs.name.as_ref() {
        quote! {
            ::arkit::router::RouteDefinition::named(#name, #path_lit)
                .expect("route definition generated by StructuredRoute derive")
        }
    } else {
        quote! {
            ::arkit::router::RouteDefinition::new(#path_lit)
                .expect("route definition generated by StructuredRoute derive")
        }
    };
    let route_match = if let Some(name) = attrs.name.as_ref() {
        quote! {
            if route.name() != Some(#name) {
                return None;
            }
        }
    } else {
        quote! {
            if route.pattern() != #path_lit {
                return None;
            }
        }
    };
    let from_route = build_from_route_tokens(&ident, &fields);

    Ok(quote! {
        impl #impl_generics ::arkit::router::StructuredRoute for #ident #ty_generics #where_clause {
            fn definition() -> ::arkit::router::RouteDefinition {
                #definition
            }

            fn path(&self) -> ::std::string::String {
                #path_build
            }

            fn from_route(route: &::arkit::router::Route) -> ::std::option::Option<Self> {
                #route_match
                #from_route
            }
        }
    })
}

fn parse_route_attrs(attrs: &[Attribute]) -> syn::Result<RouteAttrs> {
    let mut parsed = RouteAttrs::default();
    for attr in attrs {
        if !attr.path().is_ident("route") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("path") {
                parsed.path = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if meta.path.is_ident("name") {
                parsed.name = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if meta.path.is_ident("param") {
                parsed.param = Some(meta.value()?.parse()?);
                return Ok(());
            }

            Err(meta.error("unsupported route attribute"))
        })?;
    }
    Ok(parsed)
}

fn parse_route_fields(fields: &Fields) -> syn::Result<Vec<RouteField>> {
    match fields {
        Fields::Unit => Ok(Vec::new()),
        Fields::Named(named) => named
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.clone().expect("named field has ident");
                let attrs = parse_route_attrs(&field.attrs)?;
                let param = attrs
                    .param
                    .map(|param| param.value())
                    .unwrap_or_else(|| ident.to_string());
                Ok(RouteField {
                    ident,
                    ty: field.ty.clone(),
                    param,
                })
            })
            .collect(),
        Fields::Unnamed(fields) => Err(syn::Error::new_spanned(
            fields,
            "StructuredRoute only supports unit structs and named-field structs",
        )),
    }
}

fn validate_route_fields(
    ident: &Ident,
    fields: &[RouteField],
    route_params: &[String],
) -> syn::Result<()> {
    let params = route_params.iter().cloned().collect::<BTreeSet<_>>();
    let field_params = fields
        .iter()
        .map(|field| field.param.clone())
        .collect::<BTreeSet<_>>();

    for param in &params {
        if !field_params.contains(param) {
            return Err(syn::Error::new_spanned(
                ident,
                format!("route path param `{param}` has no matching struct field"),
            ));
        }
    }

    for field in fields {
        if !params.contains(&field.param) {
            return Err(syn::Error::new_spanned(
                &field.ident,
                format!(
                    "route field `{}` does not match any path param",
                    field.ident
                ),
            ));
        }
    }

    Ok(())
}

fn build_path_tokens(path: &str, fields: &[RouteField]) -> TokenStream2 {
    let fields_by_param = fields
        .iter()
        .map(|field| (field.param.as_str(), &field.ident))
        .collect::<BTreeMap<_, _>>();
    let segments = route_segments(path);

    if segments.is_empty() {
        return quote! { ::std::string::String::from("/") };
    }

    let push_segments = segments.into_iter().map(|segment| {
        if let Some(param) = segment
            .strip_prefix(':')
            .or_else(|| segment.strip_prefix('*'))
        {
            let param = if param.is_empty() { "wildcard" } else { param };
            let ident = fields_by_param
                .get(param)
                .expect("validated route param exists");
            quote! {
                __path.push('/');
                __path.push_str(&self.#ident.to_string());
            }
        } else {
            quote! {
                __path.push('/');
                __path.push_str(#segment);
            }
        }
    });

    quote! {
        let mut __path = ::std::string::String::new();
        #(#push_segments)*
        __path
    }
}

fn build_from_route_tokens(ident: &Ident, fields: &[RouteField]) -> TokenStream2 {
    if fields.is_empty() {
        return quote! { ::std::option::Option::Some(Self) };
    }

    let initializers = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        let param = &field.param;
        quote! {
            #field_ident: route.param(#param)?.parse::<#field_ty>().ok()?
        }
    });

    quote! {
        ::std::option::Option::Some(#ident {
            #(#initializers,)*
        })
    }
}

fn normalize_route_path(path: &str) -> Result<String, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("route path cannot be empty".to_string());
    }

    let mut normalized = String::new();
    let with_leading_slash = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    for segment in with_leading_slash
        .split('/')
        .filter(|segment| !segment.is_empty())
    {
        normalized.push('/');
        normalized.push_str(segment);
    }

    if normalized.is_empty() {
        Ok("/".to_string())
    } else {
        Ok(normalized)
    }
}

fn route_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn route_path_params(path: &str) -> Result<Vec<String>, String> {
    let mut params = Vec::new();
    for segment in route_segments(path) {
        if let Some(param) = segment.strip_prefix(':') {
            if param.is_empty() {
                return Err("route path contains an empty param name".to_string());
            }
            params.push(param.to_string());
        } else if let Some(param) = segment.strip_prefix('*') {
            let param = if param.is_empty() { "wildcard" } else { param };
            params.push(param.to_string());
        }
    }
    Ok(params)
}
