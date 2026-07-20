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
    /// Mounted items keyed by native handle. During a reload ArkUI may add the
    /// replacement for an index before removing the old node for that same
    /// index. Keying by index would overwrite and then dispose the replacement
    /// when the old removal arrives.
    mounted: FxHashMap<usize, ArkUINode>,
    adapter: Option<NodeAdapter>,
    attached_host: Option<ArkUINode>,
}

/// A virtual adapter attached to a `list`, `grid`, or `waterflow` host node.
/// Clone shares the underlying adapter state.
#[derive(Clone)]
pub struct VirtualNodeAdapter {
    state: Rc<RefCell<AdapterState>>,
}

impl PartialEq for VirtualNodeAdapter {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
}

impl Eq for VirtualNodeAdapter {}

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

    /// Replace the item renderer used by future creates and reloads.
    ///
    /// This only updates Rust-owned state; it does not synchronously mutate
    /// ArkUI. Call [`reload_items`](Self::reload_items) after changing the
    /// backing data for already-mounted rows.
    pub fn set_render_item(&self, render_item: RenderItem) {
        self.state.borrow_mut().render_item = render_item;
    }

    /// Return the current logical item count.
    pub fn total_count(&self) -> u32 {
        self.state.borrow().total_count
    }

    /// Detach and dispose the native adapter and every mounted item.
    pub fn detach(&self) -> ArkUIResult<()> {
        let (kind, host, adapter) = {
            let mut state = self.state.borrow_mut();
            (state.kind, state.attached_host.take(), state.adapter.take())
        };

        // Resetting the attribute and disposing the adapter can synchronously
        // emit removal events. Leave `mounted` in AdapterState so the event
        // receiver remains the single owner that disposes those native nodes.
        let reset_result = host.map_or(Ok(()), |host| host.reset_attribute(kind.adapter_attr()));
        if let Some(adapter) = adapter {
            adapter.dispose();
        }

        // A successful detach should have emitted removal callbacks. Dispose
        // only genuine leftovers; on a failed reset the host may already be
        // invalid, so issuing native operations through copied handles would
        // risk a use-after-free.
        let mounted = std::mem::take(&mut self.state.borrow_mut().mounted);
        if reset_result.is_ok() {
            for (_, mut node) in mounted {
                let _ = node.dispose();
            }
        }
        reset_result
    }

    /// Replace the logical item count and rebuild currently visible content.
    ///
    /// Prefer [`insert_items`](Self::insert_items) and
    /// [`remove_items`](Self::remove_items) when the structural change is
    /// known; they preserve unaffected native rows and scroll state.
    pub fn set_total_count(&self, total: u32) -> ArkUIResult<()> {
        if self.total_count() == total {
            return Ok(());
        }

        self.with_native_adapter(|adapter| adapter.set_total_node_count(total))?;
        self.state.borrow_mut().total_count = total;
        self.reload_all_items()
    }

    /// Re-render all currently visible items without replacing the host
    /// adapter.
    ///
    /// This preserves virtualization while allowing equal-length data updates
    /// such as selection, progress, locale and theme changes.
    pub fn reload_all_items(&self) -> ArkUIResult<()> {
        let total = self.total_count();
        if total == 0 {
            return Ok(());
        }
        self.reload_items(0, total)
    }

    /// Re-render a contiguous item range while preserving unaffected rows.
    pub fn reload_items(&self, start: u32, count: u32) -> ArkUIResult<()> {
        validate_item_range(self.total_count(), start, count)?;
        if count == 0 {
            return Ok(());
        }
        self.with_native_adapter(|adapter| adapter.reload_item(start, count))
    }

    /// Insert `count` logical items at `start` and preserve unaffected rows.
    /// The backing data must be updated before this method is called.
    pub fn insert_items(&self, start: u32, count: u32) -> ArkUIResult<()> {
        let total = self.total_count();
        validate_insert(total, start, count)?;
        if count == 0 {
            return Ok(());
        }
        let next_total = total
            .checked_add(count)
            .ok_or_else(|| invalid_parameter("virtual adapter item count overflowed u32"))?;

        self.with_native_adapter(|adapter| {
            adapter.insert_item(start, count)?;
            if let Err(error) = adapter.set_total_node_count(next_total) {
                let _ = adapter.remove_item(start, count);
                return Err(error);
            }
            Ok(())
        })?;
        self.state.borrow_mut().total_count = next_total;
        Ok(())
    }

    /// Remove `count` logical items at `start` and preserve unaffected rows.
    /// The backing data must be updated before this method is called.
    pub fn remove_items(&self, start: u32, count: u32) -> ArkUIResult<()> {
        let total = self.total_count();
        validate_item_range(total, start, count)?;
        if count == 0 {
            return Ok(());
        }
        let next_total = total - count;

        self.with_native_adapter(|adapter| {
            adapter.remove_item(start, count)?;
            if let Err(error) = adapter.set_total_node_count(next_total) {
                let _ = adapter.insert_item(start, count);
                return Err(error);
            }
            Ok(())
        })?;
        self.state.borrow_mut().total_count = next_total;
        Ok(())
    }

    /// Move one logical item while preserving its native node when possible.
    /// The backing data must be updated before this method is called.
    pub fn move_item(&self, from: u32, to: u32) -> ArkUIResult<()> {
        let total = self.total_count();
        validate_item_index(total, from)?;
        validate_item_index(total, to)?;
        if from == to {
            return Ok(());
        }
        self.with_native_adapter(|adapter| adapter.move_item(from, to))
    }

    /// Run one native adapter mutation without holding the shared state borrow.
    ///
    /// ArkUI may synchronously invoke the registered receiver from any of its
    /// update methods. Temporarily moving the adapter out of `AdapterState`
    /// prevents a nested `RefCell` borrow from panicking in that callback.
    fn with_native_adapter(
        &self,
        mutate: impl FnOnce(&mut NodeAdapter) -> ArkUIResult<()>,
    ) -> ArkUIResult<()> {
        let adapter = self.state.borrow_mut().adapter.take();
        let Some(mut adapter) = adapter else {
            return Ok(());
        };
        let result = mutate(&mut adapter);
        self.state.borrow_mut().adapter = Some(adapter);
        result
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

fn handle_adapter_event(state: &Weak<RefCell<AdapterState>>, event: &mut NodeAdapterEvent) {
    let Some(state) = state.upgrade() else {
        return;
    };
    match event.event_type() {
        NodeAdapterEventType::OnGetNodeId => {
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
                    let node_key = node.raw_handle() as usize;
                    if let Some(mut replaced) = state.borrow_mut().mounted.insert(node_key, node) {
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
            let Some(removed) = event.removed_node() else {
                ohos_hilog_binding::warn(
                    "arkit_arkui: virtual adapter removal did not include a node",
                );
                return;
            };
            let node_key = removed.raw_handle() as usize;
            if let Some(mut node) = state.borrow_mut().mounted.remove(&node_key) {
                let _ = node.dispose();
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

fn validate_item_index(total: u32, index: u32) -> ArkUIResult<()> {
    if index < total {
        Ok(())
    } else {
        Err(invalid_parameter(format!(
            "virtual adapter index {index} is outside item count {total}"
        )))
    }
}

fn validate_item_range(total: u32, start: u32, count: u32) -> ArkUIResult<()> {
    let Some(end) = start.checked_add(count) else {
        return Err(invalid_parameter(
            "virtual adapter item range overflowed u32",
        ));
    };
    if start <= total && end <= total {
        Ok(())
    } else {
        Err(invalid_parameter(format!(
            "virtual adapter range {start}..{end} is outside item count {total}"
        )))
    }
}

fn validate_insert(total: u32, start: u32, count: u32) -> ArkUIResult<()> {
    if start > total {
        return Err(invalid_parameter(format!(
            "virtual adapter insert index {start} is outside item count {total}"
        )));
    }
    total
        .checked_add(count)
        .map(|_| ())
        .ok_or_else(|| invalid_parameter("virtual adapter item count overflowed u32"))
}

fn invalid_parameter(message: impl Into<String>) -> ohos_arkui_binding::common::error::ArkUIError {
    ohos_arkui_binding::common::error::ArkUIError::new(
        ohos_arkui_binding::arkui_input_binding::ArkUIErrorCode::ParamInvalid,
        message.into(),
    )
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{validate_insert, validate_item_range, VirtualKind, VirtualNodeAdapter};

    #[test]
    fn item_ranges_accept_empty_tail_and_bounded_content() {
        assert!(validate_item_range(10, 10, 0).is_ok());
        assert!(validate_item_range(10, 3, 4).is_ok());
    }

    #[test]
    fn item_ranges_reject_overflow_and_out_of_bounds() {
        assert!(validate_item_range(10, 8, 3).is_err());
        assert!(validate_item_range(u32::MAX, u32::MAX, 1).is_err());
    }

    #[test]
    fn insert_accepts_tail_and_rejects_invalid_growth() {
        assert!(validate_insert(10, 10, 2).is_ok());
        assert!(validate_insert(10, 11, 1).is_err());
        assert!(validate_insert(u32::MAX, u32::MAX, 1).is_err());
    }

    #[test]
    fn detached_adapter_applies_structural_updates_to_its_model() {
        let adapter = VirtualNodeAdapter::new(
            VirtualKind::List,
            3,
            Rc::new(|_| panic!("detached updates must not request native rows")),
        );

        adapter.insert_items(1, 2).unwrap();
        assert_eq!(adapter.total_count(), 5);
        adapter.move_item(0, 4).unwrap();
        adapter.reload_items(1, 2).unwrap();
        adapter.remove_items(2, 3).unwrap();
        assert_eq!(adapter.total_count(), 2);
    }
}
