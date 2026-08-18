use proc_macro::TokenStream;
use proc_macro2::{Ident as MacroIdent, Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::parse::Parser;
use syn::{parse_macro_input, ItemFn};

/// Mark a function as the application entry point.
///
/// Generates OpenHarmony NAPI bindings (Ability-session init/dispose and
/// component render/dispose lifecycle, plus the pluginized bridge event
/// ports) that call the entry function. The entry function returns `Element`
/// and either takes no arguments, or a single [`OpenHarmonyApp`] handle
/// received at mount time (see "Custom bridge plugins" below).
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
/// - `init` installs the Ability-session bridge bindings
///   (`bridgeInvoke`/`bridgeInvokeSync`/`bridgeDispatch`) and init context,
///   injects framework-owned bridge plugins
///   ([`arkit_runtime::inject_plugins`], e.g. the `ohos.webview` facade when
///   the `webview` feature is enabled), registers the application plugins
///   declared with `plugins = [...]`, and creates the lifecycle handle.
///   Process-wide plugin registration runs only once even if OpenHarmony
///   recreates the Ability while the native module stays loaded.
/// - `render` mounts the native XComponent and then the dioxus runtime under a
///   render-owner token. `disposeRender` and `disposeAllRenders` unmount both
///   trees without releasing the independent Ability-session bridge.
/// - `onBridgeSyncEvent` / `onBridgeLifecycle` are exported so the ArkTS host
///   can deliver main-thread plugin events and UI-context readiness
///   transitions into the Rust plugin registry.
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
///   The handle is a clone of the shared ability and is passed at render time
///   (after the Ability-session bridge is installed), so
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
            // mount time (inside the `render` NAPI call, after the
            // Ability-session bridge is installed). Registering plugins here is safe:
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
            #framework::__private::arkit_runtime::register_user_plugin(&(*APP), #plugin);
        }
    });

    let expanded = quote! {
        #input

        #[doc(hidden)]
        pub mod __arkit_entry_mod {
            use super::*;
            use std::cell::RefCell;
            use std::sync::LazyLock;

            static APP: LazyLock<#framework::__private::openharmony_ability::OpenHarmonyApp> =
                LazyLock::new(#framework::__private::openharmony_ability::OpenHarmonyApp::new);
            static APP_CONFIGURED: std::sync::OnceLock<()> = std::sync::OnceLock::new();

            struct BridgeSessionInitGuard {
                owner: Option<String>,
            }

            impl BridgeSessionInitGuard {
                fn new(owner: String) -> Self {
                    Self { owner: Some(owner) }
                }

                fn disarm(&mut self) {
                    self.owner = None;
                }
            }

            impl Drop for BridgeSessionInitGuard {
                fn drop(&mut self) {
                    if let Some(owner) = self.owner.take() {
                        (*APP).release_bridge_session(&owner);
                    }
                }
            }

            thread_local! {
                static RUNTIME: RefCell<Option<#framework::ArkRuntime>> = RefCell::new(None);
                // Renderer-owned root created by `openharmony_ability::render`
                // (native XComponent). The owner token makes delayed component
                // cleanup unable to tear down a replacement render.
                static ROOT_NODE: RefCell<Option<(String, #framework::__private::openharmony_ability::arkui::RootNode)>> =
                    RefCell::new(None);
            }

            fn dispose_owned_render(expected_owner: Option<&str>) -> #framework::__private::napi_ohos::Result<()> {
                let owns_render = ROOT_NODE.with(|root_node| {
                    let root_node = root_node.borrow();
                    match (expected_owner, root_node.as_ref()) {
                        (Some(expected), Some((owner, _))) => owner == expected,
                        (Some(_), None) => false,
                        (None, Some(_)) => true,
                        (None, None) => false,
                    }
                });
                if !owns_render {
                    return Ok(());
                }

                // The dioxus tree owns nodes in the shared slot, so unmount it
                // before dropping the XComponent root. Always release the
                // render owner even when renderer cleanup reports an error.
                let unmount_result = RUNTIME.with(|state| {
                    if let Some(runtime) = state.borrow_mut().take() {
                        runtime.unmount()
                    } else {
                        Ok(())
                    }
                });
                let root = ROOT_NODE.with(|root_node| root_node.borrow_mut().take());
                if let Some((owner, root)) = root {
                    drop(root);
                    (*APP).release_render(&owner);
                }
                unmount_result
            }

            #entry_adapter

            #[#framework::__private::napi_derive_ohos::napi]
            pub fn on_back_press_intercept() -> bool {
                (*APP).get_back_press_interceptor()
            }

            #[#framework::__private::napi_derive_ohos::napi]
            pub fn init<'a>(
                env: &'a #framework::__private::napi_ohos::Env,
                bindings: #framework::__private::napi_ohos::bindgen_prelude::ObjectRef,
                bridge_owner: String,
                #[napi(ts_arg_type = "AbilityInitContext")]
                context: Option<#framework::__private::napi_ohos::bindgen_prelude::Object<'a>>,
            ) -> #framework::__private::napi_ohos::Result<#framework::__private::openharmony_ability::ApplicationLifecycle<'a>> {
                let init_context =
                    #framework::__private::openharmony_ability::AbilityInitContext::from_object(context.as_ref())?;
                #framework::__private::openharmony_ability::attach_bridge_session(
                    env,
                    bindings,
                    &bridge_owner,
                    &(*APP),
                )?;
                let mut bridge_guard = BridgeSessionInitGuard::new(bridge_owner);
                (*APP).set_init_context(init_context);
                // The native module can outlive an Ability instance. Configure
                // its process-wide registry once, while refreshing the bridge,
                // init context, and lifecycle handle for every new session.
                APP_CONFIGURED.get_or_init(|| {
                    #framework::__private::arkit_runtime::inject_plugins(&(*APP));
                    #(#plugin_registrations)*
                });
                let lifecycle =
                    #framework::__private::openharmony_ability::create_lifecycle_handle(env, (*APP).clone())?;
                bridge_guard.disarm();
                Ok(lifecycle)
            }

            /// Releases only the matching Ability-session transport. A stale
            /// owner cannot clear endpoints installed for a later session.
            #[#framework::__private::napi_derive_ohos::napi]
            pub fn dispose_bridge(bridge_owner: String) {
                (*APP).release_bridge_session(&bridge_owner);
            }

            #[#framework::__private::napi_derive_ohos::napi]
            pub fn render<'a>(
                env: &'a #framework::__private::napi_ohos::Env,
                #[napi(ts_arg_type = "NodeContent")] slot: #framework::__private::ohos_arkui_binding::common::handle::ArkUIHandle,
                render_owner: String,
            ) -> #framework::__private::napi_ohos::Result<()> {
                if render_owner.is_empty() {
                    return Err(#framework::__private::napi_ohos::Error::from_reason(
                        "renderOwner must not be empty",
                    ));
                }
                RUNTIME.with(|state| -> #framework::__private::napi_ohos::Result<()> {
                    if state.borrow().is_some()
                        || ROOT_NODE.with(|root_node| root_node.borrow().is_some())
                    {
                        return Err(#framework::__private::napi_ohos::Error::from_reason(
                            "This native module is already rendered by another DefaultXComponent; use a distinct native module for every active component",
                        ));
                    }
                    // The bridge session already exists from `init`; rendering
                    // owns only the XComponent and dioxus trees.
                    let root = #framework::__private::openharmony_ability::render(
                        env,
                        slot,
                        render_owner.clone(),
                        (*APP).clone(),
                    )?;

                    // `#root_fn` is the user's root component `fn() -> Element`,
                    // or the generated adapter when the entry fn takes the
                    // app handle. The runtime creates a VirtualDom from it and
                    // rebuilds into an ArkUIRenderer mounted on `slot`.
                    let runtime = match #framework::mount_entry_with_policy(
                        slot,
                        (*APP).clone(),
                        #root_fn,
                        #safe_area_policy,
                    ) {
                        Ok(runtime) => runtime,
                        Err(error) => {
                            drop(root);
                            (*APP).release_render(&render_owner);
                            return Err(error);
                        }
                    };
                    ROOT_NODE.with(|root_node| {
                        root_node.replace(Some((render_owner, root)));
                    });
                    state.replace(Some(runtime));
                    Ok(())
                })
            }

            #[#framework::__private::napi_derive_ohos::napi]
            pub fn dispose_render(render_owner: String) -> #framework::__private::napi_ohos::Result<()> {
                dispose_owned_render(Some(&render_owner))
            }

            #[#framework::__private::napi_derive_ohos::napi]
            pub fn dispose_all_renders() -> #framework::__private::napi_ohos::Result<()> {
                dispose_owned_render(None)
            }

            /// Synchronous ArkTS platform callback -> Rust plugin decision port.
            ///
            /// The N-API value is scoped to this call; the returned value is
            /// produced before ArkTS resumes the originating platform callback.
            #[#framework::__private::napi_derive_ohos::napi]
            pub fn on_bridge_sync_event<'a>(
                env: &'a #framework::__private::napi_ohos::Env,
                plugin_id: String,
                event: String,
                request_type_name: String,
                response_type_name: String,
                value: #framework::__private::napi_ohos::bindgen_prelude::Unknown<'a>,
            ) -> #framework::__private::napi_ohos::Result<#framework::__private::napi_ohos::bindgen_prelude::Unknown<'a>> {
                let event = #framework::__private::openharmony_ability::BridgeMainThreadEvent::new(
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
            #[#framework::__private::napi_derive_ohos::napi]
            pub fn on_bridge_lifecycle(kind: String) -> #framework::__private::napi_ohos::Result<()> {
                let event =
                    #framework::__private::openharmony_ability::PluginLifecycleEvent::from_arkts(&kind)?;
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
