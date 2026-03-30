use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Mark a function as a reactive component.
///
/// The function body runs **once** inside a child reactive owner scope.
/// All subsequent updates are driven by reactive signals via effects —
/// aligned with the SolidJS component model.
///
/// ## How it works (for IDE / rust-analyzer)
///
/// The macro uses a **rename + wrapper** pattern inspired by Leptos:
///
/// 1. The original function is renamed to `__component_{snake_case}`
///    and marked `#[doc(hidden)]`.
/// 2. A new wrapper function with the original name is generated;
///    it enters a reactive scope, calls the renamed body, and returns
///    a scoped element.
///
/// Since the wrapper directly calls the renamed body, rust-analyzer can
/// trace the full call chain without any `#[allow(dead_code)]` workarounds.
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let vis = &input.vis;
    let sig = &input.sig;
    let attrs = &input.attrs;
    let body = &input.block;
    let fn_name = &sig.ident;

    // Rename the original function so we can call it from the wrapper.
    let body_name = quote::format_ident!("__component_{}", fn_name);
    let mut body_sig = sig.clone();
    body_sig.ident = body_name.clone();

    // Collect parameter names so the wrapper can forward them to the renamed body.
    let param_names: Vec<_> = sig.inputs.iter().map(|arg| {
        match arg {
            syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                syn::Pat::Ident(ident) => &ident.ident,
                _ => panic!("unsupported parameter pattern"),
            },
            syn::FnArg::Receiver(_) => panic!("#[component] does not support self parameters"),
        }
    }).collect();

    let expanded = quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #(#attrs)*
        #vis #body_sig {
            #body
        }

        #vis #sig {
            let __arkit_scope_guard = ::arkit::enter_scope();
            let __arkit_result = #body_name(#(#param_names),*);
            let __arkit_child_owner = __arkit_scope_guard.exit();
            ::arkit::scope_owned(__arkit_child_owner, __arkit_result)
        }
    };

    expanded.into()
}

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
                static RUNTIME: RefCell<Option<::arkit::Runtime>> = RefCell::new(None);
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
                let _ = env;
                let _ = helper;

                RUNTIME.with(|state| -> ::arkit::napi_ohos::Result<()> {
                    let mut runtime_state = state.borrow_mut();
                    if runtime_state.is_some() {
                        // Already mounted — SolidJS model renders once.
                        Ok(())
                    } else {
                        let runtime = ::arkit::Runtime::new(slot, (*APP).clone(), || #fn_name().into())?;
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
