use super::*;

pub(super) fn register_lifecycle_callback(
    webview: &Webview,
    callback: &WebViewLifecycleCallback,
    lifecycle: WebViewLifecycle,
) -> Result<(), String> {
    let callback = callback.clone();
    let callback_dispatcher = arkit_runtime::internal::global_dispatcher();
    match lifecycle {
        WebViewLifecycle::ControllerAttach => webview.on_controller_attach({
            let callback = callback.clone();
            let callback_dispatcher = callback_dispatcher.clone();
            move || {
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    let mut callback = callback.borrow_mut();
                    (callback.as_mut())();
                });
            }
        }),
        WebViewLifecycle::PageBegin => webview.on_page_begin({
            let callback = callback.clone();
            let callback_dispatcher = callback_dispatcher.clone();
            move || {
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    let mut callback = callback.borrow_mut();
                    (callback.as_mut())();
                });
            }
        }),
        WebViewLifecycle::PageEnd => webview.on_page_end({
            let callback = callback.clone();
            let callback_dispatcher = callback_dispatcher.clone();
            move || {
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    let mut callback = callback.borrow_mut();
                    (callback.as_mut())();
                });
            }
        }),
        WebViewLifecycle::Destroy => webview.on_destroy({
            let callback = callback.clone();
            let callback_dispatcher = callback_dispatcher.clone();
            move || {
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    let mut callback = callback.borrow_mut();
                    (callback.as_mut())();
                });
            }
        }),
    }
    .map_err(|error| error.to_string())
}

#[cfg(feature = "webview")]
pub(super) fn register_webview_lifecycle_callbacks(
    controller: &WebViewController,
    webview: &Webview,
) -> Result<(), String> {
    for callback in controller.inner.controller_attach_callbacks.borrow().iter() {
        register_lifecycle_callback(webview, callback, WebViewLifecycle::ControllerAttach)?;
    }
    for callback in controller.inner.page_begin_callbacks.borrow().iter() {
        register_lifecycle_callback(webview, callback, WebViewLifecycle::PageBegin)?;
    }
    for callback in controller.inner.page_end_callbacks.borrow().iter() {
        register_lifecycle_callback(webview, callback, WebViewLifecycle::PageEnd)?;
    }
    for callback in controller.inner.destroy_callbacks.borrow().iter() {
        register_lifecycle_callback(webview, callback, WebViewLifecycle::Destroy)?;
    }
    Ok(())
}

#[cfg(feature = "webview")]
pub(super) fn register_internal_webview_callbacks(
    controller: &WebViewController,
    webview: &Webview,
) -> Result<(), String> {
    let attach_controller = controller.clone();
    webview
        .on_controller_attach(move || {
            attach_controller.inner.attached.set(true);
        })
        .map_err(|error| error.to_string())?;

    let state = controller.inner.clone();
    webview
        .on_destroy(move || {
            state.attached.set(false);
        })
        .map_err(|error| error.to_string())
}

#[cfg(feature = "webview")]
pub(super) fn build_initial_webview_style(
    spec: &WebViewSpec,
    _frame: Option<LayoutFrame>,
) -> Option<WebViewStyle> {
    let style = spec.style.clone().unwrap_or_default();
    let has_style = style.x.is_some()
        || style.y.is_some()
        || style.visible.is_some()
        || style.background_color.is_some();
    has_style.then_some(style)
}

#[cfg(feature = "webview")]
pub(super) fn webview_frame_is_valid(frame: LayoutFrame) -> bool {
    frame.width.is_finite() && frame.height.is_finite() && frame.width > 0.0 && frame.height > 0.0
}

#[cfg(feature = "webview")]
pub(super) fn sync_embedded_webview_node_bounds(
    controller: &WebViewController,
    frame: Option<LayoutFrame>,
) -> Result<(), String> {
    let Some(frame) = frame.filter(|frame| webview_frame_is_valid(*frame)) else {
        return Ok(());
    };
    let Some(mut node) = controller.inner.embedded_node.borrow().clone() else {
        return Ok(());
    };

    let runtime = RuntimeNode(&mut node);
    runtime
        .set_position(vec![0.0_f32, 0.0_f32])
        .map_err(|error| error.to_string())?;
    runtime
        .set_size(vec![frame.width, frame.height])
        .map_err(|error| error.to_string())?;
    runtime
        .set_layout_rect(vec![0.0_f32, 0.0_f32, frame.width, frame.height])
        .map_err(|error| error.to_string())
}

#[cfg(feature = "webview")]
pub(super) fn same_style_value(
    left: &Option<Either<f64, String>>,
    right: &Option<Either<f64, String>>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(Either::A(left)), Some(Either::A(right))) => left == right,
        (Some(Either::B(left)), Some(Either::B(right))) => left == right,
        _ => false,
    }
}

#[cfg(feature = "webview")]
pub(super) fn same_webview_style(
    left: Option<&WebViewStyle>,
    right: Option<&WebViewStyle>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            same_style_value(&left.x, &right.x)
                && same_style_value(&left.y, &right.y)
                && left.visible == right.visible
                && left.background_color == right.background_color
        }
        _ => false,
    }
}

#[cfg(feature = "webview")]
pub(super) fn current_applied_state(
    spec: &WebViewSpec,
    frame: Option<LayoutFrame>,
) -> WebViewAppliedState {
    WebViewAppliedState {
        url: spec.url.clone(),
        html: spec.html.clone(),
        headers: spec.headers.clone(),
        style: build_initial_webview_style(spec, frame),
    }
}

#[cfg(feature = "webview")]
pub(super) struct NativeWebViewMount {
    pub(super) webview: Webview,
    pub(super) node: ArkUINode,
}

#[cfg(feature = "webview")]
pub(super) fn create_native_webview(
    spec: &WebViewSpec,
    frame: Option<LayoutFrame>,
) -> Result<NativeWebViewMount, String> {
    let controller = spec
        .controller
        .as_ref()
        .ok_or_else(|| String::from("webview controller is missing"))?;
    let helper = unsafe { get_helper() };
    let helper_borrow = helper.borrow();
    let helper_ref = helper_borrow
        .as_ref()
        .ok_or_else(|| String::from("arkts helper is not available"))?;
    let env = get_main_thread_env();
    let env_borrow = env.borrow();
    let env_ref = env_borrow
        .as_ref()
        .ok_or_else(|| String::from("main thread env is not available"))?;
    let helper_object = helper_ref
        .get_value(env_ref)
        .map_err(|error| error.to_string())?;
    let create_webview = helper_object
        .get_named_property::<Function<'_, WebViewInitData<'_>, ObjectRef>>("createEmbeddedWebview")
        .map_err(|error| error.to_string())?;
    let callback_dispatcher = arkit_runtime::internal::global_dispatcher();

    let on_drag_and_drop = spec.on_drag_and_drop.as_ref().and_then(|handler| {
        let handler = handler.clone();
        let callback_dispatcher = callback_dispatcher.clone();
        env_ref
            .create_function_from_closure("arkit_on_drag_and_drop", move |ctx| {
                let event = ctx.try_get::<String>(0)?;
                let event = match event {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let handler = handler.clone();
                run_with_webview_dispatcher(callback_dispatcher.clone(), || handler(event));
                Ok(())
            })
            .ok()
    });

    let on_download_start = spec.on_download_start.as_ref().and_then(|handler| {
        let handler = handler.clone();
        let callback_dispatcher = callback_dispatcher.clone();
        env_ref
            .create_function_from_closure("arkit_on_download_start", move |ctx| {
                let origin_url = ctx.try_get::<String>(0)?;
                let temp_path = ctx.try_get::<String>(1)?;
                let origin_url = match origin_url {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let temp_path = match temp_path {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let mut path = PathBuf::from(temp_path);
                let handler = handler.clone();
                let allow = run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    handler(origin_url, &mut path)
                });
                Ok(DownloadStartResult {
                    allow,
                    temp_path: Some(path.to_string_lossy().to_string()),
                })
            })
            .ok()
    });

    let on_download_end = spec.on_download_end.as_ref().and_then(|handler| {
        let handler = handler.clone();
        let callback_dispatcher = callback_dispatcher.clone();
        env_ref
            .create_function_from_closure("arkit_on_download_end", move |ctx| {
                let origin_url = ctx.try_get::<String>(0)?;
                let temp_path = ctx.try_get::<String>(1)?;
                let success = ctx.try_get::<bool>(2)?;
                let origin_url = match origin_url {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let temp_path = match temp_path {
                    Either::A(value) => Some(PathBuf::from(value)),
                    Either::B(_undefined) => None,
                };
                let success = match success {
                    Either::A(value) => value,
                    Either::B(_undefined) => false,
                };
                let handler = handler.clone();
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    handler(origin_url, temp_path, success);
                });
                Ok(())
            })
            .ok()
    });

    let on_navigation_request = spec.on_navigation_request.as_ref().and_then(|handler| {
        let handler = handler.clone();
        let callback_dispatcher = callback_dispatcher.clone();
        env_ref
            .create_function_from_closure("arkit_on_navigation_request", move |ctx| {
                let url = ctx.try_get::<String>(0)?;
                let url = match url {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let handler = handler.clone();
                Ok(run_with_webview_dispatcher(
                    callback_dispatcher.clone(),
                    || handler(url),
                ))
            })
            .ok()
    });

    let on_title_change = spec.on_title_change.as_ref().and_then(|handler| {
        let handler = handler.clone();
        let callback_dispatcher = callback_dispatcher.clone();
        env_ref
            .create_function_from_closure("arkit_on_title_change", move |ctx| {
                let title = ctx.try_get::<String>(0)?;
                let title = match title {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let handler = handler.clone();
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    handler(title);
                });
                Ok(())
            })
            .ok()
    });

    let embedded_webview = create_webview
        .call(WebViewInitData {
            url: spec.url.clone(),
            id: Some(controller.id().to_string()),
            style: build_initial_webview_style(spec, frame),
            javascript_enabled: spec.javascript_enabled,
            devtools: spec.devtools,
            user_agent: spec.user_agent.clone(),
            autoplay: spec.autoplay,
            initialization_scripts: spec.initialization_scripts.clone(),
            headers: spec.headers.clone(),
            html: spec.html.clone(),
            transparent: spec.transparent,
            on_drag_and_drop,
            on_download_start,
            on_download_end,
            on_navigation_request,
            on_title_change,
        })
        .map_err(|error| error.to_string())?;

    let embedded_value = embedded_webview
        .get_value(env_ref)
        .map_err(|error| error.to_string())?;
    let controller_object = embedded_value
        .get::<Object>("controller")
        .map_err(|error| error.to_string())?
        .ok_or_else(|| String::from("embedded webview controller is missing"))?;
    let node_raw = embedded_value
        .get::<ArkUINodeRaw>("content")
        .map_err(|error| error.to_string())?
        .ok_or_else(|| String::from("embedded webview content is missing"))?;
    let controller_ref = controller_object
        .create_ref::<true>()
        .map_err(|error| error.to_string())?;
    let node = ArkUINode::from_raw_handle(node_raw.raw)
        .ok_or_else(|| String::from("embedded webview content handle is null"))?;
    let webview = Webview::new(controller.id().to_string(), controller_ref)
        .map_err(|error| error.to_string())?;

    Ok(NativeWebViewMount { webview, node })
}

#[cfg(feature = "webview")]
pub(super) fn attach_embedded_webview_node(
    host: &mut ArkUINode,
    controller: &WebViewController,
) -> Result<(), String> {
    let Some(node) = controller.inner.embedded_node.borrow().clone() else {
        return Ok(());
    };
    let raw_handle = node.raw_handle() as usize;
    if host
        .children()
        .iter()
        .any(|child| child.borrow().raw_handle() as usize == raw_handle)
    {
        return Ok(());
    }

    let mut runtime = RuntimeNode(host);
    runtime
        .add_existing_child(node)
        .map_err(|error| error.to_string())
}

#[cfg(feature = "webview")]
pub(super) fn detach_embedded_webview_node(host: &mut ArkUINode, controller: &WebViewController) {
    let Some(raw_handle) = controller
        .inner
        .embedded_node
        .borrow()
        .as_ref()
        .map(|node| node.raw_handle() as usize)
    else {
        return;
    };

    if let Err(error) = remove_child_by_raw(host, raw_handle) {
        ohos_hilog_binding::error(format!(
            "webview error: failed to detach embedded webview '{}': {error}",
            controller.id()
        ));
    }
}
