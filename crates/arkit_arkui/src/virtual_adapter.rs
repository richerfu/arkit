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
    /// Rendering generation encoded into the node ids returned to ArkUI.
    ///
    /// `ReloadAllItems` diffs the new data against mounted nodes by node id.
    /// Keeping `index` as the id for equal-length content changes makes ArkUI
    /// reuse the old native subtree, including its old event callbacks. The
    /// generation flips the id namespace on every content reload so visible
    /// rows are rebuilt from the latest render callback.
    render_generation: u32,
    render_item: RenderItem,
    /// Mounted items keyed by native handle. During `reload_all_items`, ArkUI
    /// may add the replacement for an index before removing the old node for
    /// that same index. Keying by index would overwrite and then dispose the
    /// replacement's Rust event receiver when the old removal arrives.
    mounted: FxHashMap<usize, MountedItem>,
    adapter: Option<NodeAdapter>,
    attached_host: Option<ArkUINode>,
}

struct MountedItem {
    index: u32,
    node: ArkUINode,
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
                render_generation: 0,
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
        for (_, mut item) in mounted {
            let _ = item.node.dispose();
        }
        if let Some(adapter) = adapter {
            adapter.dispose();
        }
        reset_result
    }

    /// Update the total item count and notify the adapter to reload.
    pub fn set_total_count(&self, total: u32) -> ArkUIResult<()> {
        let (adapter, previous_generation) = {
            let mut state = self.state.borrow_mut();
            if state.total_count == total {
                return Ok(());
            }
            state.total_count = total;
            let previous_generation = state.render_generation;
            state.render_generation = state.render_generation.wrapping_add(1);
            (state.adapter.take(), previous_generation)
        };
        let Some(mut adapter) = adapter else {
            return Ok(());
        };

        // NodeAdapter mutations synchronously invoke the registered receiver
        // on some ArkUI versions. Do not hold AdapterState's RefCell borrow
        // across either native call: the receiver needs to borrow the same
        // state to remove and build mounted nodes.
        let result = adapter
            .set_total_node_count(total)
            .and_then(|()| adapter.reload_all_items());
        let mut state = self.state.borrow_mut();
        state.adapter = Some(adapter);
        if result.is_err() {
            state.render_generation = previous_generation;
        }
        result
    }

    /// Re-render all mounted items without replacing the host adapter.
    ///
    /// This preserves virtualization while allowing equal-length data updates
    /// such as selection, progress, locale and theme changes.
    pub fn reload_all_items(&self) -> ArkUIResult<()> {
        let (adapter, previous_generation) = {
            let mut state = self.state.borrow_mut();
            let previous_generation = state.render_generation;
            state.render_generation = state.render_generation.wrapping_add(1);
            (state.adapter.take(), previous_generation)
        };
        let Some(mut adapter) = adapter else {
            return Ok(());
        };

        // Reload can synchronously deliver OnRemoveNodeFromAdapter and
        // OnAddNodeToAdapter. Keep AdapterState unborrowed while ArkUI invokes
        // those callbacks.
        let result = adapter.reload_all_items();
        let mut state = self.state.borrow_mut();
        state.adapter = Some(adapter);
        if result.is_err() {
            state.render_generation = previous_generation;
        }
        result
    }

    /// Re-render mounted items in a contiguous range without mutating the
    /// adapter's item sequence.
    ///
    /// Use this for equal-length item-local visual changes. Calling ArkUI's
    /// positional `ReloadItem` for a Grid can change its visible cache anchor
    /// even though the data order is unchanged. Arkit therefore keeps the
    /// ListItem/GridItem/FlowItem wrapper mounted at the same index and only
    /// replaces its content subtree. The adapter order, scroll position and
    /// column assignment are never touched.
    pub fn reload_items(&self, start: u32, count: u32) -> ArkUIResult<()> {
        if count == 0 {
            return Ok(());
        }
        let end = start.saturating_add(count);
        let (render_item, mut targets) = {
            let mut state = self.state.borrow_mut();
            let render_item = state.render_item.clone();
            let keys = state
                .mounted
                .iter()
                .filter_map(|(key, item)| (item.index >= start && item.index < end).then_some(*key))
                .collect::<Vec<_>>();
            let targets = keys
                .into_iter()
                .filter_map(|key| state.mounted.remove(&key).map(|item| (key, item)))
                .collect::<Vec<_>>();
            (render_item, targets)
        };

        let mut first_error = None;
        for (_, item) in &mut targets {
            match render_item(item.index)
                .and_then(|content| replace_item_content(&mut item.node, content))
            {
                Ok(()) => {}
                Err(error) if first_error.is_none() => first_error = Some(error),
                Err(_) => {}
            }
        }

        let mut state = self.state.borrow_mut();
        for (key, item) in targets {
            state.mounted.insert(key, item);
        }
        first_error.map_or(Ok(()), Err)
    }
}

impl Drop for VirtualNodeAdapter {
    fn drop(&mut self) {
        // Only dispose when the last reference drops.
        if Rc::strong_count(&self.state) == 1 {
            // Dioxus removes the rendered host node before it drops the
            // component's hooks. At that point `attached_host` is only a
            // copied raw handle; resetting an attribute through it is a
            // use-after-free in ArkUI. Explicit `detach()` remains available
            // while a host is known to be alive, but the finalizer must only
            // release resources owned by the adapter itself.
            let adapter = {
                let mut state = self.state.borrow_mut();
                state.attached_host = None;
                state.adapter.take()
            };
            if let Some(adapter) = adapter {
                // Disposing the adapter may synchronously emit removal events;
                // those callbacks own disposal while the native items are
                // still valid. Any wrappers left afterwards belonged to the
                // already-removed host subtree, so only drop their Rust-side
                // handles instead of issuing another native dispose.
                adapter.dispose();
            }
            self.state.borrow_mut().mounted.clear();
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
            let index = event.item_index();
            let generation = state.borrow().render_generation;
            let _ = event.set_node_id(node_id_for_generation(index, generation));
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
                    let node_key = node.raw_handle() as usize;
                    let mounted = MountedItem { index, node };
                    if let Some(mut replaced) = state.borrow_mut().mounted.insert(node_key, mounted)
                    {
                        let _ = replaced.node.dispose();
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
            let Some(removed) = event.removed_node() else {
                ohos_hilog_binding::warn(
                    "arkit_arkui: virtual adapter removal did not include a node",
                );
                return;
            };
            let node_key = removed.raw_handle() as usize;
            if let Some(mut item) = state.borrow_mut().mounted.remove(&node_key) {
                let _ = item.node.dispose();
            }
        }
        NodeAdapterEventType::WillAttachToNode => {}
        NodeAdapterEventType::WillDetachFromNode => {
            // The host owns its native lifetime. Once ArkUI starts detaching,
            // keeping its copied raw handle would make a later hook drop try
            // to access an already-disposed node.
            state.borrow_mut().attached_host = None;
        }
    }
}

/// Return a unique, monotonically increasing id for `index` whose namespace
/// changes on every generation.
///
/// ArkUI uses these ids to diff `ReloadAllItems`. The low bit selects one of
/// two disjoint generations while the remaining bits preserve index order.
/// This forces an equal-length content reload to replace the old native node
/// and event callback without exposing a reversed or negative id sequence to
/// the adapter's internal diff.
fn node_id_for_generation(index: u32, generation: u32) -> i32 {
    debug_assert!(index <= 0x3fff_ffff);
    ((index << 1) | (generation & 1)) as i32
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

fn replace_item_content(wrapper: &mut ArkUINode, content: ArkUINode) -> ArkUIResult<()> {
    // Insert first so a failed native insertion leaves the existing subtree
    // intact. Once the replacement is attached, the previous child moves to
    // index 1 and can be detached and disposed safely.
    wrapper.insert_child(content, 0)?;
    if let Some(previous) = wrapper.remove_child(1)? {
        previous.borrow_mut().dispose()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::node_id_for_generation;

    #[test]
    fn reload_generation_changes_every_visible_node_identity() {
        for index in 0..10_000 {
            assert_ne!(
                node_id_for_generation(index, 0),
                node_id_for_generation(index, 1)
            );
        }
    }

    #[test]
    fn node_ids_remain_unique_within_each_generation() {
        for generation in 0..4 {
            let ids = (0..10_000)
                .map(|index| node_id_for_generation(index, generation))
                .collect::<HashSet<_>>();
            assert_eq!(ids.len(), 10_000);
        }
    }

    #[test]
    fn node_ids_are_non_negative_and_preserve_index_order() {
        for generation in 0..4 {
            let ids = (0..10_000)
                .map(|index| node_id_for_generation(index, generation))
                .collect::<Vec<_>>();
            assert!(ids.iter().all(|id| *id >= 0));
            assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
        }
    }
}
