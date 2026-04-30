use super::*;

pub(super) fn sync_element_tree<Message, AppTheme>(
    element: &Element<Message, AppTheme>,
    tree: &mut advanced::widget::Tree,
    state_cache: &mut StateCache,
) where
    Message: 'static,
    AppTheme: 'static,
{
    let widget = element.as_widget();
    let next_tag = widget.tag();
    let next_persistent_key = widget.persistent_key();
    if tree.tag() != next_tag || tree.persistent_key() != next_persistent_key {
        let next_tree = state_cache
            .take(next_tag, next_persistent_key)
            .unwrap_or_else(|| arkit_core::advanced::tree_of(element));
        let previous_tree = std::mem::replace(tree, next_tree);
        state_cache.store(previous_tree);
    }

    widget.diff(tree);
    tree.set_persistent_key(next_persistent_key.map(str::to_string));
}

pub(super) fn sync_child_trees<Message, AppTheme>(
    children: &[Element<Message, AppTheme>],
    tree: &mut advanced::widget::Tree,
    state_cache: &mut StateCache,
) where
    Message: 'static,
    AppTheme: 'static,
{
    let mut next_trees = Vec::with_capacity(children.len());
    let mut existing = std::mem::take(tree.children_mut());

    for child in children {
        let widget = child.as_widget();
        let next_tag = widget.tag();
        let next_persistent_key = widget.persistent_key();
        let mut child_tree = if let Some(persistent_key) = next_persistent_key {
            if let Some(index) = existing.iter().position(|tree| {
                tree.tag() == next_tag && tree.persistent_key() == Some(persistent_key)
            }) {
                existing.remove(index)
            } else {
                state_cache
                    .take(next_tag, Some(persistent_key))
                    .unwrap_or_else(|| arkit_core::advanced::tree_of(child))
            }
        } else if existing.is_empty() {
            arkit_core::advanced::tree_of(child)
        } else {
            existing.remove(0)
        };
        sync_element_tree(child, &mut child_tree, state_cache);
        next_trees.push(child_tree);
    }

    for child_tree in existing {
        state_cache.store(child_tree);
    }

    tree.replace_children(next_trees);
}

fn sync_composite_child_tree<'a, Message, AppTheme>(
    tree: &'a mut advanced::widget::Tree,
    index: usize,
    element: &Element<Message, AppTheme>,
    state_cache: &mut StateCache,
) -> &'a mut advanced::widget::Tree
where
    Message: 'static,
    AppTheme: 'static,
{
    while tree.children().len() <= index {
        tree.children_mut()
            .push(arkit_core::advanced::tree_of(element));
    }

    let child_tree = tree
        .child_mut(index)
        .expect("composite child tree was just initialized");
    sync_element_tree(element, child_tree, state_cache);
    child_tree
}

fn prune_composite_children(
    tree: &mut advanced::widget::Tree,
    keep: usize,
    state_cache: &mut StateCache,
) {
    if tree.children().len() <= keep {
        return;
    }

    for child_tree in tree.children_mut().split_off(keep) {
        state_cache.store(child_tree);
    }
}

pub(super) struct CompiledElement<Message, AppTheme = arkit_core::Theme> {
    pub(super) body: Element<Message, AppTheme>,
    pub(super) overlays: Vec<Element<Message, AppTheme>>,
}

pub(super) fn retained_element<Message, AppTheme>() -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    Node::new(NodeKind::Retained).into()
}

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

pub(super) fn compile_node<Message, AppTheme>(
    node: Node<Message, AppTheme>,
    tree: &mut advanced::widget::Tree,
    state_cache: &mut StateCache,
    renderer: &Renderer,
    bind_state: bool,
) -> CompiledElement<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let Node {
        kind,
        key,
        persistent_key,
        init_attrs,
        patch_attrs,
        mut event_handlers,
        long_press_handler,
        mount_effects,
        mut attach_effects,
        patch_effects,
        exit_effect,
        mut state_bound,
        virtual_adapter,
        #[cfg(feature = "webview")]
            webview: _,
        children,
    } = prepare_node(node);

    if bind_state {
        bind_node_state(
            kind,
            &mut event_handlers,
            &mut attach_effects,
            &mut state_bound,
            tree,
        );
    }
    sync_child_trees(&children, tree, state_cache);

    let mut compiled_children = Vec::with_capacity(children.len());
    let mut overlays = Vec::new();

    for (child, child_tree) in children.into_iter().zip(tree.children_mut().iter_mut()) {
        let compiled = compile_element(child, child_tree, state_cache, renderer, bind_state);
        compiled_children.push(compiled.body);
        overlays.extend(compiled.overlays);
    }

    CompiledElement {
        body: Node {
            kind,
            key,
            persistent_key,
            init_attrs,
            patch_attrs,
            event_handlers,
            long_press_handler,
            mount_effects,
            attach_effects,
            patch_effects,
            exit_effect,
            state_bound,
            virtual_adapter,
            #[cfg(feature = "webview")]
            webview: None,
            children: compiled_children,
        }
        .into(),
        overlays,
    }
}

pub(super) fn compile_element<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    tree: &mut advanced::widget::Tree,
    state_cache: &mut StateCache,
    renderer: &Renderer,
    bind_state: bool,
) -> CompiledElement<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    sync_element_tree(&element, tree, state_cache);

    let widget = element.into_widget();
    if widget.as_any().is::<Node<Message, AppTheme>>() {
        let any = widget.into_any();
        let node = any
            .downcast::<Node<Message, AppTheme>>()
            .unwrap_or_else(|_| {
                panic!(
                    "renderer node downcast failed for {}",
                    type_name::<Node<Message, AppTheme>>()
                )
            });
        return compile_node(*node, tree, state_cache, renderer, bind_state);
    }

    if component::is_component_widget::<Message, AppTheme>(widget.as_any()) {
        let any = widget.into_any();
        let node = component::component_into_node::<Message, AppTheme>(any).unwrap_or_else(|| {
            panic!(
                "renderer component downcast failed for {}",
                type_name::<Component<Message, AppTheme>>()
            )
        });
        return compile_node(node, tree, state_cache, renderer, bind_state);
    }

    let body = widget
        .cached_body(tree, renderer)
        .unwrap_or_else(|| panic!("composite widget did not provide a body element"));
    let body = match body {
        advanced::Body::Rebuild(body) => body,
        advanced::Body::Retain { overlays } => {
            return CompiledElement {
                body: retained_element(),
                overlays: (0..overlays).map(|_| retained_element()).collect(),
            };
        }
    };
    let compiled_body = {
        let body_tree = sync_composite_child_tree(tree, 0, &body, state_cache);
        compile_element(body, body_tree, state_cache, renderer, bind_state)
    };

    let overlay = widget.overlay(tree, renderer);
    let mut overlays = compiled_body.overlays;
    if let Some(overlay) = overlay {
        let compiled_overlay = {
            let overlay_tree = sync_composite_child_tree(tree, 1, &overlay, state_cache);
            compile_element(overlay, overlay_tree, state_cache, renderer, bind_state)
        };
        prune_composite_children(tree, 2, state_cache);
        overlays.push(compiled_overlay.body);
        overlays.extend(compiled_overlay.overlays);
    } else {
        prune_composite_children(tree, 1, state_cache);
    }
    widget.cache_overlay_count(tree, overlays.len());

    CompiledElement {
        body: compiled_body.body,
        overlays,
    }
}

pub(super) fn compose_compiled_overlays<Message, AppTheme>(
    compiled: CompiledElement<Message, AppTheme>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let overlay_hit_test = if compiled.overlays.is_empty() {
        HitTestBehavior::Transparent
    } else {
        HitTestBehavior::Default
    };
    let children = vec![
        compiled.body,
        stack_component::<Message, AppTheme>()
            .percent_width(1.0)
            .percent_height(1.0)
            .attr(ArkUINodeAttributeType::Clip, false)
            .hit_test_behavior(overlay_hit_test)
            .attr(
                ArkUINodeAttributeType::Alignment,
                i32::from(Alignment::TopStart),
            )
            .attr(ArkUINodeAttributeType::ZIndex, 10_000_i32)
            .children(compiled.overlays)
            .into(),
    ];

    stack_component::<Message, AppTheme>()
        .percent_width(1.0)
        .percent_height(1.0)
        .attr(ArkUINodeAttributeType::Clip, false)
        .attr(
            ArkUINodeAttributeType::Alignment,
            i32::from(Alignment::TopStart),
        )
        .children(children)
        .into()
}

pub(super) fn into_node<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> Node<Message, AppTheme>
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
        false,
    );
    let widget = compiled.body.into_widget();
    let any = widget.into_any();
    *any.downcast::<Node<Message, AppTheme>>()
        .unwrap_or_else(|_| {
            panic!(
                "arkit renderer only supports renderer::Node widget bodies in this build; got {}",
                type_name::<Node<Message, AppTheme>>()
            )
        })
}
