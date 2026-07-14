use proc_macro::TokenStream;
use proc_macro2::{Ident as MacroIdent, Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Ident, ItemFn, Token};

/// Mark a function as the application entry point.
///
/// Generates OpenHarmony NAPI bindings (init / render / destroy lifecycle)
/// that call the entry function. The entry function must take no arguments
/// and return `Element`.
///
/// `#[entry]` keeps business content in the framework safe viewport.
/// `#[entry(edge_to_edge)]` lets it fill the mounted XComponent surface while
/// retaining window metrics and framework-owned overlay avoidance.
///
/// ## How it works (for IDE / rust-analyzer)
///
/// The generated NAPI module is `pub` (with `#[doc(hidden)]`), so its
/// `render()` function is reachable from the crate root. Since `render()`
/// calls the user's entry function, rust-analyzer can trace the entire
/// call chain — no `#[allow(dead_code)]` needed.
#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr with Punctuated::<Ident, Token![,]>::parse_terminated);
    let mut edge_to_edge = false;
    for arg in args {
        if arg == "edge_to_edge" {
            edge_to_edge = true;
        } else {
            return syn::Error::new_spanned(arg, "unsupported #[entry] option")
                .to_compile_error()
                .into();
        }
    }
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
        return syn::Error::new_spanned(input.sig.asyncness, "#[entry] function must not be async")
            .to_compile_error()
            .into();
    }

    let fn_name = input.sig.ident.clone();
    let framework = match dependency_path("arkit") {
        Ok(path) => path,
        Err(error) => return error.to_compile_error().into(),
    };
    let safe_area_policy = if edge_to_edge {
        quote!(#framework::SafeAreaPolicy::EdgeToEdge)
    } else {
        quote!(#framework::SafeAreaPolicy::Safe)
    };

    let expanded = quote! {
        #input

        #[doc(hidden)]
        pub mod __arkit_entry_mod {
            use super::*;
            use std::cell::RefCell;
            use std::sync::LazyLock;

            static APP: LazyLock<#framework::openharmony_ability::OpenHarmonyApp> =
                LazyLock::new(#framework::openharmony_ability::OpenHarmonyApp::new);

            thread_local! {
                static RUNTIME: RefCell<Option<#framework::ArkRuntime>> = RefCell::new(None);
            }

            #[#framework::napi_derive_ohos::napi]
            pub fn on_back_press_intercept() -> bool {
                (*APP).get_back_press_interceptor()
            }

            #[#framework::napi_derive_ohos::napi]
            pub fn init<'a>(
                env: &'a #framework::napi_ohos::Env,
                #[napi(ts_arg_type = "AbilityInitContext")]
                context: Option<#framework::napi_ohos::bindgen_prelude::Object<'a>>,
            ) -> #framework::napi_ohos::Result<#framework::openharmony_ability::ApplicationLifecycle<'a>> {
                let init_context =
                    #framework::openharmony_ability::AbilityInitContext::from_object(context.as_ref())?;
                let resource_manager =
                    #framework::openharmony_ability::ResourceManager::from_init_context(*env, context.as_ref())?;

                (*APP).set_init_context(init_context);
                (*APP).set_resource_manager(resource_manager);

                #framework::openharmony_ability::create_lifecycle_handle(env, (*APP).clone())
            }

            #[#framework::napi_derive_ohos::napi]
            pub fn render<'a>(
                env: &'a #framework::napi_ohos::Env,
                helper: #framework::napi_ohos::bindgen_prelude::ObjectRef,
                #[napi(ts_arg_type = "NodeContent")] slot: #framework::ohos_arkui_binding::common::handle::ArkUIHandle,
            ) -> #framework::napi_ohos::Result<()> {
                #framework::openharmony_ability::set_helper(helper);
                #framework::openharmony_ability::set_main_thread_env(*env);
                let _ = #framework::openharmony_ability::create_permission_request_tsfn(env);

                RUNTIME.with(|state| -> #framework::napi_ohos::Result<()> {
                    let mut runtime_state = state.borrow_mut();
                    if runtime_state.is_some() {
                        // Already mounted — the OHOS entrypoint only mounts once.
                        Ok(())
                    } else {
                        // `#fn_name` is the user's root component `fn() -> Element`.
                        // The runtime creates a VirtualDom from it and rebuilds
                        // into an ArkUIRenderer mounted on `slot`.
                        let runtime = #framework::mount_entry_with_policy(
                            slot,
                            (*APP).clone(),
                            #fn_name,
                            #safe_area_policy,
                        )?;
                        runtime_state.replace(runtime);
                        Ok(())
                    }
                })
            }

            #[#framework::napi_derive_ohos::napi]
            pub fn destroy() -> #framework::napi_ohos::Result<()> {
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

fn dependency_path(package: &str) -> syn::Result<TokenStream2> {
    match crate_name(package) {
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(FoundCrate::Name(name)) => {
            let ident = MacroIdent::new(&name.replace('-', "_"), Span::call_site());
            Ok(quote!(::#ident))
        }
        Err(error) => Err(syn::Error::new(
            Span::call_site(),
            format!(
                "#[entry] requires a dependency on package `{package}` (dependency lookup failed: {error})"
            ),
        )),
    }
}
