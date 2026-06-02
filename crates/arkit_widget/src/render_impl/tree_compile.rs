use super::*;

/// WebView 节点挂载前的额外初始化（仅在 webview feature 下有效果）。
#[cfg(feature = "webview")]
pub(super) fn prepare_node<Message, AppTheme>(
    mut node: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    if node.kind == NodeKind::WebViewHost {
        enrich_webview_host(&mut node);
    }
    node
}

#[cfg(not(feature = "webview"))]
pub(super) fn prepare_node<Message, AppTheme>(
    node: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    node
}

/// 将 Scroll 节点的状态绑定到 event_handlers 和 attach_effects 中，使得
/// 滚动偏移量在 widget tree 重建后能自动恢复。
pub(super) fn bind_node_state(
    kind: NodeKind,
    event_handlers: &mut Vec<EventHandlerSpec>,
    attach_effects: &mut Vec<AttachEffect>,
    state_bound: &mut bool,
    tree: &mut advanced::widget::Tree,
) {
    if *state_bound || !matches!(kind, NodeKind::Scroll) {
        return;
    }

    let Some(scroll_state) = tree
        .state()
        .downcast_mut::<Rc<RefCell<ScrollState>>>()
        .cloned()
    else {
        return;
    };

    event_handlers.push(EventHandlerSpec {
        event_type: NodeEventType::ScrollEventOnScroll,
        callback: Rc::new({
            let scroll_state = scroll_state.clone();
            move |event| {
                let event_offset = ScrollOffset {
                    x: event.f32_value(0).unwrap_or_default(),
                    y: event.f32_value(1).unwrap_or_default(),
                };
                let offset = scroll_state
                    .borrow()
                    .node
                    .clone()
                    .and_then(|mut node| read_scroll_offset(&mut node))
                    .unwrap_or(event_offset);
                let mut state = scroll_state.borrow_mut();
                state.offset = offset;
                if let Some(viewport) = state.viewport.as_mut() {
                    viewport.offset = offset;
                }
            }
        }),
    });

    attach_effects.push(Box::new(move |node| {
        let alive = Rc::new(Cell::new(true));
        let scroll_node = node.clone();
        let scroll_state = scroll_state.clone();
        scroll_state.borrow_mut().node = Some(scroll_node.clone());

        let restore_state = scroll_state.clone();
        let restore = Rc::new(move || {
            let offset = restore_state.borrow().offset;
            if offset == ScrollOffset::default() {
                return;
            }
            let mut scroll_node = scroll_node.clone();
            if let Err(error) = apply_scroll_offset(&mut scroll_node, offset) {
                ohos_hilog_binding::error(format!(
                    "renderer error: failed to restore scroll offset: {error}"
                ));
            }
        });

        let frame_alive = alive.clone();
        let frame_restore = restore.clone();
        node.post_frame_callback(move |_timestamp, _frame| {
            if !frame_alive.get() {
                return;
            }
            frame_restore();
        })?;

        let idle_alive = alive.clone();
        let idle_restore = restore;
        node.post_idle_callback(move |_time_left, _frame| {
            if !idle_alive.get() {
                return;
            }
            idle_restore();
        })?;

        Ok(Some(Box::new(move || {
            alive.set(false);
            scroll_state.borrow_mut().node = None;
        }) as Cleanup))
    }));

    *state_bound = true;
}
