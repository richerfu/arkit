use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

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

        mod __arkit_entry_mod {
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
                    if let Some(runtime) = runtime_state.as_ref() {
                        runtime.render()?;
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
