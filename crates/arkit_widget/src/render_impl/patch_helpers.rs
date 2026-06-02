use super::*;

// mount_long_press、attach_child 等函数被 mounted.rs 内部通过 use super::* 访问，
// 此文件继续作为这些底层辅助的归属。

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

/// node_specialized.rs 中用于设置 slot 节点等对象类型属性。
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
