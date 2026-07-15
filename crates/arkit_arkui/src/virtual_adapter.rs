//! ArkUI `NodeAdapter`-backed virtual container support.
//!
//! A [`VirtualNodeAdapter`] drives an ArkUI `NodeAdapter` so that only visible
//! items are created (lazy, data-driven), instead of instantiating every child
//! up front. The adapter supports `ListNodeAdapter`, `GridNodeAdapter`, and
//! `WaterFlowNodeAdapter`; [`VirtualKind`] selects the matching host attribute
//! and item wrapper.
//!
//! `render_item` is a callback invoked on-demand by ArkUI for each visible
//! index; it receives the index and must return a fresh [`ArkUINode`] for that
//! item. Items are disposed when ArkUI removes them from the adapter.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use ohos_arkui_binding::api::attribute_option::{NodeAdapter, NodeAdapterEvent};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::types::advanced::NodeAdapterEventType;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use rustc_hash::FxHashMap;

/// Kind of virtual container — selects which `*NodeAdapter` attribute to set
/// on the host node and which item-wrapper to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualKind {
    List,
    Grid,
    WaterFlow,
}

impl VirtualKind {
    fn adapter_attr(self) -> ArkUINodeAttributeType {
        match self {
            Self::List => ArkUINodeAttributeType::ListNodeAdapter,
            Self::Grid => ArkUINodeAttributeType::GridNodeAdapter,
            Self::WaterFlow => ArkUINodeAttributeType::WaterFlowNodeAdapter,
        }
    }

    /// Create the item-wrapper node for this container kind (ListItem / GridItem
    /// / FlowItem). The actual content node is added as a child of this wrapper.
    fn create_item_wrapper(self) -> ArkUIResult<ArkUINode> {
        use ohos_arkui_binding::component::built_in_component::{FlowItem, GridItem, ListItem};
        Ok(match self {
            Self::List => ListItem::new()?.into(),
            Self::Grid => GridItem::new()?.into(),
            Self::WaterFlow => FlowItem::new()?.into(),
        })
    }
}

/// Callback that renders a single item's content node for the given index.
pub type RenderItem = Rc<dyn Fn(u32) -> ArkUIResult<ArkUINode>>;

struct AdapterState {
    kind: VirtualKind,
    total_count: u32,
    render_item: RenderItem,
    /// Mounted items keyed by index, so we can dispose them on removal.
    mounted: FxHashMap<u32, ArkUINode>,
    adapter: Option<NodeAdapter>,
    attached_host: Option<ArkUINode>,
}

/// A virtual adapter attached to a `list`, `grid`, or `waterflow` host node.
/// Clone shares the underlying adapter state.
#[derive(Clone)]
pub struct VirtualNodeAdapter {
    state: Rc<RefCell<AdapterState>>,
}

impl VirtualNodeAdapter {
    /// Create a new adapter. Call [`attach`](Self::attach) to bind it to a host
    /// node.
    pub fn new(kind: VirtualKind, total_count: u32, render_item: RenderItem) -> Self {
        Self {
            state: Rc::new(RefCell::new(AdapterState {
                kind,
                total_count,
                render_item,
                mounted: FxHashMap::default(),
                adapter: None,
                attached_host: None,
            })),
        }
    }

    /// Attach this adapter to a host `list`/`grid`/`waterflow` node. Creates
    /// the native `NodeAdapter`, sets the total count, registers the event
    /// receiver, and sets the `*NodeAdapter` attribute on the host. Idempotent
    /// for the same host (re-attaching replaces the adapter).
    pub fn attach(&self, host: &ArkUINode) -> ArkUIResult<()> {
        let host_handle = host.raw_handle();
        let already_attached = {
            let state = self.state.borrow();
            state.adapter.is_some()
                && state
                    .attached_host
                    .as_ref()
                    .is_some_and(|current| current.raw_handle() == host_handle)
        };
        if already_attached {
            return Ok(());
        }
        self.detach()?;

        let kind = self.state.borrow().kind;
        let total = self.state.borrow().total_count;

        let mut adapter = NodeAdapter::new()?;
        if let Err(error) = adapter.set_total_node_count(total) {
            adapter.dispose();
            return Err(error);
        }

        let state = Rc::downgrade(&self.state);
        if let Err(error) = adapter.register_event_receiver(move |event| {
            handle_adapter_event(&state, event);
        }) {
            adapter.dispose();
            return Err(error);
        }

        if let Err(error) = host.set_attribute(kind.adapter_attr(), (&adapter).into()) {
            adapter.dispose();
            return Err(error);
        }
        let mut state = self.state.borrow_mut();
        state.adapter = Some(adapter);
        state.attached_host = Some(host.clone());
        Ok(())
    }

    /// Detach and dispose the native adapter and every mounted item.
    pub fn detach(&self) -> ArkUIResult<()> {
        let (kind, host, adapter, mounted) = {
            let mut state = self.state.borrow_mut();
            (
                state.kind,
                state.attached_host.take(),
                state.adapter.take(),
                std::mem::take(&mut state.mounted),
            )
        };
        // Cleanup must not be skipped when the host was already detached by
        // ArkUI and resetting its attribute reports an error.
        let reset_result = host.map_or(Ok(()), |host| host.reset_attribute(kind.adapter_attr()));
        for (_, mut node) in mounted {
            let _ = node.dispose();
        }
        if let Some(adapter) = adapter {
            adapter.dispose();
        }
        reset_result
    }

    /// Update the total item count and notify the adapter to reload.
    pub fn set_total_count(&self, total: u32) -> ArkUIResult<()> {
        let mut state = self.state.borrow_mut();
        if state.total_count == total {
            return Ok(());
        }
        state.total_count = total;
        if let Some(adapter) = state.adapter.as_mut() {
            adapter.set_total_node_count(total)?;
            adapter.reload_all_items()?;
        }
        Ok(())
    }
}

impl Drop for VirtualNodeAdapter {
    fn drop(&mut self) {
        // Only dispose when the last reference drops.
        if Rc::strong_count(&self.state) == 1 {
            let _ = self.detach();
        }
    }
}

/// Backwards-compatible name for [`VirtualNodeAdapter`].
///
/// New code should use the container-neutral name because the adapter also
/// owns Grid and WaterFlow virtualization.
pub type VirtualListAdapter = VirtualNodeAdapter;

fn handle_adapter_event(state: &Weak<RefCell<AdapterState>>, event: &mut NodeAdapterEvent) {
    let Some(state) = state.upgrade() else {
        return;
    };
    match event.event_type() {
        NodeAdapterEventType::OnGetNodeId => {
            // Use the item index as its node id.
            let index = event.item_index();
            let _ = event.set_node_id(index as i32);
        }
        NodeAdapterEventType::OnAddNodeToAdapter => {
            let index = event.item_index();
            let (kind, render_item) = {
                let s = state.borrow();
                (s.kind, s.render_item.clone())
            };
            match build_item(kind, index, &render_item) {
                Ok(mut node) => {
                    if let Err(e) = event.set_item(&node) {
                        let _ = node.dispose();
                        ohos_hilog_binding::error(format!(
                            "arkit_arkui: virtual adapter set_item failed: {e}"
                        ));
                        return;
                    }
                    if let Some(mut replaced) = state.borrow_mut().mounted.insert(index, node) {
                        let _ = replaced.dispose();
                    }
                }
                Err(e) => {
                    ohos_hilog_binding::error(format!(
                        "arkit_arkui: virtual adapter render_item({index}) failed: {e}"
                    ));
                }
            }
        }
        NodeAdapterEventType::OnRemoveNodeFromAdapter => {
            let index = event.item_index();
            if let Some(mut node) = state.borrow_mut().mounted.remove(&index) {
                let _ = node.dispose();
            }
        }
        NodeAdapterEventType::WillAttachToNode | NodeAdapterEventType::WillDetachFromNode => {}
    }
}

/// Build a single virtual item: a wrapper (ListItem/GridItem/FlowItem)
/// containing the content node returned by `render_item`.
fn build_item(kind: VirtualKind, index: u32, render_item: &RenderItem) -> ArkUIResult<ArkUINode> {
    let mut wrapper = kind.create_item_wrapper()?;
    let content = match render_item(index) {
        Ok(content) => content,
        Err(error) => {
            let _ = wrapper.dispose();
            return Err(error);
        }
    };
    if let Err(error) = wrapper.add_child(content) {
        let _ = wrapper.dispose();
        return Err(error);
    }
    Ok(wrapper)
}
