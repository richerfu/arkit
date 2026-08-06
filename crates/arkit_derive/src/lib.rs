use proc_macro::TokenStream;
use proc_macro2::{Ident as MacroIdent, Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Mark a function as the application entry point.
///
/// Generates OpenHarmony NAPI bindings (init / render / destroy lifecycle,
/// plus the pluginized bridge event ports) that call the entry function. The
/// entry function must take no arguments and return `Element`.
///
/// Business content fills the mounted surface edge-to-edge by default.
/// Safe-area avoidance is opt-in: read the insets with [`use_safe_area`]
/// (or the `SafeArea` component) and apply them where the layout needs them.
///
/// ## Pluginized initialization
///
/// The generated module follows the `openharmony-ability` mainline bridge
/// contract:
///
/// - `init` installs the ability init context, injects framework-owned bridge
///   plugins ([`arkit_runtime::inject_plugins`], e.g. the `ohos.webview`
///   facade when the `webview` feature is enabled), and creates the lifecycle
///   handle. `onBridgeSyncEvent` / `onBridgeLifecycle` are exported so the
///   ArkTS host can deliver main-thread plugin events and UI-context
///   readiness transitions into the Rust plugin registry.
/// - `render` calls `openharmony_ability::render` first: it installs the
///   bridge bindings (`bridgeInvoke`/`bridgeInvokeSync`/`bridgeDispatch`) and
///   mounts the native XComponent. Only then is the dioxus runtime mounted, so
///   plugin clients resolve from the first render onward.
///
/// ## How it works (for IDE / rust-analyzer)
///
/// The generated NAPI module is `pub` (with `#[doc(hidden)]`), so its
/// `render()` function is reachable from the crate root. Since `render()`
/// calls the user's entry function, rust-analyzer can trace the entire
/// call chain — no `#[allow(dead_code)]` needed.
#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[entry] takes no options; safe-area avoidance is opt-in through `use_safe_area`",
        )
        .to_compile_error()
        .into();
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
    let safe_area_policy = quote!(#framework::SafeAreaPolicy::EdgeToEdge);

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
                // Renderer-owned root created by `openharmony_ability::render`
                // (native XComponent + bridge bindings). Kept alive for the
                // module lifetime; dropping it would unmount the XComponent.
                static ROOT_NODE: RefCell<Option<#framework::openharmony_ability::arkui::RootNode>> =
                    RefCell::new(None);
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
                (*APP).set_init_context(init_context);
                // Framework-owned plugin injection (e.g. the `ohos.webview`
                // bridge facade under the `webview` feature). Runs during the
                // ability-init stage, before any ArkTS plugin event is
                // delivered to the registry.
                #framework::arkit_runtime::inject_plugins(&(*APP));
                #framework::openharmony_ability::create_lifecycle_handle(env, (*APP).clone())
            }

            #[#framework::napi_derive_ohos::napi]
            pub fn render<'a>(
                env: &'a #framework::napi_ohos::Env,
                bindings: #framework::napi_ohos::bindgen_prelude::ObjectRef,
                #[napi(ts_arg_type = "NodeContent")] slot: #framework::ohos_arkui_binding::common::handle::ArkUIHandle,
            ) -> #framework::napi_ohos::Result<()> {
                RUNTIME.with(|state| -> #framework::napi_ohos::Result<()> {
                    if state.borrow().is_some() {
                        // Already mounted — the OHOS entrypoint only mounts once.
                        return Ok(());
                    }
                    // Pluginized bridge initialization: installs the bridge
                    // bindings and mounts the native XComponent before the
                    // dioxus tree is built, so plugin clients resolve from the
                    // first render onward.
                    let root = #framework::openharmony_ability::render(
                        env,
                        bindings,
                        slot,
                        (*APP).clone(),
                    )?;
                    ROOT_NODE.with(|root_node| root_node.replace(Some(root)));

                    // `#fn_name` is the user's root component `fn() -> Element`.
                    // The runtime creates a VirtualDom from it and rebuilds
                    // into an ArkUIRenderer mounted on `slot`.
                    let runtime = #framework::mount_entry_with_policy(
                        slot,
                        (*APP).clone(),
                        #fn_name,
                        #safe_area_policy,
                    )?;
                    state.replace(Some(runtime));
                    Ok(())
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

            /// Synchronous ArkTS platform callback -> Rust plugin decision port.
            ///
            /// The N-API value is scoped to this call; the returned value is
            /// produced before ArkTS resumes the originating platform callback.
            #[#framework::napi_derive_ohos::napi]
            pub fn on_bridge_sync_event<'a>(
                env: &'a #framework::napi_ohos::Env,
                plugin_id: String,
                event: String,
                request_type_name: String,
                response_type_name: String,
                value: #framework::napi_ohos::bindgen_prelude::Unknown<'a>,
            ) -> #framework::napi_ohos::Result<#framework::napi_ohos::bindgen_prelude::Unknown<'a>> {
                let event = #framework::openharmony_ability::BridgeMainThreadEvent::new(
                    env,
                    plugin_id,
                    event,
                    request_type_name,
                    response_type_name,
                    value,
                )?;
                (*APP).dispatch_bridge_main_thread_event(event)
            }

            /// ArkTS-only lifecycle transitions, currently UI-context readiness.
            #[#framework::napi_derive_ohos::napi]
            pub fn on_bridge_lifecycle(kind: String) -> #framework::napi_ohos::Result<()> {
                let event =
                    #framework::openharmony_ability::PluginLifecycleEvent::from_arkts(&kind)?;
                (*APP).dispatch_plugin_lifecycle(event)
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
