use super::*;

pub(super) fn mount_detached_element<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> ArkUIResult<(ArkUINode, MountedNode)>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let (tree, root) = compile_detached_element_root(element);
    let (node, render) = mount_node(root)?;
    Ok((node, MountedNode::new(tree, render)))
}

pub(super) fn compile_detached_element_root<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> (advanced::widget::Tree, Element<Message, AppTheme>)
where
    Message: 'static,
    AppTheme: 'static,
{
    let mut tree = arkit_core::advanced::tree_of(&element);
    let mut state_cache = StateCache::default();
    let compiled = compile_element(
        element,
        &mut tree,
        &mut state_cache,
        &Renderer::default(),
        true,
    );
    if !compiled.overlays.is_empty() {
        ohos_hilog_binding::warn(
            "renderer warning: detached virtual/list slot overlays are ignored".to_string(),
        );
    }
    (tree, compiled.body)
}

pub(super) fn mount_node<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> ArkUIResult<(ArkUINode, MountedRenderNode)>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let node = prepare_node(into_node(element));
    if node.kind == NodeKind::Retained {
        panic!("retained renderer node cannot be mounted");
    }

    let Node {
        kind,
        key,
        persistent_key: _,
        init_attrs,
        patch_attrs,
        event_handlers,
        long_press_handler,
        mount_effects,
        attach_effects,
        patch_effects,
        exit_effect,
        state_bound: _,
        virtual_adapter,
        #[cfg(feature = "webview")]
            webview: _,
        children,
    } = node;

    let mut node = create_node(kind)?;
    let init_attr_keys = attr_types(&init_attrs);
    let pending_patch_attrs = desired_attrs(Vec::new(), patch_attrs);
    let final_attr_keys = desired_attr_types(&init_attrs, &pending_patch_attrs);
    let mount_effect_count = mount_effects.len();
    let attach_effect_count = attach_effects.len();
    let patch_effect_count = patch_effects.len();
    let has_long_press = long_press_handler.is_some();
    apply_attr_list(&mut node, init_attrs);

    let mut cleanups = Vec::new();
    for effect in mount_effects {
        match effect(&mut node) {
            Ok(Some(cleanup)) => cleanups.push(cleanup),
            Ok(None) => {}
            Err(error) => {
                run_cleanups(cleanups);
                let _ = node.dispose();
                return Err(error);
            }
        }
    }

    apply_event_handlers(&mut node, &event_handlers);
    let events = event_types(&event_handlers);
    let (long_press_cleanup, long_press_callback) = match long_press_handler.as_ref() {
        Some(handler) => {
            let (cleanup, callback) = mount_long_press(&mut node, handler)?;
            (cleanup, Some(callback))
        }
        None => (None, None),
    };
    let virtual_adapter_kind = virtual_adapter.as_ref().map(|spec| spec.kind);
    let virtual_adapter = mount_virtual_adapter(&mut node, virtual_adapter)?;

    let mut mounted_children = Vec::with_capacity(children.len());
    for child in children {
        let (child_node, child_mounted) = mount_node(child)?;
        attach_child(&mut node, child_node)?;
        mounted_children.push(child_mounted);
    }

    Ok((
        node,
        MountedRenderNode::new(
            node_type_id(kind),
            key,
            if pending_patch_attrs.is_empty() && patch_effect_count == 0 {
                init_attr_keys
            } else {
                final_attr_keys
            },
            events,
            mount_effect_count,
            attach_effect_count,
            patch_effect_count,
            has_long_press,
            long_press_cleanup,
            long_press_callback,
            cleanups,
            exit_effect,
            pending_patch_attrs,
            attach_effects,
            patch_effects,
            virtual_adapter_kind,
            virtual_adapter,
            mounted_children,
        ),
    ))
}

pub(super) fn patch_node<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    node: &mut ArkUINode,
    mounted: &mut MountedRenderNode,
) -> ArkUIResult<()>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let next_node = prepare_node(into_node(element));
    if next_node.kind == NodeKind::Retained {
        return Ok(());
    }

    let Node {
        kind,
        key,
        persistent_key: _,
        init_attrs,
        patch_attrs,
        event_handlers,
        long_press_handler,
        mount_effects: _,
        attach_effects: _,
        patch_effects,
        exit_effect,
        state_bound: _,
        virtual_adapter,
        #[cfg(feature = "webview")]
            webview: _,
        children,
    } = next_node;

    mounted.tag = node_type_id(kind);
    mounted.key = key;
    mounted.exit_effect = exit_effect;
    let attrs = desired_attrs(init_attrs, patch_attrs);
    let next_attr_types = attr_types(&attrs);
    reset_stale_attrs(node, &mounted.attrs, &next_attr_types);
    apply_attr_list(node, attrs);
    mounted.attrs = next_attr_types;
    for effect in patch_effects {
        effect(node)?;
    }

    let next_events = event_types(&event_handlers);
    clear_removed_events(node, &mounted.events, &next_events);
    apply_event_handlers(node, &event_handlers);
    mounted.events = next_events;

    match (
        long_press_handler.as_ref(),
        mounted.long_press_callback.as_ref(),
    ) {
        (Some(handler), Some(callback)) => {
            callback.replace(handler.callback.clone());
        }
        (Some(handler), None) => {
            let (cleanup, callback) = mount_long_press(node, handler)?;
            mounted.long_press_cleanup = cleanup;
            mounted.long_press_callback = Some(callback);
        }
        (None, Some(_)) => {
            if let Some(cleanup) = mounted.long_press_cleanup.take() {
                cleanup();
            }
            mounted.long_press_callback = None;
        }
        (None, None) => {}
    }
    mounted.has_long_press = long_press_handler.is_some();

    patch_virtual_adapter(node, mounted, virtual_adapter)?;

    reconcile_children(node, mounted, children)
}

pub fn mount<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> ArkUIResult<(ArkUINode, MountedNode)>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let mut tree = arkit_core::advanced::tree_of(&element);
    let mut state_cache = StateCache::default();
    let compiled = compile_element(
        element,
        &mut tree,
        &mut state_cache,
        &Renderer::default(),
        true,
    );
    let root = compose_compiled_overlays(compiled);
    let (node, render) = mount_node(root)?;
    Ok((node, MountedNode::new(tree, render)))
}

pub fn realize_attached_mount(node: &mut ArkUINode, mounted: &mut MountedNode) -> ArkUIResult<()> {
    realize_attached_node(node, mounted.render_mut())
}

pub fn patch<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    node: &mut ArkUINode,
    mounted: &mut MountedNode,
) -> ArkUIResult<()>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let MountedNode {
        tree,
        retained_state,
        render,
    } = mounted;
    sync_element_tree(&element, tree, retained_state);
    let compiled = compile_element(element, tree, retained_state, &Renderer::default(), true);
    let root = compose_compiled_overlays(compiled);
    patch_node(root, node, render)
}
