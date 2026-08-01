//! ArkUI `NodeAdapter`-backed virtual container support.
//!
//! A [`VirtualSource`] drives an ArkUI `NodeAdapter` so that only visible
//! items are created (lazy, data-driven), instead of instantiating every child
//! up front. The adapter supports `ListNodeAdapter`, `GridNodeAdapter`, and
//! `WaterFlowNodeAdapter`; [`VirtualKind`] selects the matching host attribute
//! and item wrapper.
//!
//! The adapter supports both a native `render_item` callback returning a fresh
//! [`OwnedNativeNode`] and a direct wrapper mount callback used by the RSX
//! integration. Items and their retained Rust-side owners are released when
//! ArkUI removes them from the adapter.

use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use dioxus_core::{AttributeValue, IntoAttributeValue};
use ohos_arkui_binding::api::attribute_option::{NodeAdapter, NodeAdapterEvent};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::types::advanced::NodeAdapterEventType;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use rustc_hash::FxHashMap;

use crate::{element_ref::SharedNativeNode, OwnedNativeNode};

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
pub type RenderItem = Rc<dyn Fn(u32) -> ArkUIResult<OwnedNativeNode>>;

/// Callback that mounts an item directly into its adapter-owned native wrapper.
///
/// This lower-level boundary powers RSX-backed virtual items. The callback
/// receives the actual ListItem/GridItem/FlowItem wrapper and returns an owner
/// that must stay alive for as long as that wrapper is mounted.
pub type MountItem = Rc<dyn Fn(u32, Rc<RefCell<ArkUINode>>) -> ArkUIResult<VirtualItemMount>>;
type AbandonItemOwner = Box<dyn FnOnce(Box<dyn Any>)>;
type UpdateItemIndex = Rc<dyn Fn(u32)>;

/// Rust-side lifetime owner retained alongside one mounted virtual item.
///
/// Dropping this value tears down item-local state before the adapter disposes
/// the corresponding native wrapper.
pub struct VirtualItemMount {
    owner: Option<Box<dyn Any>>,
    abandon_owner: Option<AbandonItemOwner>,
    update_index: Option<UpdateItemIndex>,
}

impl VirtualItemMount {
    /// Retain an arbitrary item-local owner until ArkUI removes the item.
    pub fn retain(owner: impl Any) -> Self {
        Self {
            owner: Some(Box::new(owner)),
            abandon_owner: None,
            update_index: None,
        }
    }

    /// Retain an owner with a custom invalid-native-root abandonment path.
    ///
    /// The callback is used only when ArkUI has already destroyed the host
    /// subtree without delivering normal item-removal events.
    pub fn retain_with_abandon<T: Any>(owner: T, abandon_owner: impl FnOnce(T) + 'static) -> Self {
        Self {
            owner: Some(Box::new(owner)),
            abandon_owner: Some(Box::new(move |owner| {
                let owner = owner
                    .downcast::<T>()
                    .expect("virtual item owner type changed before abandonment");
                abandon_owner(*owner);
            })),
            update_index: None,
        }
    }

    /// Retain an owner that can observe logical index changes without
    /// replacing its native wrapper.
    pub fn retain_indexed_with_abandon<T: Any>(
        owner: T,
        update_index: impl Fn(&T, u32) + 'static,
        abandon_owner: impl FnOnce(T) + 'static,
    ) -> Self {
        let owner = Rc::new(owner);
        let update_owner = Rc::downgrade(&owner);
        Self {
            owner: Some(Box::new(owner)),
            abandon_owner: Some(Box::new(move |owner| {
                let owner = owner
                    .downcast::<Rc<T>>()
                    .expect("virtual item owner type changed before abandonment");
                let owner = Rc::try_unwrap(*owner).unwrap_or_else(|_| {
                    panic!("virtual item owner was retained during abandonment")
                });
                abandon_owner(owner);
            })),
            update_index: Some(Rc::new(move |index| {
                if let Some(owner) = update_owner.upgrade() {
                    update_index(&owner, index);
                }
            })),
        }
    }

    fn index_updater(&self) -> Option<UpdateItemIndex> {
        self.update_index.clone()
    }

    fn abandon(mut self) {
        if let Some(owner) = self.owner.take() {
            if let Some(abandon_owner) = self.abandon_owner.take() {
                abandon_owner(owner);
            }
        }
    }
}

enum ItemRenderer {
    Content(RenderItem),
    Mounted(MountItem),
}

struct MountedItem {
    index: u32,
    node: Rc<RefCell<ArkUINode>>,
    mount: Option<VirtualItemMount>,
}

impl MountedItem {
    fn prepare_index_update(&mut self, index: u32) -> Option<UpdateItemIndex> {
        if self.index == index {
            return None;
        }
        self.index = index;
        self.mount
            .as_ref()
            .and_then(VirtualItemMount::index_updater)
    }

    fn dispose(mut self) {
        // Item-local runtimes/listeners must stop touching native nodes before
        // the adapter-owned wrapper is destroyed.
        self.mount.take();
        let _ = self.node.borrow_mut().dispose();
    }

    fn abandon(mut self) {
        // The host subtree is already invalid, so item-local cleanup must not
        // call back into its native root. Leaking the retained integration
        // owner is safer than a use-after-free on this exceptional path.
        if let Some(mount) = self.mount.take() {
            mount.abandon();
        }
    }
}

struct AdapterState {
    kind: VirtualKind,
    total_count: u32,
    renderer: ItemRenderer,
    /// Mounted items keyed by native handle. During a reload ArkUI may add the
    /// replacement for an index before removing the old node for that same
    /// index. Keying by index would overwrite and then dispose the replacement
    /// when the old removal arrives.
    mounted: FxHashMap<usize, MountedItem>,
    adapter: Option<NodeAdapter>,
    attached_host: Option<Weak<RefCell<ArkUINode>>>,
}

/// A virtual adapter attached to a `list`, `grid`, or `waterflow` host node.
/// Clone shares the underlying adapter state.
#[derive(Clone)]
pub struct VirtualSource {
    state: Rc<RefCell<AdapterState>>,
}

impl PartialEq for VirtualSource {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
}

impl Eq for VirtualSource {}

impl IntoAttributeValue for VirtualSource {
    fn into_value(self) -> AttributeValue {
        AttributeValue::any_value(self)
    }
}

impl VirtualSource {
    /// Create a native-item source. Assign it to a compatible container's
    /// `virtual_source` attribute; the renderer owns attachment and detachment.
    pub fn new(kind: VirtualKind, total_count: u32, render_item: RenderItem) -> Self {
        Self {
            state: Rc::new(RefCell::new(AdapterState {
                kind,
                total_count,
                renderer: ItemRenderer::Content(render_item),
                mounted: FxHashMap::default(),
                adapter: None,
                attached_host: None,
            })),
        }
    }

    /// Create an adapter whose callback mounts directly into the generated item
    /// wrapper. Most applications should use the RSX hook exposed by
    /// `arkit_hooks`; this constructor exists for renderer integrations.
    #[doc(hidden)]
    pub fn new_mounted(kind: VirtualKind, total_count: u32, mount_item: MountItem) -> Self {
        Self {
            state: Rc::new(RefCell::new(AdapterState {
                kind,
                total_count,
                renderer: ItemRenderer::Mounted(mount_item),
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
    pub(crate) fn attach(&self, host: &SharedNativeNode) -> ArkUIResult<()> {
        let host_handle = host.borrow().raw_handle();
        let already_attached = {
            let state = self.state.borrow();
            state.adapter.is_some()
                && state
                    .attached_host
                    .as_ref()
                    .and_then(Weak::upgrade)
                    .is_some_and(|current| current.borrow().raw_handle() == host_handle)
        };
        if already_attached {
            // ArkUI child insertion may replace only the Rust wrapper while
            // retaining the native handle. Refresh the weak owner so a later
            // renderer teardown can still reset the adapter attribute.
            self.state.borrow_mut().attached_host = Some(Rc::downgrade(host));
            return Ok(());
        }
        self.detach_current()?;

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

        if let Err(error) = host
            .borrow()
            .set_attribute(kind.adapter_attr(), (&adapter).into())
        {
            adapter.dispose();
            return Err(error);
        }
        let mut state = self.state.borrow_mut();
        state.adapter = Some(adapter);
        state.attached_host = Some(Rc::downgrade(host));
        Ok(())
    }

    /// Replace the item renderer used by future creates and reloads.
    ///
    /// This only updates Rust-owned state; it does not synchronously mutate
    /// ArkUI. Call [`reload_items`](Self::reload_items) after changing the
    /// backing data for already-mounted rows.
    pub fn set_render_item(&self, render_item: RenderItem) {
        self.state.borrow_mut().renderer = ItemRenderer::Content(render_item);
    }

    /// Replace the direct item mounter used by future creates and reloads.
    pub fn set_mount_item(&self, mount_item: MountItem) {
        self.state.borrow_mut().renderer = ItemRenderer::Mounted(mount_item);
    }

    /// Return the current logical item count.
    pub fn total_count(&self) -> u32 {
        self.state.borrow().total_count
    }

    /// Detach only if `host` still owns the current source attachment.
    ///
    /// A mutation batch may attach the same source to its new host before the
    /// old host is disposed. The stale owner must not tear down the new one.
    pub(crate) fn detach(&self, host: &SharedNativeNode) -> ArkUIResult<()> {
        let host_handle = host.borrow().raw_handle();
        let owns_attachment = self
            .state
            .borrow()
            .attached_host
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some_and(|current| current.borrow().raw_handle() == host_handle);
        if !owns_attachment {
            return Ok(());
        }
        self.detach_current()
    }

    /// Deliberately detach whichever host currently owns this source.
    fn detach_current(&self) -> ArkUIResult<()> {
        let (kind, host, adapter) = {
            let mut state = self.state.borrow_mut();
            (state.kind, state.attached_host.take(), state.adapter.take())
        };

        // Resetting the attribute and disposing the adapter can synchronously
        // emit removal events. Leave `mounted` in AdapterState so the event
        // receiver remains the single owner that disposes those native nodes.
        let reset_result = host.and_then(|host| host.upgrade()).map_or(Ok(()), |host| {
            host.borrow().reset_attribute(kind.adapter_attr())
        });
        if let Some(adapter) = adapter {
            adapter.dispose();
        }

        // A successful detach should have emitted removal callbacks. Dispose
        // only genuine leftovers; on a failed reset the host may already be
        // invalid, so issuing native operations through copied handles would
        // risk a use-after-free.
        let mounted = std::mem::take(&mut self.state.borrow_mut().mounted);
        if reset_result.is_ok() {
            for (_, item) in mounted {
                item.dispose();
            }
        } else {
            for (_, item) in mounted {
                item.abandon();
            }
        }
        reset_result
    }

    pub(crate) fn abandon_attachment(&self, host: &SharedNativeNode) {
        let host_handle = host.borrow().raw_handle();
        let owns_attachment = self
            .state
            .borrow()
            .attached_host
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some_and(|current| current.borrow().raw_handle() == host_handle);
        if !owns_attachment {
            return;
        }
        let adapter = {
            let mut state = self.state.borrow_mut();
            state.attached_host = None;
            state.adapter.take()
        };
        if let Some(adapter) = adapter {
            adapter.dispose();
        }
        let mounted = std::mem::take(&mut self.state.borrow_mut().mounted);
        for (_, item) in mounted {
            item.abandon();
        }
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

        let mounted = self.mounted_indices();
        self.with_native_adapter(|adapter| {
            adapter.insert_item(start, count)?;
            if let Err(error) = adapter.set_total_node_count(next_total) {
                let _ = adapter.remove_item(start, count);
                return Err(error);
            }
            Ok(())
        })?;
        self.state.borrow_mut().total_count = next_total;
        self.update_mounted_indices(&mounted, |index| (index >= start).then(|| index + count));
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

        let mounted = self.mounted_indices();
        let removed_end = start + count;
        self.with_native_adapter(|adapter| {
            adapter.remove_item(start, count)?;
            if let Err(error) = adapter.set_total_node_count(next_total) {
                let _ = adapter.insert_item(start, count);
                return Err(error);
            }
            Ok(())
        })?;
        self.state.borrow_mut().total_count = next_total;
        self.update_mounted_indices(&mounted, |index| {
            (index >= removed_end).then(|| index - count)
        });
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
        let mounted = self.mounted_indices();
        self.with_native_adapter(|adapter| adapter.move_item(from, to))?;
        self.update_mounted_indices(&mounted, |index| moved_item_index(index, from, to));
        Ok(())
    }

    fn mounted_indices(&self) -> Vec<(usize, u32)> {
        self.state
            .borrow()
            .mounted
            .iter()
            .map(|(handle, item)| (*handle, item.index))
            .collect()
    }

    fn update_mounted_indices(
        &self,
        mounted: &[(usize, u32)],
        next_index: impl Fn(u32) -> Option<u32>,
    ) {
        let updates = {
            let mut state = self.state.borrow_mut();
            let mut updates = Vec::new();
            for (handle, previous_index) in mounted {
                let Some(index) = next_index(*previous_index) else {
                    continue;
                };
                let Some(item) = state.mounted.get_mut(handle) else {
                    continue;
                };
                // A synchronous native callback may have replaced this
                // handle. Only reindex the item represented by the
                // pre-mutation snapshot.
                if item.index == *previous_index {
                    if let Some(update) = item.prepare_index_update(index) {
                        updates.push((update, index));
                    }
                }
            }
            updates
        };
        for (update, index) in updates {
            update(index);
        }
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

impl Drop for VirtualSource {
    fn drop(&mut self) {
        // Only dispose when the last reference drops.
        if Rc::strong_count(&self.state) == 1 {
            // Dioxus removes the rendered host node before it drops the
            // component's hooks. At that point `attached_host` is only a
            // copied raw handle; resetting an attribute through it is a
            // use-after-free in ArkUI. The renderer calls `detach()` while a
            // host is known to be alive; this last-resort finalizer must only
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
            let mounted = std::mem::take(&mut self.state.borrow_mut().mounted);
            for (_, item) in mounted {
                item.abandon();
            }
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
            let (kind, renderer) = {
                let s = state.borrow();
                let renderer = match &s.renderer {
                    ItemRenderer::Content(render_item) => {
                        ItemRenderer::Content(render_item.clone())
                    }
                    ItemRenderer::Mounted(mount_item) => ItemRenderer::Mounted(mount_item.clone()),
                };
                (s.kind, renderer)
            };
            match build_item(kind, index, &renderer) {
                Ok(item) => {
                    let set_item_result = {
                        let node = item.node.borrow();
                        event.set_item(&node)
                    };
                    if let Err(e) = set_item_result {
                        item.dispose();
                        ohos_hilog_binding::error(format!(
                            "arkit_arkui: virtual adapter set_item failed: {e}"
                        ));
                        return;
                    }
                    let node_key = item.node.borrow().raw_handle() as usize;
                    if let Some(replaced) = state.borrow_mut().mounted.insert(node_key, item) {
                        replaced.dispose();
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
            if let Some(item) = state.borrow_mut().mounted.remove(&node_key) {
                item.dispose();
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
fn build_item(kind: VirtualKind, index: u32, renderer: &ItemRenderer) -> ArkUIResult<MountedItem> {
    let wrapper = Rc::new(RefCell::new(kind.create_item_wrapper()?));
    let mount = match renderer {
        ItemRenderer::Content(render_item) => {
            let content = match render_item(index) {
                Ok(content) => content,
                Err(error) => {
                    let _ = wrapper.borrow_mut().dispose();
                    return Err(error);
                }
            };
            if let Err(error) = wrapper.borrow_mut().add_child(content.as_raw().clone()) {
                let _ = wrapper.borrow_mut().dispose();
                return Err(error);
            }
            drop(content.into_raw());
            None
        }
        ItemRenderer::Mounted(mount_item) => match mount_item(index, wrapper.clone()) {
            Ok(mount) => Some(mount),
            Err(error) => {
                let _ = wrapper.borrow_mut().dispose();
                return Err(error);
            }
        },
    };
    Ok(MountedItem {
        index,
        node: wrapper,
        mount,
    })
}

fn moved_item_index(index: u32, from: u32, to: u32) -> Option<u32> {
    if index == from {
        return Some(to);
    }
    if from < to && index > from && index <= to {
        return Some(index - 1);
    }
    if from > to && index >= to && index < from {
        return Some(index + 1);
    }
    None
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
    use std::cell::Cell;
    use std::rc::Rc;

    use super::{
        moved_item_index, validate_insert, validate_item_range, VirtualItemMount, VirtualKind,
        VirtualSource,
    };

    struct DropProbe(Rc<Cell<u32>>);

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

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
        let adapter = VirtualSource::new(
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

    #[test]
    fn virtual_item_mount_drops_its_owner_normally() {
        let drops = Rc::new(Cell::new(0));
        drop(VirtualItemMount::retain(DropProbe(drops.clone())));
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn virtual_item_mount_uses_custom_abandonment() {
        let drops = Rc::new(Cell::new(0));
        let abandons = Rc::new(Cell::new(0));
        let count_abandons = abandons.clone();
        let mount =
            VirtualItemMount::retain_with_abandon(DropProbe(drops.clone()), move |_owner| {
                count_abandons.set(count_abandons.get() + 1);
            });

        mount.abandon();
        assert_eq!(abandons.get(), 1);
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn indexed_mount_observes_retained_item_moves() {
        let index = Rc::new(Cell::new(3));
        let update_index = index.clone();
        let mount = VirtualItemMount::retain_indexed_with_abandon(
            (),
            move |_, next| update_index.set(next),
            |_| {},
        );

        mount.index_updater().unwrap()(7);
        assert_eq!(index.get(), 7);
    }

    #[test]
    fn moving_an_item_reindexes_the_affected_interval() {
        assert_eq!(moved_item_index(1, 1, 4), Some(4));
        assert_eq!(moved_item_index(2, 1, 4), Some(1));
        assert_eq!(moved_item_index(4, 1, 4), Some(3));
        assert_eq!(moved_item_index(0, 1, 4), None);

        assert_eq!(moved_item_index(4, 4, 1), Some(1));
        assert_eq!(moved_item_index(1, 4, 1), Some(2));
        assert_eq!(moved_item_index(3, 4, 1), Some(4));
        assert_eq!(moved_item_index(0, 4, 1), None);
    }
}
