use super::*;

pub(super) static NEXT_VIRTUAL_ADAPTER_ID: AtomicI32 = AtomicI32::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VirtualContainerKind {
    List,
    Grid,
    WaterFlow,
    ListItemGroup,
}

pub(super) struct VirtualMountedItem {
    pub(super) node: ArkUINode,
    pub(super) mounted: MountedNode,
}

pub(super) struct VirtualAdapterState<Message, AppTheme = arkit_core::Theme> {
    pub(super) id: i32,
    pub(super) kind: VirtualContainerKind,
    pub(super) total_count: u32,
    pub(super) render_item: Rc<dyn Fn(u32) -> Element<Message, AppTheme>>,
    pub(super) adapter: Option<NodeAdapter>,
    pub(super) mounted_items: HashMap<u32, VirtualMountedItem>,
}

impl<Message, AppTheme> VirtualAdapterState<Message, AppTheme> {
    pub(super) fn new(
        kind: VirtualContainerKind,
        total_count: u32,
        render_item: Rc<dyn Fn(u32) -> Element<Message, AppTheme>>,
    ) -> Self {
        Self {
            id: NEXT_VIRTUAL_ADAPTER_ID.fetch_add(1, Ordering::Relaxed),
            kind,
            total_count,
            render_item,
            adapter: None,
            mounted_items: HashMap::new(),
        }
    }

    pub(super) fn node_id(&self, index: u32) -> i32 {
        self.id.wrapping_mul(1_000_003).wrapping_add(index as i32)
    }
}

pub(super) struct VirtualAdapterSpec<Message, AppTheme = arkit_core::Theme> {
    pub(super) kind: VirtualContainerKind,
    pub(super) total_count: u32,
    pub(super) render_item: Rc<dyn Fn(u32) -> Element<Message, AppTheme>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VirtualAdapterCountChange {
    Unchanged,
    Insert { start: u32, count: u32 },
    Remove { start: u32, count: u32 },
}

pub(super) fn virtual_adapter_count_change(
    previous_total: u32,
    next_total: u32,
) -> VirtualAdapterCountChange {
    match previous_total.cmp(&next_total) {
        std::cmp::Ordering::Less => VirtualAdapterCountChange::Insert {
            start: previous_total,
            count: next_total - previous_total,
        },
        std::cmp::Ordering::Greater => VirtualAdapterCountChange::Remove {
            start: next_total,
            count: previous_total - next_total,
        },
        std::cmp::Ordering::Equal => VirtualAdapterCountChange::Unchanged,
    }
}

pub(super) trait MountedVirtualAdapter {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn cleanup(self: Box<Self>);
}

pub(super) struct MountedVirtualAdapterState<Message, AppTheme = arkit_core::Theme> {
    pub(super) state: Rc<RefCell<VirtualAdapterState<Message, AppTheme>>>,
}

impl<Message, AppTheme> MountedVirtualAdapter for MountedVirtualAdapterState<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn cleanup(self: Box<Self>) {
        cleanup_virtual_adapter_state(&self.state);
    }
}

pub(super) fn wrap_virtual_item<Message, AppTheme>(
    kind: VirtualContainerKind,
    index: u32,
    item: Element<Message, AppTheme>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    match kind {
        VirtualContainerKind::List | VirtualContainerKind::ListItemGroup => list_item_component()
            .key(format!("virtual-list-item-{index}"))
            .child(item)
            .into(),
        VirtualContainerKind::Grid => grid_item_component()
            .key(format!("virtual-grid-item-{index}"))
            .child(item)
            .into(),
        VirtualContainerKind::WaterFlow => flow_item_component()
            .key(format!("virtual-flow-item-{index}"))
            .child(item)
            .into(),
    }
}

pub(super) fn mount_virtual_item<Message, AppTheme>(
    kind: VirtualContainerKind,
    index: u32,
    render_item: &Rc<dyn Fn(u32) -> Element<Message, AppTheme>>,
) -> ArkUIResult<VirtualMountedItem>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let item = wrap_virtual_item(kind, index, render_item(index));
    let (node, mounted) = mount_detached_element(item)?;
    Ok(VirtualMountedItem { node, mounted })
}

pub(super) fn cleanup_virtual_item(mut item: VirtualMountedItem) {
    item.mounted.cleanup_recursive();
    let _ = item.node.dispose();
}

pub(super) fn cleanup_virtual_adapter_state<Message, AppTheme>(
    state: &Rc<RefCell<VirtualAdapterState<Message, AppTheme>>>,
) {
    let mut state = state.borrow_mut();
    if let Some(adapter) = state.adapter.take() {
        adapter.dispose();
    }
    let items = std::mem::take(&mut state.mounted_items);
    for (_, item) in items {
        cleanup_virtual_item(item);
    }
}

pub(super) fn adapter_attr(kind: VirtualContainerKind) -> ArkUINodeAttributeType {
    match kind {
        VirtualContainerKind::List => ArkUINodeAttributeType::ListNodeAdapter,
        VirtualContainerKind::Grid => ArkUINodeAttributeType::GridNodeAdapter,
        VirtualContainerKind::WaterFlow => ArkUINodeAttributeType::WaterFlowNodeAdapter,
        VirtualContainerKind::ListItemGroup => ArkUINodeAttributeType::ListItemGroupNodeAdapter,
    }
}

pub(super) fn set_adapter_attribute(
    node: &mut ArkUINode,
    kind: VirtualContainerKind,
    adapter: &NodeAdapter,
) -> ArkUIResult<()> {
    RuntimeNode(node).set_attribute(adapter_attr(kind), adapter.into())
}

pub(super) fn handle_node_adapter_event<Message, AppTheme>(
    state: &Rc<RefCell<VirtualAdapterState<Message, AppTheme>>>,
    event: &mut NodeAdapterEvent,
) where
    Message: Send + 'static,
    AppTheme: 'static,
{
    match event.event_type() {
        NodeAdapterEventType::OnGetNodeId => {
            let index = event.item_index();
            let node_id = state.borrow().node_id(index);
            if let Err(error) = event.set_node_id(node_id) {
                ohos_hilog_binding::error(format!(
                    "renderer error: failed to set virtual item node id: {error}"
                ));
            }
        }
        NodeAdapterEventType::OnAddNodeToAdapter => {
            let index = event.item_index();
            let (kind, render_item) = {
                let state = state.borrow();
                (state.kind, state.render_item.clone())
            };
            match mount_virtual_item(kind, index, &render_item) {
                Ok(mut item) => {
                    if let Err(error) = event.set_item(&item.node) {
                        ohos_hilog_binding::error(format!(
                            "renderer error: failed to set virtual adapter item: {error}"
                        ));
                        cleanup_virtual_item(item);
                        return;
                    }
                    if let Err(error) = realize_attached_mount(&mut item.node, &mut item.mounted) {
                        ohos_hilog_binding::error(format!(
                            "renderer error: failed to realize virtual adapter item: {error}"
                        ));
                    }
                    if let Some(previous) = state.borrow_mut().mounted_items.insert(index, item) {
                        cleanup_virtual_item(previous);
                    }
                }
                Err(error) => {
                    ohos_hilog_binding::error(format!(
                        "renderer error: failed to mount virtual adapter item: {error}"
                    ));
                }
            }
        }
        NodeAdapterEventType::OnRemoveNodeFromAdapter => {
            let index = event.item_index();
            if let Some(item) = state.borrow_mut().mounted_items.remove(&index) {
                cleanup_virtual_item(item);
            }
        }
        NodeAdapterEventType::WillAttachToNode | NodeAdapterEventType::WillDetachFromNode => {}
    }
}

pub(super) fn mount_virtual_adapter<Message, AppTheme>(
    node: &mut ArkUINode,
    spec: Option<VirtualAdapterSpec<Message, AppTheme>>,
) -> ArkUIResult<Option<Box<dyn MountedVirtualAdapter>>>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let Some(spec) = spec else {
        return Ok(None);
    };
    let state = Rc::new(RefCell::new(VirtualAdapterState::new(
        spec.kind,
        spec.total_count,
        spec.render_item,
    )));
    let mut adapter = NodeAdapter::new()?;
    adapter.set_total_node_count(spec.total_count)?;
    let event_state = state.clone();
    adapter.register_event_receiver(move |event| {
        handle_node_adapter_event(&event_state, event);
    })?;
    set_adapter_attribute(node, spec.kind, &adapter)?;
    state.borrow_mut().adapter = Some(adapter);
    Ok(Some(Box::new(MountedVirtualAdapterState { state })))
}

pub(super) fn apply_virtual_adapter_count_change(
    adapter: &mut NodeAdapter,
    change: VirtualAdapterCountChange,
    next_total: u32,
) -> ArkUIResult<()> {
    match change {
        VirtualAdapterCountChange::Unchanged => Ok(()),
        VirtualAdapterCountChange::Insert { start, count } => {
            adapter.set_total_node_count(next_total)?;
            if let Err(error) = adapter.insert_item(start, count) {
                ohos_hilog_binding::error(format!(
                    "renderer error: failed to insert virtual adapter items: {error}"
                ));
                adapter.reload_all_items()?;
            }
            Ok(())
        }
        VirtualAdapterCountChange::Remove { start, count } => {
            if let Err(error) = adapter.remove_item(start, count) {
                ohos_hilog_binding::error(format!(
                    "renderer error: failed to remove virtual adapter items: {error}"
                ));
                adapter.set_total_node_count(next_total)?;
                adapter.reload_all_items()?;
                return Ok(());
            }
            adapter.set_total_node_count(next_total)
        }
    }
}

pub(super) fn patch_virtual_adapter<Message, AppTheme>(
    node: &mut ArkUINode,
    mounted: &mut MountedRenderNode,
    spec: Option<VirtualAdapterSpec<Message, AppTheme>>,
) -> ArkUIResult<()>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    match (mounted.virtual_adapter.as_mut(), spec) {
        (Some(adapter), Some(spec)) if mounted.virtual_adapter_kind == Some(spec.kind) => {
            let Some(adapter) = adapter
                .as_any_mut()
                .downcast_mut::<MountedVirtualAdapterState<Message, AppTheme>>()
            else {
                let kind = spec.kind;
                if let Some(adapter) = mounted.virtual_adapter.take() {
                    adapter.cleanup();
                }
                mounted.virtual_adapter = mount_virtual_adapter(node, Some(spec))?;
                mounted.virtual_adapter_kind = Some(kind);
                return Ok(());
            };
            let mut state = adapter.state.borrow_mut();
            let previous_total = state.total_count;
            let count_change = virtual_adapter_count_change(previous_total, spec.total_count);
            state.total_count = spec.total_count;
            state.render_item = spec.render_item;
            if let Some(native_adapter) = state.adapter.as_mut() {
                apply_virtual_adapter_count_change(native_adapter, count_change, spec.total_count)?;
            }
        }
        (Some(_), Some(spec)) => {
            if let Some(adapter) = mounted.virtual_adapter.take() {
                adapter.cleanup();
            }
            mounted.virtual_adapter_kind = Some(spec.kind);
            mounted.virtual_adapter = mount_virtual_adapter(node, Some(spec))?;
        }
        (Some(_), None) => {
            if let Some(adapter) = mounted.virtual_adapter.take() {
                adapter.cleanup();
            }
            mounted.virtual_adapter_kind = None;
        }
        (None, Some(spec)) => {
            mounted.virtual_adapter_kind = Some(spec.kind);
            mounted.virtual_adapter = mount_virtual_adapter(node, Some(spec))?;
        }
        (None, None) => {}
    }
    Ok(())
}
