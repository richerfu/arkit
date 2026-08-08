use proc_macro::TokenStream;
use proc_macro2::{Ident as MacroIdent, Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::parse::Parser;
use syn::{parse_macro_input, ItemFn};

/// Mark a function as the application entry point.
///
/// Generates OpenHarmony NAPI bindings (init / render / destroy lifecycle,
/// plus the pluginized bridge event ports) that call the entry function. The
/// entry function returns `Element` and either takes no arguments, or a
/// single [`OpenHarmonyApp`] handle received at mount time (see "Custom
/// bridge plugins" below).
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
///   facade when the `webview` feature is enabled), registers the application
///   plugins declared with `plugins = [...]`, and creates the lifecycle
///   handle. `onBridgeSyncEvent` / `onBridgeLifecycle` are exported so the
///   ArkTS host can deliver main-thread plugin events and UI-context
///   readiness transitions into the Rust plugin registry.
/// - `render` calls `openharmony_ability::render` first: it installs the
///   bridge bindings (`bridgeInvoke`/`bridgeInvokeSync`/`bridgeDispatch`) and
///   mounts the native XComponent. Only then is the dioxus runtime mounted, so
///   plugin clients resolve from the first render onward.
///
/// ## Custom bridge plugins
///
/// Applications register their own `openharmony_ability::BridgePlugin`
/// facades in either of two composable ways:
///
/// - Declarative list: `#[entry(plugins = [MyPlugin, UrlBridgePlugin])]`.
///   Each item is an expression evaluated inside the generated `init`, after
///   the framework-owned plugins and before any ArkTS plugin event is
///   delivered. Items resolve at module scope, so entry-function locals
///   cannot be referenced; unit types and constructor calls both work.
///   Registration failures are logged, not fatal.
/// - App handle argument: `fn app(handle: OpenHarmonyApp) -> Element`.
///   The handle is a clone of the shared ability and is passed at first
///   render (after the bridge bindings are installed), so
///   `handle.register_plugin(...)` may run arbitrary setup. Late registration
///   is safe: the registry replays the bounded lifecycle history to plugins
///   whose `REQUIRED_CONTEXTS` are already satisfied.
///
/// A plugin implements [`openharmony_ability::BridgePlugin`] (`type Mode =
/// AsyncBridge` or `MainThreadSyncBridge`, `ID`, optional `REQUIRED_CONTEXTS`
/// and lifecycle / main-thread-event hooks). Its ArkTS counterpart must be
/// installed in the host ability's `bridgePlugins` array — that side is
/// hand-managed; this macro cannot touch ArkTS sources.
///
/// ## How it works (for IDE / rust-analyzer)
///
/// The generated NAPI module is `pub` (with `#[doc(hidden)]`), so its
/// `render()` function is reachable from the crate root. Since `render()`
/// calls the user's entry function, rust-analyzer can trace the entire
/// call chain — no `#[allow(dead_code)]` needed.
#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let plugins = match parse_entry_options(attr) {
        Ok(plugins) => plugins,
        Err(error) => return error.to_compile_error().into(),
    };
    let input = parse_macro_input!(item as ItemFn);

    if input.sig.inputs.len() > 1 {
        return syn::Error::new_spanned(
            &input.sig.inputs,
            "#[entry] function must have at most one argument (an `OpenHarmonyApp` handle)",
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
    let has_app_arg = input.sig.inputs.len() == 1;
    // The fn passed to `mount_entry_with_policy` (which takes
    // `root: fn() -> Element`): the user fn itself for zero-arg entries, the
    // generated adapter otherwise.
    let root_fn = if has_app_arg {
        quote!(__arkit_entry_root)
    } else {
        quote!(#fn_name)
    };
    let entry_adapter = if has_app_arg {
        quote! {
            // One-argument entry roots receive the shared app handle at
            // mount time (inside the `render` NAPI call, after the bridge
            // bindings are installed). Registering plugins here is safe:
            // the registry replays the bounded lifecycle history to plugins
            // whose required contexts are already ready.
            fn __arkit_entry_root() -> #framework::Element {
                #fn_name((*APP).clone())
            }
        }
    } else {
        TokenStream2::new()
    };
    let plugin_registrations = plugins.iter().map(|plugin| {
        quote! {
            #framework::arkit_runtime::register_user_plugin(&(*APP), #plugin);
        }
    });

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

            #entry_adapter

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
                // Application-owned bridge plugins declared with
                // `#[entry(plugins = [...])]` — after the framework-owned
                // set, still before any plugin event reaches the registry.
                #(#plugin_registrations)*
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

                    // `#root_fn` is the user's root component `fn() -> Element`,
                    // or the generated adapter when the entry fn takes the
                    // app handle. The runtime creates a VirtualDom from it and
                    // rebuilds into an ArkUIRenderer mounted on `slot`.
                    let runtime = #framework::mount_entry_with_policy(
                        slot,
                        (*APP).clone(),
                        #root_fn,
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

/// Parses `#[entry(plugins = [plugin, ...])]` into the plugin expressions.
///
/// Only the `plugins` option is supported. Each element is an arbitrary
/// expression (a unit-struct path, a constructor call, ...) evaluated inside
/// the generated `init`; type errors surface naturally at the generated
/// `register_plugin` call site.
fn parse_entry_options(attr: TokenStream) -> syn::Result<Vec<syn::Expr>> {
    if attr.is_empty() {
        return Ok(Vec::new());
    }
    let mut plugins = Vec::new();
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("plugins") {
            if !meta.input.peek(syn::Token![=]) {
                return Err(meta.error("expected `plugins = [plugin, ...]`"));
            }
            let value: syn::Expr = meta.value()?.parse()?;
            match value {
                syn::Expr::Array(array) => {
                    plugins.extend(array.elems);
                    Ok(())
                }
                value => Err(syn::Error::new_spanned(
                    value,
                    "expected `plugins = [plugin, ...]`",
                )),
            }
        } else {
            Err(syn::Error::new_spanned(
                &meta.path,
                "unsupported #[entry] option; the only supported option is `plugins = [plugin, ...]`",
            ))
        }
    });
    parser.parse2(attr.into())?;
    Ok(plugins)
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
