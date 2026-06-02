use super::*;

// ── Scroll 状态 ──────────────────────────────────────────────────────────────

#[derive(Default)]
pub(super) struct ScrollState {
    pub(super) offset: ScrollOffset,
    pub(super) viewport: Option<ScrollViewport>,
    pub(super) node: Option<ArkUINode>,
}

// ── 挂载树状态 ────────────────────────────────────────────────────────────────

/// 应用侧持有的完整挂载状态，包含 widget 树和渲染节点。
pub struct MountedNode {
    pub(super) tree: advanced::widget::Tree,
    pub(super) retained_state: StateCache,
    pub(super) render: MountedRenderNode,
}

impl MountedNode {
    pub(super) fn new(tree: advanced::widget::Tree, render: MountedRenderNode) -> Self {
        Self {
            tree,
            retained_state: StateCache::default(),
            render,
        }
    }

    pub(super) fn render_mut(&mut self) -> &mut MountedRenderNode {
        &mut self.render
    }

    pub fn cleanup_recursive(self) {
        self.render.cleanup_recursive();
    }
}

// ── 渲染节点挂载状态 ──────────────────────────────────────────────────────────

pub(super) struct MountedRenderNode {
    pub(super) tag: TypeId,
    pub(super) key: Option<String>,
    pub(super) attrs: Vec<ArkUINodeAttributeType>,
    pub(super) events: Vec<NodeEventType>,
    pub(super) mount_effect_count: usize,
    pub(super) attach_effect_count: usize,
    pub(super) patch_effect_count: usize,
    pub(super) has_long_press: bool,
    pub(super) long_press_cleanup: Option<Cleanup>,
    pub(super) long_press_callback: Option<Rc<RefCell<Rc<dyn Fn()>>>>,
    pub(super) cleanups: Vec<Cleanup>,
    pub(super) exit_effect: Option<ExitEffect>,
    pub(super) exiting_children: Rc<RefCell<Vec<PendingExit>>>,
    pub(super) pending_patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    pub(super) pending_attach_effects: Vec<AttachEffect>,
    pub(super) pending_patch_effects: Vec<PatchEffect>,
    pub(super) virtual_adapter_kind: Option<VirtualContainerKind>,
    pub(super) virtual_adapter: Option<Box<dyn MountedVirtualAdapter>>,
    pub(super) children: Vec<MountedRenderNode>,
}

// ── 子节点退出追踪 ────────────────────────────────────────────────────────────

pub(super) struct PendingExit {
    pub(super) raw_handle: usize,
    pub(super) alive: Rc<Cell<bool>>,
    pub(super) mounted: Rc<RefCell<Option<MountedRenderNode>>>,
    pub(super) effect_cleanup: Rc<RefCell<Option<Cleanup>>>,
}

// ── 签名（用于 reconcile 复用判断）───────────────────────────────────────────

#[derive(Clone, PartialEq, Eq)]
pub(super) struct NodeSignature {
    events: Vec<NodeEventType>,
    mount_effect_count: usize,
    attach_effect_count: usize,
    patch_effect_count: usize,
    has_long_press: bool,
    virtual_adapter_kind: Option<VirtualContainerKind>,
}

impl<Message, AppTheme> Node<Message, AppTheme> {
    pub(super) fn signature(&self) -> NodeSignature {
        NodeSignature {
            events: self.event_types(),
            mount_effect_count: self.mount_effects.len(),
            attach_effect_count: self.attach_effects.len(),
            patch_effect_count: self.patch_effects.len(),
            has_long_press: self.long_press_handler.is_some(),
            virtual_adapter_kind: self.virtual_adapter.as_ref().map(|spec| spec.kind),
        }
    }

    /// 提取所有事件处理器中不重复的事件类型列表。
    pub(super) fn event_types(&self) -> Vec<NodeEventType> {
        let mut events = Vec::new();
        for handler in &self.event_handlers {
            if !events.contains(&handler.event_type) {
                events.push(handler.event_type);
            }
        }
        events
    }

    /// 将 init_attrs 和 patch_attrs 合并为最终属性列表（后者覆盖前者）。
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
}

impl MountedRenderNode {
    pub(super) fn signature(&self) -> NodeSignature {
        NodeSignature {
            events: self.events.clone(),
            mount_effect_count: self.mount_effect_count,
            attach_effect_count: self.attach_effect_count,
            patch_effect_count: self.patch_effect_count,
            has_long_press: self.has_long_press,
            virtual_adapter_kind: self.virtual_adapter_kind,
        }
    }

    /// 将 pending 的属性、attach effect、patch effect 应用到已挂载的 ArkUI 节点上。
    pub(super) fn realize_attached(&mut self, node: &mut ArkUINode) -> ArkUIResult<()> {
        if !self.pending_patch_attrs.is_empty() {
            apply_attr_list(node, std::mem::take(&mut self.pending_patch_attrs));
        }

        for effect in std::mem::take(&mut self.pending_attach_effects) {
            match effect(node)? {
                Some(cleanup) => self.cleanups.push(cleanup),
                None => {}
            }
        }

        for effect in std::mem::take(&mut self.pending_patch_effects) {
            effect(node)?;
        }

        for (child_handle, child_mounted) in
            node.children().iter().zip(self.children.iter_mut())
        {
            let mut child_node = child_handle.borrow_mut();
            child_mounted.realize_attached(&mut child_node)?;
        }

        Ok(())
    }

    /// 将事件处理器注册到 ArkUI 节点上（按事件类型分组）。
    pub(super) fn apply_event_handlers(node: &mut ArkUINode, handlers: &[EventHandlerSpec]) {
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
        let mut runtime = RuntimeNode(node);
        for (event_type, callbacks) in groups {
            runtime.on_event(event_type, move |event| {
                for callback in &callbacks {
                    callback(event);
                }
            });
        }
    }

    /// 清除已移除的事件监听（注册为空回调）。
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

    /// 重置已不再需要的属性为默认值。
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

    // ── 挂载（构造） ──────────────────────────────────────────────────────────

    /// 将一个 Element 挂载为新的 ArkUI 节点，返回节点及对应的挂载状态。
    pub(super) fn mount<Message, AppTheme>(
        element: Element<Message, AppTheme>,
    ) -> ArkUIResult<(ArkUINode, Self)>
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

        let mut ark_node = create_node(kind)?;
        let init_attr_keys: Vec<_> = init_attrs.iter().map(|(a, _)| *a).collect();
        let pending_patch_attrs = Node::<Message, AppTheme>::desired_attrs(Vec::new(), patch_attrs);
        let final_attr_keys = {
            let mut keys = init_attr_keys.clone();
            for (attr, _) in &pending_patch_attrs {
                if !keys.contains(attr) {
                    keys.push(*attr);
                }
            }
            keys
        };
        let mount_effect_count = mount_effects.len();
        let attach_effect_count = attach_effects.len();
        let patch_effect_count = patch_effects.len();
        let has_long_press = long_press_handler.is_some();
        apply_attr_list(&mut ark_node, init_attrs);

        let mut cleanups = Vec::new();
        for effect in mount_effects {
            match effect(&mut ark_node) {
                Ok(Some(cleanup)) => cleanups.push(cleanup),
                Ok(None) => {}
                Err(error) => {
                    run_cleanups(cleanups);
                    let _ = ark_node.dispose();
                    return Err(error);
                }
            }
        }

        Self::apply_event_handlers(&mut ark_node, &event_handlers);
        let events: Vec<_> = {
            let mut v = Vec::new();
            for h in &event_handlers {
                if !v.contains(&h.event_type) {
                    v.push(h.event_type);
                }
            }
            v
        };
        let (long_press_cleanup, long_press_callback) = match long_press_handler.as_ref() {
            Some(handler) => {
                let (cleanup, callback) = mount_long_press(&mut ark_node, handler)?;
                (cleanup, Some(callback))
            }
            None => (None, None),
        };
        let virtual_adapter_kind = virtual_adapter.as_ref().map(|spec| spec.kind);
        let virtual_adapter = mount_virtual_adapter(&mut ark_node, virtual_adapter)?;

        let mut mounted_children = Vec::with_capacity(children.len());
        for child in children {
            let (child_node, child_mounted) = Self::mount(child)?;
            attach_child(&mut ark_node, child_node)?;
            mounted_children.push(child_mounted);
        }

        let attrs = if pending_patch_attrs.is_empty() && patch_effect_count == 0 {
            init_attr_keys
        } else {
            final_attr_keys
        };

        Ok((
            ark_node,
            Self {
                tag: node_type_id(kind),
                key,
                attrs,
                events,
                mount_effect_count,
                attach_effect_count,
                patch_effect_count,
                has_long_press,
                long_press_cleanup,
                long_press_callback,
                cleanups,
                exit_effect,
                exiting_children: Rc::new(RefCell::new(Vec::new())),
                pending_patch_attrs,
                pending_attach_effects: attach_effects,
                pending_patch_effects: patch_effects,
                virtual_adapter_kind,
                virtual_adapter,
                children: mounted_children,
            },
        ))
    }

    // ── 更新（patch） ─────────────────────────────────────────────────────────

    /// 用新的 Element 更新已挂载节点的状态。
    pub(super) fn patch<Message, AppTheme>(
        &mut self,
        element: Element<Message, AppTheme>,
        node: &mut ArkUINode,
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

        self.tag = node_type_id(kind);
        self.key = key;
        self.exit_effect = exit_effect;

        let attrs = Node::<Message, AppTheme>::desired_attrs(init_attrs, patch_attrs);
        let next_attr_types: Vec<_> = attrs.iter().map(|(a, _)| *a).collect();
        Self::reset_stale_attrs(node, &self.attrs, &next_attr_types);
        apply_attr_list(node, attrs);
        self.attrs = next_attr_types;
        for effect in patch_effects {
            effect(node)?;
        }

        let next_events: Vec<_> = {
            let mut v = Vec::new();
            for h in &event_handlers {
                if !v.contains(&h.event_type) {
                    v.push(h.event_type);
                }
            }
            v
        };
        Self::clear_removed_events(node, &self.events, &next_events);
        Self::apply_event_handlers(node, &event_handlers);
        self.events = next_events;

        match (
            long_press_handler.as_ref(),
            self.long_press_callback.as_ref(),
        ) {
            (Some(handler), Some(callback)) => {
                callback.replace(handler.callback.clone());
            }
            (Some(handler), None) => {
                let (cleanup, callback) = mount_long_press(node, handler)?;
                self.long_press_cleanup = cleanup;
                self.long_press_callback = Some(callback);
            }
            (None, Some(_)) => {
                if let Some(cleanup) = self.long_press_cleanup.take() {
                    cleanup();
                }
                self.long_press_callback = None;
            }
            (None, None) => {}
        }
        self.has_long_press = long_press_handler.is_some();

        patch_virtual_adapter(node, self, virtual_adapter)?;

        self.reconcile_children(node, children)
    }

    // ── 子节点协调 ─────────────────────────────────────────────────────────────

    /// 将当前子节点列表与新的 Element 列表协调，复用、更新或挂载新节点。
    pub(super) fn reconcile_children<Message, AppTheme>(
        &mut self,
        parent: &mut ArkUINode,
        next_children: Vec<Element<Message, AppTheme>>,
    ) -> ArkUIResult<()>
    where
        Message: Send + 'static,
        AppTheme: 'static,
    {
        let mut next_nodes: Vec<Node<Message, AppTheme>> = Vec::with_capacity(next_children.len());
        for child in next_children {
            next_nodes.push(into_node(child));
        }

        let next_len = next_nodes.len();
        let pending_exits = self.exiting_children.clone();
        let mounted_children = &mut self.children;

        for (index, child) in next_nodes.into_iter().enumerate() {
            if child.kind == NodeKind::Retained {
                if index >= mounted_children.len() {
                    panic!("retained renderer child has no mounted subtree at index {index}");
                }
                continue;
            }

            let next_key = child
                .key
                .clone()
                .map(|key| (node_type_id(child.kind), key));
            let current_key = mounted_children.get(index).and_then(|m| {
                m.key.clone().map(|key| (m.tag, key))
            });

            let can_reuse = mounted_children.get(index).is_some_and(|m| {
                let key_match = if next_key.is_none() && current_key.is_none() {
                    true
                } else {
                    next_key == current_key
                };
                key_match && node_type_id(child.kind) == m.tag && child.signature() == m.signature()
            });

            if can_reuse {
                let child_handle = parent.children()[index].clone();
                let mut child_node = child_handle.borrow_mut();
                mounted_children[index].patch(child.into(), &mut child_node)?;
                continue;
            }

            if index < mounted_children.len() {
                let mounted = mounted_children.remove(index);
                Self::remove_or_exit_child(parent, index, mounted, pending_exits.clone())?;
            }

            let (child_node, mut child_meta) = Self::mount(child.into())?;
            attach_child_at(parent, child_node, index)?;
            if let Some(child_handle) = parent.children().get(index) {
                let mut child_node = child_handle.borrow_mut();
                child_meta.realize_attached(&mut child_node)?;
            }
            mounted_children.insert(index, child_meta);
        }

        while mounted_children.len() > next_len {
            let index = mounted_children.len() - 1;
            let mounted = mounted_children.remove(index);
            Self::remove_or_exit_child(parent, index, mounted, pending_exits.clone())?;
        }

        Ok(())
    }

    // ── 子节点移除 ─────────────────────────────────────────────────────────────

    fn remove_or_exit_child(
        parent: &mut ArkUINode,
        index: usize,
        mut mounted: MountedRenderNode,
        pending_exits: Rc<RefCell<Vec<PendingExit>>>,
    ) -> ArkUIResult<()> {
        let Some(exit_effect) = mounted.exit_effect.take() else {
            mounted.cleanup_recursive();
            let removed = parent.remove_child(index)?;
            if let Some(removed) = removed {
                let mut removed = removed.borrow().clone();
                let _ = removed.dispose();
            }
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
            Self::complete_exiting_child(
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
                Self::complete_exiting_child(
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

    fn complete_exiting_child(
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

        if let Some(mounted) = mounted.borrow_mut().take() {
            mounted.cleanup_recursive();
        }

        let index = parent
            .children()
            .iter()
            .position(|child| child.borrow().raw_handle() as usize == raw_handle);

        match index {
            Some(index) => match parent.remove_child(index) {
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
            },
            None => {}
        }
    }

    // ── 清理 ──────────────────────────────────────────────────────────────────

    pub(super) fn cleanup_recursive(self) {
        for child in self.children {
            child.cleanup_recursive();
        }
        let pending_exits = self
            .exiting_children
            .borrow_mut()
            .drain(..)
            .collect::<Vec<_>>();
        for exit in pending_exits {
            exit.alive.set(false);
            if let Some(cleanup) = exit.effect_cleanup.borrow_mut().take() {
                cleanup();
            }
            if let Some(mounted) = exit.mounted.borrow_mut().take() {
                mounted.cleanup_recursive();
            }
        }
        if let Some(cleanup) = self.long_press_cleanup {
            cleanup();
        }
        if let Some(adapter) = self.virtual_adapter {
            adapter.cleanup();
        }
        run_cleanups(self.cleanups);
    }
}

// ── StateCache（widget 状态缓存）──────────────────────────────────────────────

#[derive(Default)]
pub(super) struct StateCache {
    pub(super) entries: Vec<advanced::widget::Tree>,
}

impl StateCache {
    pub(super) fn store(&mut self, mut tree: advanced::widget::Tree) {
        let Some(key) = tree.persistent_key().map(str::to_string) else {
            return;
        };
        Self::snapshot_tree_state(&mut tree);
        let tag = tree.tag();
        self.entries
            .retain(|entry| !(entry.tag() == tag && entry.persistent_key() == Some(key.as_str())));
        self.entries.push(tree);
    }

    pub(super) fn take(
        &mut self,
        tag: advanced::widget::Tag,
        persistent_key: Option<&str>,
    ) -> Option<advanced::widget::Tree> {
        let persistent_key = persistent_key?;
        let index = self
            .entries
            .iter()
            .position(|tree| tree.tag() == tag && tree.persistent_key() == Some(persistent_key))?;
        Some(self.entries.remove(index))
    }

    fn snapshot_tree_state(tree: &mut advanced::widget::Tree) {
        if let Some(scroll_state) = tree
            .state()
            .downcast_mut::<Rc<RefCell<ScrollState>>>()
            .cloned()
        {
            let offset = scroll_state
                .borrow()
                .node
                .clone()
                .and_then(|mut node| read_scroll_offset(&mut node));
            if let Some(offset) = offset {
                scroll_state.borrow_mut().offset = offset;
            }
        }

        for child in tree.children_mut() {
            Self::snapshot_tree_state(child);
        }
    }

    // ── 树同步 ────────────────────────────────────────────────────────────────

    /// 将 element 的 widget 类型同步到 tree，必要时从缓存恢复或重建 tree。
    pub(super) fn sync_tree<Message, AppTheme>(
        &mut self,
        element: &Element<Message, AppTheme>,
        tree: &mut advanced::widget::Tree,
    ) where
        Message: 'static,
        AppTheme: 'static,
    {
        let widget = element.as_widget();
        let next_tag = widget.tag();
        let next_persistent_key = widget.persistent_key();
        if tree.tag() != next_tag || tree.persistent_key() != next_persistent_key {
            let next_tree = self
                .take(next_tag, next_persistent_key)
                .unwrap_or_else(|| arkit_core::advanced::tree_of(element));
            let previous_tree = std::mem::replace(tree, next_tree);
            self.store(previous_tree);
        }
        widget.diff(tree);
        tree.set_persistent_key(next_persistent_key.map(str::to_string));
    }

    /// 同步一组子 element 到 tree 的 children 中。
    pub(super) fn sync_children<Message, AppTheme>(
        &mut self,
        children: &[Element<Message, AppTheme>],
        tree: &mut advanced::widget::Tree,
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
                    self.take(next_tag, Some(persistent_key))
                        .unwrap_or_else(|| arkit_core::advanced::tree_of(child))
                }
            } else if existing.is_empty() {
                arkit_core::advanced::tree_of(child)
            } else {
                existing.remove(0)
            };
            self.sync_tree(child, &mut child_tree);
            next_trees.push(child_tree);
        }

        for child_tree in existing {
            self.store(child_tree);
        }

        tree.replace_children(next_trees);
    }

    fn sync_composite_child_tree<'a, Message, AppTheme>(
        &mut self,
        tree: &'a mut advanced::widget::Tree,
        index: usize,
        element: &Element<Message, AppTheme>,
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
        self.sync_tree(element, child_tree);
        child_tree
    }

    fn prune_composite_children(
        &mut self,
        tree: &mut advanced::widget::Tree,
        keep: usize,
    ) {
        if tree.children().len() <= keep {
            return;
        }
        for child_tree in tree.children_mut().split_off(keep) {
            self.store(child_tree);
        }
    }

    // ── 编译 ──────────────────────────────────────────────────────────────────

    /// 编译 element 为渲染用的 CompiledElement（展开复合 widget、收集 overlay）。
    pub(super) fn compile<Message, AppTheme>(
        &mut self,
        element: Element<Message, AppTheme>,
        tree: &mut advanced::widget::Tree,
        renderer: &Renderer,
        bind_state: bool,
    ) -> CompiledElement<Message, AppTheme>
    where
        Message: 'static,
        AppTheme: 'static,
    {
        self.sync_tree(&element, tree);

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
            return self.compile_node(*node, tree, renderer, bind_state);
        }

        if component::is_component_widget::<Message, AppTheme>(widget.as_any()) {
            let any = widget.into_any();
            let node =
                component::component_into_node::<Message, AppTheme>(any).unwrap_or_else(|| {
                    panic!(
                        "renderer component downcast failed for {}",
                        type_name::<Component<Message, AppTheme>>()
                    )
                });
            return self.compile_node(node, tree, renderer, bind_state);
        }

        let body = widget.cached_body(tree, renderer);
        let body = match body {
            advanced::Body::Rebuild(body) => body,
            advanced::Body::Retain { overlays } => {
                return CompiledElement {
                    body: Node::new(NodeKind::Retained).into(),
                    overlays: (0..overlays)
                        .map(|_| Node::new(NodeKind::Retained).into())
                        .collect(),
                };
            }
        };
        let compiled_body = {
            let body_tree = self.sync_composite_child_tree(tree, 0, &body);
            self.compile(body, body_tree, renderer, bind_state)
        };

        let overlay = widget.overlay(tree, renderer);
        let mut overlays = compiled_body.overlays;
        if let Some(overlay) = overlay {
            let compiled_overlay = {
                let overlay_tree = self.sync_composite_child_tree(tree, 1, &overlay);
                self.compile(overlay, overlay_tree, renderer, bind_state)
            };
            self.prune_composite_children(tree, 2);
            overlays.push(compiled_overlay.body);
            overlays.extend(compiled_overlay.overlays);
        } else {
            self.prune_composite_children(tree, 1);
        }
        widget.cache_overlay_count(tree, overlays.len());

        CompiledElement {
            body: compiled_body.body,
            overlays,
        }
    }

    fn compile_node<Message, AppTheme>(
        &mut self,
        node: Node<Message, AppTheme>,
        tree: &mut advanced::widget::Tree,
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
        self.sync_children(&children, tree);

        let mut compiled_children = Vec::with_capacity(children.len());
        let mut overlays = Vec::new();

        for (child, child_tree) in children.into_iter().zip(tree.children_mut().iter_mut()) {
            let compiled = self.compile(child, child_tree, renderer, bind_state);
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
}

// ── CompiledElement（编译结果）────────────────────────────────────────────────

pub(super) struct CompiledElement<Message, AppTheme = arkit_core::Theme> {
    pub(super) body: Element<Message, AppTheme>,
    pub(super) overlays: Vec<Element<Message, AppTheme>>,
}

impl<Message, AppTheme> CompiledElement<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    /// 将 body 和所有 overlay 组合成单个根 Stack Element。
    pub(super) fn into_root_element(self) -> Element<Message, AppTheme> {
        let overlay_hit_test = if self.overlays.is_empty() {
            HitTestBehavior::Transparent
        } else {
            HitTestBehavior::Default
        };
        let children = vec![
            self.body,
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
                .children(self.overlays)
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
}

// ── 将 Element 解包为 Node（不经过编译展开）──────────────────────────────────

pub(super) fn into_node<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> Node<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let mut tree = arkit_core::advanced::tree_of(&element);
    let mut state_cache = StateCache::default();
    let compiled = state_cache.compile(element, &mut tree, &Renderer::default(), false);
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

// ── 公共 API：挂载 / 更新 ─────────────────────────────────────────────────────

/// 将 Element 挂载到 ArkUI 树，返回节点和挂载状态。
pub fn mount<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> ArkUIResult<(ArkUINode, MountedNode)>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let mut tree = arkit_core::advanced::tree_of(&element);
    let mut state_cache = StateCache::default();
    let compiled = state_cache.compile(element, &mut tree, &Renderer::default(), true);
    let root = compiled.into_root_element();
    let (node, render) = MountedRenderNode::mount(root)?;
    Ok((node, MountedNode::new(tree, render)))
}

/// 将 pending 的 attach/patch 效果应用到已挂载节点。
pub fn realize_attached_mount(node: &mut ArkUINode, mounted: &mut MountedNode) -> ArkUIResult<()> {
    mounted.render.realize_attached(node)
}

/// 用新 Element 更新已挂载的节点树。
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
    retained_state.sync_tree(&element, tree);
    let compiled = retained_state.compile(element, tree, &Renderer::default(), true);
    let root = compiled.into_root_element();
    render.patch(root, node)
}

// ── 离屏挂载辅助（用于 virtual/list slot）────────────────────────────────────

pub(super) fn mount_detached_element<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> ArkUIResult<(ArkUINode, MountedNode)>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let (tree, root) = compile_detached_element_root(element);
    let (node, render) = MountedRenderNode::mount(root)?;
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
    let compiled = state_cache.compile(element, &mut tree, &Renderer::default(), true);
    if !compiled.overlays.is_empty() {
        ohos_hilog_binding::warn(
            "renderer warning: detached virtual/list slot overlays are ignored".to_string(),
        );
    }
    (tree, compiled.body)
}
