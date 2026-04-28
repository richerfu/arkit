use super::*;

pub(super) fn desired_attrs(
    init_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
) -> Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)> {
    let mut attrs = Vec::new();
    for (attr, value) in init_attrs.into_iter().chain(patch_attrs) {
        if let Some(index) = attrs
            .iter()
            .position(|(current_attr, _)| *current_attr == attr)
        {
            attrs.remove(index);
        }
        attrs.push((attr, value));
    }
    attrs
}

pub(super) fn attr_types(
    attrs: &[(ArkUINodeAttributeType, ArkUINodeAttributeItem)],
) -> Vec<ArkUINodeAttributeType> {
    attrs.iter().map(|(attr, _)| *attr).collect()
}

pub(super) fn desired_attr_types(
    init_attrs: &[(ArkUINodeAttributeType, ArkUINodeAttributeItem)],
    patch_attrs: &[(ArkUINodeAttributeType, ArkUINodeAttributeItem)],
) -> Vec<ArkUINodeAttributeType> {
    let mut attrs = Vec::new();
    for (attr, _) in init_attrs.iter().chain(patch_attrs.iter()) {
        if !attrs.contains(attr) {
            attrs.push(*attr);
        }
    }
    attrs
}

pub(super) fn reset_stale_attrs(
    node: &mut ArkUINode,
    previous: &[ArkUINodeAttributeType],
    next: &[ArkUINodeAttributeType],
) {
    let runtime = RuntimeNode(node);
    for attr in previous {
        if !next.contains(attr) {
            let _ = runtime.reset_attribute(*attr);
        }
    }
}

pub(super) fn apply_event_handlers(node: &mut ArkUINode, handlers: &[EventHandlerSpec]) {
    let mut runtime = RuntimeNode(node);
    for (event_type, callbacks) in grouped_event_handlers(handlers) {
        runtime.on_event(event_type, move |event| {
            for callback in &callbacks {
                callback(event);
            }
        });
    }
}

pub(super) fn grouped_event_handlers(
    handlers: &[EventHandlerSpec],
) -> Vec<(NodeEventType, Vec<EventCallback>)> {
    let mut groups = Vec::<(NodeEventType, Vec<EventCallback>)>::new();
    for handler in handlers {
        if let Some((_, callbacks)) = groups
            .iter_mut()
            .find(|(event_type, _)| *event_type == handler.event_type)
        {
            callbacks.push(handler.callback.clone());
        } else {
            groups.push((handler.event_type, vec![handler.callback.clone()]));
        }
    }
    groups
}

pub(super) fn event_types(handlers: &[EventHandlerSpec]) -> Vec<NodeEventType> {
    let mut events = Vec::new();
    for handler in handlers {
        if !events.contains(&handler.event_type) {
            events.push(handler.event_type);
        }
    }
    events
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct NodeSignature {
    events: Vec<NodeEventType>,
    mount_effect_count: usize,
    attach_effect_count: usize,
    patch_effect_count: usize,
    has_long_press: bool,
    virtual_adapter_kind: Option<VirtualContainerKind>,
}

pub(super) fn node_signature<Message, AppTheme>(node: &Node<Message, AppTheme>) -> NodeSignature {
    NodeSignature {
        events: event_types(&node.event_handlers),
        mount_effect_count: node.mount_effects.len(),
        attach_effect_count: node.attach_effects.len(),
        patch_effect_count: node.patch_effects.len(),
        has_long_press: node.long_press_handler.is_some(),
        virtual_adapter_kind: node.virtual_adapter.as_ref().map(|spec| spec.kind),
    }
}

pub(super) fn mounted_signature(mounted: &MountedRenderNode) -> NodeSignature {
    NodeSignature {
        events: mounted.events.clone(),
        mount_effect_count: mounted.mount_effect_count,
        attach_effect_count: mounted.attach_effect_count,
        patch_effect_count: mounted.patch_effect_count,
        has_long_press: mounted.has_long_press,
        virtual_adapter_kind: mounted.virtual_adapter_kind,
    }
}

pub(super) fn clear_removed_events(
    node: &mut ArkUINode,
    previous: &[NodeEventType],
    next: &[NodeEventType],
) {
    let mut runtime = RuntimeNode(node);
    for event_type in previous {
        if !next.contains(event_type) {
            runtime.on_event(*event_type, |_| {});
        }
    }
}

pub(super) fn mount_long_press(
    node: &mut ArkUINode,
    handler: &LongPressHandlerSpec,
) -> ArkUIResult<(Option<Cleanup>, Rc<RefCell<Rc<dyn Fn()>>>)> {
    let gesture = Gesture::create_long_gesture(1, true, DEFAULT_LONG_PRESS_DURATION_MS)?;
    let callback_state = Rc::new(RefCell::new(handler.callback.clone()));
    let callback_data = Box::into_raw(Box::new(LongPressCallbackContext {
        callback: callback_state.clone(),
    }));

    if let Err(error) = gesture.on_gesture_with_data(
        GestureEventAction::Accept | GestureEventAction::Update | GestureEventAction::End,
        callback_data.cast(),
        long_press_gesture_callback,
    ) {
        unsafe {
            drop(Box::from_raw(callback_data));
        }
        let _ = gesture.dispose();
        return Err(error);
    }

    let runtime = RuntimeNode(node);
    if let Err(error) = runtime.add_gesture_ref(&gesture, None, None) {
        unsafe {
            drop(Box::from_raw(callback_data));
        }
        let _ = gesture.dispose();
        return Err(error);
    }

    let mut cleanup_node = node.clone();
    let cleanup = Box::new(move || {
        let runtime = RuntimeNode(&mut cleanup_node);
        let _ = runtime.remove_gesture(&gesture);
        let _ = gesture.dispose();
        unsafe {
            drop(Box::from_raw(callback_data));
        }
    }) as Cleanup;
    Ok((Some(cleanup), callback_state))
}

pub(super) fn attach_child(parent: &mut ArkUINode, child: ArkUINode) -> ArkUIResult<()> {
    let mut runtime = RuntimeNode(parent);
    runtime.add_child(child)
}

pub(super) fn insert_child(
    parent: &mut ArkUINode,
    child: ArkUINode,
    index: usize,
) -> ArkUIResult<()> {
    let mut runtime = RuntimeNode(parent);
    runtime.insert_child(child, index)
}

pub(super) fn attach_child_at(
    parent: &mut ArkUINode,
    child: ArkUINode,
    index: usize,
) -> ArkUIResult<()> {
    if index == parent.children().len() {
        attach_child(parent, child)
    } else {
        insert_child(parent, child, index)
    }
}

pub(super) fn remove_child_by_raw(
    parent: &mut ArkUINode,
    raw_handle: usize,
) -> ArkUIResult<Option<Rc<RefCell<ArkUINode>>>> {
    let index = parent
        .children()
        .iter()
        .position(|child| child.borrow().raw_handle() as usize == raw_handle);

    match index {
        Some(index) => parent.remove_child(index),
        None => Ok(None),
    }
}

pub(super) fn complete_exiting_child(
    mut parent: ArkUINode,
    raw_handle: usize,
    mounted: Rc<RefCell<Option<MountedRenderNode>>>,
    pending_exits: Rc<RefCell<Vec<PendingExit>>>,
    alive: Rc<Cell<bool>>,
) {
    if !alive.replace(false) {
        return;
    }

    pending_exits
        .borrow_mut()
        .retain(|exit| exit.raw_handle != raw_handle);

    match remove_child_by_raw(&mut parent, raw_handle) {
        Ok(Some(removed)) => {
            let mut removed = removed.borrow().clone();
            let _ = removed.dispose();
        }
        Ok(None) => {}
        Err(error) => {
            ohos_hilog_binding::error(format!(
                "renderer error: failed to remove exiting child: {error}"
            ));
        }
    }

    if let Some(mounted) = mounted.borrow_mut().take() {
        mounted.cleanup_recursive();
    }
}

pub(super) fn remove_or_exit_child(
    parent: &mut ArkUINode,
    index: usize,
    mut mounted: MountedRenderNode,
    pending_exits: Rc<RefCell<Vec<PendingExit>>>,
) -> ArkUIResult<()> {
    let Some(exit_effect) = mounted.exit_effect.take() else {
        let removed = parent.remove_child(index)?;
        if let Some(removed) = removed {
            let mut removed = removed.borrow().clone();
            let _ = removed.dispose();
        }
        mounted.cleanup_recursive();
        return Ok(());
    };

    let Some(child_handle) = parent.children().get(index).cloned() else {
        mounted.cleanup_recursive();
        return Ok(());
    };

    let raw_handle = child_handle.borrow().raw_handle() as usize;
    let alive = Rc::new(Cell::new(true));
    let mounted_slot = Rc::new(RefCell::new(Some(mounted)));
    let effect_cleanup = Rc::new(RefCell::new(None::<Cleanup>));

    pending_exits.borrow_mut().push(PendingExit {
        raw_handle,
        alive: alive.clone(),
        mounted: mounted_slot.clone(),
        effect_cleanup: effect_cleanup.clone(),
    });

    let finish_parent = parent.clone();
    let finish_mounted = mounted_slot.clone();
    let finish_pending = pending_exits.clone();
    let finish_alive = alive.clone();
    let finish = Box::new(move || {
        complete_exiting_child(
            finish_parent,
            raw_handle,
            finish_mounted,
            finish_pending,
            finish_alive,
        );
    }) as Cleanup;

    let mut child_node = child_handle.borrow_mut();
    if let Err(error) = child_node.set_attribute(
        ArkUINodeAttributeType::HitTestBehavior,
        i32::from(HitTestBehavior::None).into(),
    ) {
        ohos_hilog_binding::error(format!(
            "renderer error: failed to disable exiting child hit test: {error}"
        ));
    }

    match exit_effect(&mut child_node, finish) {
        Ok(cleanup) => {
            effect_cleanup.replace(cleanup);
        }
        Err(error) => {
            ohos_hilog_binding::error(format!("renderer error: exit effect failed: {error}"));
            drop(child_node);
            complete_exiting_child(
                parent.clone(),
                raw_handle,
                mounted_slot,
                pending_exits,
                alive,
            );
        }
    }

    Ok(())
}

pub(super) fn realize_attached_node(
    node: &mut ArkUINode,
    mounted: &mut MountedRenderNode,
) -> ArkUIResult<()> {
    if !mounted.pending_patch_attrs.is_empty() {
        apply_attr_list(node, std::mem::take(&mut mounted.pending_patch_attrs));
    }

    for effect in std::mem::take(&mut mounted.pending_attach_effects) {
        match effect(node)? {
            Some(cleanup) => mounted.cleanups.push(cleanup),
            None => {}
        }
    }

    for effect in std::mem::take(&mut mounted.pending_patch_effects) {
        effect(node)?;
    }

    for (child_handle, child_mounted) in node.children().iter().zip(mounted.children.iter_mut()) {
        let mut child_node = child_handle.borrow_mut();
        realize_attached_node(&mut child_node, child_mounted)?;
    }

    Ok(())
}

pub(super) fn set_node_object_attribute(
    node: &mut ArkUINode,
    attr: ArkUINodeAttributeType,
    value: &ArkUINode,
) -> ArkUIResult<()> {
    RuntimeNode(node).set_attribute(
        attr,
        ArkUINodeAttributeItem::Object(value.raw_handle().cast::<c_void>()),
    )
}
