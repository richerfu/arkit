use super::*;

#[derive(Default)]
pub(super) struct ScrollState {
    pub(super) offset: ScrollOffset,
    pub(super) viewport: Option<ScrollViewport>,
    pub(super) node: Option<ArkUINode>,
}

pub struct MountedNode {
    pub(super) tree: advanced::widget::Tree,
    pub(super) retained_state: StateCache,
    pub(super) render: MountedRenderNode,
}

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

pub(super) struct PendingExit {
    pub(super) raw_handle: usize,
    pub(super) alive: Rc<Cell<bool>>,
    pub(super) mounted: Rc<RefCell<Option<MountedRenderNode>>>,
    pub(super) effect_cleanup: Rc<RefCell<Option<Cleanup>>>,
}

#[derive(Default)]
pub(super) struct StateCache {
    pub(super) entries: Vec<advanced::widget::Tree>,
}

impl StateCache {
    pub(super) fn store(&mut self, mut tree: advanced::widget::Tree) {
        let Some(key) = tree.persistent_key().map(str::to_string) else {
            return;
        };
        snapshot_tree_state(&mut tree);
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
}

pub(super) fn snapshot_tree_state(tree: &mut advanced::widget::Tree) {
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
        snapshot_tree_state(child);
    }
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

impl MountedRenderNode {
    pub(super) fn new(
        tag: TypeId,
        key: Option<String>,
        attrs: Vec<ArkUINodeAttributeType>,
        events: Vec<NodeEventType>,
        mount_effect_count: usize,
        attach_effect_count: usize,
        patch_effect_count: usize,
        has_long_press: bool,
        long_press_cleanup: Option<Cleanup>,
        long_press_callback: Option<Rc<RefCell<Rc<dyn Fn()>>>>,
        cleanups: Vec<Cleanup>,
        exit_effect: Option<ExitEffect>,
        pending_patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
        pending_attach_effects: Vec<AttachEffect>,
        pending_patch_effects: Vec<PatchEffect>,
        virtual_adapter_kind: Option<VirtualContainerKind>,
        virtual_adapter: Option<Box<dyn MountedVirtualAdapter>>,
        children: Vec<MountedRenderNode>,
    ) -> Self {
        Self {
            tag,
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
            pending_attach_effects,
            pending_patch_effects,
            virtual_adapter_kind,
            virtual_adapter,
            children,
        }
    }

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
