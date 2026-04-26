use super::*;

pub(super) fn mount_or_show_webview(
    host: &mut ArkUINode,
    controller: &WebViewController,
    spec: &WebViewSpec,
    frame: Option<LayoutFrame>,
) -> Result<(), String> {
    let created = if controller.inner.webview.borrow().is_none() {
        controller.inner.attached.set(false);
        let mount = match create_native_webview(spec, frame) {
            Ok(mount) => mount,
            Err(error) => {
                controller.inner.active_binding.set(false);
                return Err(error);
            }
        };
        let mut runtime = RuntimeNode(host);
        if let Err(error) = runtime.add_existing_child(mount.node.clone()) {
            let _ = mount.webview.dispose();
            return Err(error.to_string());
        }
        controller
            .inner
            .embedded_node
            .replace(Some(mount.node.clone()));
        let webview = mount.webview;
        controller.inner.webview.replace(Some(webview.clone()));
        register_internal_webview_callbacks(controller, &webview)?;
        register_webview_lifecycle_callbacks(controller, &webview)?;
        for callback in controller.inner.ready_callbacks.borrow().iter() {
            callback(&webview);
        }
        controller
            .inner
            .applied
            .replace(Some(current_applied_state(spec, frame)));
        true
    } else {
        false
    };

    if controller.inner.active_binding.replace(true) && created {
        ohos_hilog_binding::error(format!(
            "webview error: controller '{}' is already bound to an active host",
            controller.id()
        ));
    }

    attach_embedded_webview_node(host, controller)?;
    sync_embedded_webview_node_bounds(controller, frame)?;

    if !created {
        sync_webview_config(controller, spec, frame)?;
    }
    Ok(())
}

#[cfg(feature = "webview")]
pub(super) fn ensure_webview_mounted(
    host: &mut ArkUINode,
    controller: &WebViewController,
    spec: &WebViewSpec,
    frame: Option<LayoutFrame>,
) -> Result<(), String> {
    if controller.inner.webview.borrow().is_none() {
        mount_or_show_webview(host, controller, spec, frame)
    } else {
        attach_embedded_webview_node(host, controller)?;
        sync_embedded_webview_node_bounds(controller, frame)?;
        sync_webview_config(controller, spec, frame)
    }
}

#[cfg(feature = "webview")]
pub(super) fn sync_webview_config(
    controller: &WebViewController,
    spec: &WebViewSpec,
    frame: Option<LayoutFrame>,
) -> Result<(), String> {
    let current = current_applied_state(spec, frame);
    let previous = controller.inner.applied.borrow().clone();
    controller.inner.applied.replace(Some(current.clone()));

    let Some(webview) = controller.inner.webview.borrow().clone() else {
        return Ok(());
    };

    if force_webview_style_sync(previous.as_ref(), &current) {
        let previous_style = previous.as_ref().and_then(|state| state.style.as_ref());
        let current_style = current.style.as_ref();

        let previous_visible = previous_style.and_then(|style| style.visible);
        let current_visible = current_style.and_then(|style| style.visible);
        if previous_visible != current_visible {
            if let Some(visible) = current_visible {
                webview
                    .set_visible(visible)
                    .map_err(|error| error.to_string())?;
            }
        }

        let previous_background =
            previous_style.and_then(|style| style.background_color.as_deref());
        let current_background = current_style.and_then(|style| style.background_color.as_deref());
        if previous_background != current_background {
            if let Some(background) = current_background {
                webview
                    .set_background_color(background)
                    .map_err(|error| error.to_string())?;
            }
        }
    }

    if !controller.inner.attached.get() {
        return Ok(());
    }

    let page_source_changed = match previous.as_ref() {
        Some(state) => state.url != current.url || state.headers != current.headers,
        None => current.url.is_some() || current.headers.is_some(),
    };
    let html_changed = match previous.as_ref() {
        Some(state) => state.html != current.html,
        None => current.html.is_some(),
    };

    if page_source_changed {
        if let Some(url) = current.url.as_deref() {
            if let Some(headers) = current.headers.clone() {
                let headers = headers
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            key.parse::<http::header::HeaderName>()
                                .expect("webview header name should be valid"),
                            value
                                .parse::<http::header::HeaderValue>()
                                .expect("webview header value should be valid"),
                        )
                    })
                    .collect();
                webview
                    .load_url_with_headers(url, headers)
                    .map_err(|error| error.to_string())?;
            } else {
                webview.load_url(url).map_err(|error| error.to_string())?;
            }
        } else if let Some(html) = current.html.as_deref() {
            webview.load_html(html).map_err(|error| error.to_string())?;
        }
    } else if html_changed {
        if let Some(html) = current.html.as_deref() {
            webview.load_html(html).map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

#[cfg(feature = "webview")]
pub(super) fn force_webview_style_sync(
    previous: Option<&WebViewAppliedState>,
    current: &WebViewAppliedState,
) -> bool {
    !same_webview_style(
        previous.and_then(|state| state.style.as_ref()),
        current.style.as_ref(),
    )
}

#[cfg(feature = "webview")]
pub(super) fn unmount_webview(controller: &WebViewController) {
    controller.inner.active_binding.set(false);
    controller.inner.attached.set(false);
    controller.inner.applied.replace(None);
    controller.inner.embedded_node.replace(None);
    if let Some(webview) = controller.inner.webview.borrow_mut().take() {
        let _ = webview.dispose();
    }
}

#[cfg(feature = "webview")]
pub(super) fn enrich_webview_host<Message, AppTheme>(node: &mut Node<Message, AppTheme>) {
    let Some(spec) = node.webview.clone() else {
        return;
    };
    let Some(controller) = spec.controller.clone() else {
        return;
    };
    let node_ref = Rc::new(RefCell::new(None::<ArkUINode>));

    node.mount_effects.push(Box::new({
        let controller = controller.clone();
        let node_ref = node_ref.clone();
        move |_node| {
            Ok(Some(Box::new(move || {
                if let Some(mut host) = node_ref.borrow().as_ref().cloned() {
                    detach_embedded_webview_node(&mut host, &controller);
                }
                unmount_webview(&controller);
            }) as Cleanup))
        }
    }));

    node.attach_effects.push(Box::new({
        let controller = controller.clone();
        let spec = spec.clone();
        let node_ref = node_ref.clone();
        move |node| {
            let frame = read_layout_frame(node);
            if let Err(error) = ensure_webview_mounted(node, &controller, &spec, frame) {
                ohos_hilog_binding::error(format!(
                    "webview error: failed to mount webview '{}': {error}",
                    controller.id()
                ));
            }
            node_ref.replace(Some(node.clone()));
            Ok(None)
        }
    }));

    node.event_handlers.push(EventHandlerSpec {
        event_type: NodeEventType::EventOnAreaChange,
        callback: Rc::new({
            let controller = controller.clone();
            let spec = spec.clone();
            let node_ref = node_ref.clone();
            move |_| {
                let Some(node) = node_ref.borrow().as_ref().cloned() else {
                    return;
                };
                if controller.inner.webview.borrow().is_none() {
                    return;
                }
                if let Err(error) =
                    sync_webview_config(&controller, &spec, read_layout_frame(&node))
                {
                    ohos_hilog_binding::error(format!(
                        "webview error: failed to sync webview '{}' after area change: {error}",
                        controller.id()
                    ));
                }
            }
        }),
    });

    node.patch_effects.push(Box::new({
        let controller = controller.clone();
        let spec = spec.clone();
        let node_ref = node_ref.clone();
        move |node| {
            let frame = read_layout_frame(node);
            if let Err(error) = ensure_webview_mounted(node, &controller, &spec, frame) {
                ohos_hilog_binding::error(format!(
                    "webview error: failed to sync webview '{}': {error}",
                    controller.id()
                ));
            }
            node_ref.replace(Some(node.clone()));
            Ok(())
        }
    }));
}
