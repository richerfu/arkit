//! Virtual List, Grid, and WaterFlow containers backed by ArkUI `NodeAdapter`.
//!
//! [`use_virtual_node_adapter`] creates a [`VirtualNodeAdapterHandle`] that,
//! once the host `list`/`grid`/`waterflow` node is resolved (via
//! [`use_ark_node`]), attaches the matching ArkUI `NodeAdapter` so only visible
//! items are created on demand — true virtualization, not full instantiation.
//!
//! `render_item` is invoked by ArkUI on demand for each visible index and must
//! return a fresh [`ArkUINode`] for that item's content (the adapter wraps it
//! in a ListItem/GridItem/FlowItem automatically).

use std::cell::RefCell;
use std::rc::Rc;

use arkit_prelude::{use_effect, use_hook, use_reactive};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;

use arkit_arkui::{VirtualKind, VirtualNodeAdapter};

/// Callback that renders a single virtual item's content node.
pub type VirtualRenderItem = Rc<dyn Fn(u32) -> ArkUIResult<ArkUINode>>;

/// Handle returned by [`use_virtual_node_adapter`]. Call
/// [`attach`](Self::attach) once the host node is available (e.g. inside a
/// `use_effect` or `use_layout_frame_node` callback).
#[derive(Clone)]
pub struct VirtualNodeAdapterHandle {
    adapter: VirtualNodeAdapter,
}

impl VirtualNodeAdapterHandle {
    /// Attach the adapter to a host `list`/`grid`/`waterflow` node. Idempotent.
    pub fn attach(&self, host: &ArkUINode) -> ArkUIResult<()> {
        self.adapter.attach(host)
    }

    /// Update the total item count and reload.
    pub fn set_total_count(&self, total: u32) -> ArkUIResult<()> {
        self.adapter.set_total_count(total)
    }

    /// Re-render the mounted native items while keeping the host adapter.
    pub fn reload_all_items(&self) -> ArkUIResult<()> {
        self.adapter.reload_all_items()
    }

    /// Re-render a contiguous range while preserving all unaffected rows.
    pub fn reload_items(&self, start: u32, count: u32) -> ArkUIResult<()> {
        self.adapter.reload_items(start, count)
    }
}

struct KeyedVirtualAdapterState<K> {
    key: K,
    total_count: u32,
    render_item: Rc<RefCell<VirtualRenderItem>>,
    adapter: VirtualNodeAdapter,
}

struct ItemKeyedVirtualAdapterState<K> {
    item_keys: Vec<K>,
    render_item: Rc<RefCell<VirtualRenderItem>>,
    adapter: VirtualNodeAdapter,
}

/// Backwards-compatible name for [`VirtualNodeAdapterHandle`].
pub type VirtualListHandle = VirtualNodeAdapterHandle;

/// Create a [`VirtualNodeAdapterHandle`] for a virtual List, Grid, or WaterFlow.
///
/// `kind` selects the container; `total_count` is the full item count;
/// `render_item` returns the content node for a given index (invoked on demand
/// by ArkUI as items scroll into view).
///
/// The handle must be attached to the host node once it is resolved. The
/// simplest pattern is to use this inside a component that renders a `list`/
/// `grid` element and attach in a `use_effect`:
///
/// ```ignore
/// fn my_list() -> Element {
///     let handle = use_virtual_node_adapter(VirtualKind::List, 10_000, move |index| {
///         let mut node = create_node_by_tag("row")?;
///         // ... set attributes ...
///         Ok(node)
///     });
///     let node_ref = use_ark_node();
///     use_effect(move || {
///         if let Some(node) = node_ref.peek() {
///             let _ = handle.attach(&node.borrow());
///         }
///     });
///     rsx! { list { percent_width: 1.0, percent_height: 1.0 } }
/// }
/// ```
#[track_caller]
pub fn use_virtual_node_adapter(
    kind: VirtualKind,
    total_count: u32,
    render_item: impl Fn(u32) -> ArkUIResult<ArkUINode> + 'static,
) -> VirtualNodeAdapterHandle {
    let render_item: VirtualRenderItem = Rc::new(render_item);
    let adapter = use_hook(|| VirtualNodeAdapter::new(kind, total_count, render_item.clone()));
    // Keep the adapter's total count in sync if the hook re-runs with a new
    // count (cheap no-op when unchanged).
    let _ = adapter.set_total_count(total_count);
    VirtualNodeAdapterHandle { adapter }
}

/// Create a reactive virtual adapter whose mounted items reload only when the
/// caller-provided data key or total count changes.
///
/// The render callback is replaced on every component render, while the key
/// prevents unrelated parent renders from rebuilding native rows. Use a hash
/// or monotonically increasing revision that covers every visual data input.
#[track_caller]
pub fn use_virtual_node_adapter_keyed<K>(
    kind: VirtualKind,
    total_count: u32,
    data_key: K,
    render_item: impl Fn(u32) -> ArkUIResult<ArkUINode> + 'static,
) -> VirtualNodeAdapterHandle
where
    K: Clone + PartialEq + 'static,
{
    let next_render_item: VirtualRenderItem = Rc::new(render_item);
    let initial_render_item = next_render_item.clone();
    let state = use_hook(|| {
        let render_item = Rc::new(RefCell::new(initial_render_item));
        let render_item_for_adapter = render_item.clone();
        let adapter_render_item: VirtualRenderItem = Rc::new(move |index| {
            let render = render_item_for_adapter.borrow().clone();
            render(index)
        });
        Rc::new(RefCell::new(KeyedVirtualAdapterState {
            key: data_key.clone(),
            total_count,
            render_item,
            adapter: VirtualNodeAdapter::new(kind, total_count, adapter_render_item),
        }))
    });

    // Updating this indirection is render-local state only. The native ArkUI
    // mutation is deliberately deferred to an effect below: Dioxus component
    // bodies may be re-run while diffing and must not synchronously mutate the
    // mounted host tree.
    *state.borrow().render_item.borrow_mut() = next_render_item;

    let effect_state = state.clone();
    use_effect(use_reactive(
        (&total_count, &data_key),
        move |(next_total_count, next_key)| {
            let (adapter, count_changed, key_changed) = {
                let state = effect_state.borrow();
                (
                    state.adapter.clone(),
                    state.total_count != next_total_count,
                    state.key != next_key,
                )
            };

            let result = if count_changed {
                adapter.set_total_count(next_total_count)
            } else if key_changed {
                adapter.reload_all_items()
            } else {
                Ok(())
            };

            if let Err(error) = result {
                ohos_hilog_binding::error(format!(
                    "arkit_hooks: virtual adapter update failed: {error}"
                ));
                return;
            }

            let mut state = effect_state.borrow_mut();
            state.total_count = next_total_count;
            state.key = next_key;
        },
    ));

    let adapter = state.borrow().adapter.clone();
    VirtualNodeAdapterHandle { adapter }
}

/// Create a reactive virtual adapter with item-local invalidation.
///
/// `item_keys[index]` must cover every visual input for that row. When the
/// component re-renders with the same number of items, Arkit compares keys and
/// refreshes the mounted content subtrees in the smallest range containing
/// every changed row. The native item wrappers remain attached at their
/// original indices, so Grid column assignment and scroll anchoring stay
/// stable.
#[track_caller]
pub fn use_virtual_node_adapter_items_keyed<K>(
    kind: VirtualKind,
    item_keys: Vec<K>,
    render_item: impl Fn(u32) -> ArkUIResult<ArkUINode> + 'static,
) -> VirtualNodeAdapterHandle
where
    K: Clone + PartialEq + 'static,
{
    let total_count = item_keys.len() as u32;
    let next_render_item: VirtualRenderItem = Rc::new(render_item);
    let initial_render_item = next_render_item.clone();
    let initial_item_keys = item_keys.clone();
    let state = use_hook(|| {
        let render_item = Rc::new(RefCell::new(initial_render_item));
        let render_item_for_adapter = render_item.clone();
        let adapter_render_item: VirtualRenderItem = Rc::new(move |index| {
            let render = render_item_for_adapter.borrow().clone();
            render(index)
        });
        Rc::new(RefCell::new(ItemKeyedVirtualAdapterState {
            item_keys: initial_item_keys,
            render_item,
            adapter: VirtualNodeAdapter::new(kind, total_count, adapter_render_item),
        }))
    });

    *state.borrow().render_item.borrow_mut() = next_render_item;

    let effect_state = state.clone();
    use_effect(use_reactive((&item_keys,), move |(next_item_keys,)| {
        let (adapter, previous_item_keys) = {
            let state = effect_state.borrow();
            (state.adapter.clone(), state.item_keys.clone())
        };

        let result = if previous_item_keys.len() != next_item_keys.len() {
            adapter.set_total_count(next_item_keys.len() as u32)
        } else if let Some((start, count)) =
            changed_item_range(&previous_item_keys, &next_item_keys)
        {
            adapter.reload_items(start, count)
        } else {
            Ok(())
        };

        if let Err(error) = result {
            ohos_hilog_binding::error(format!(
                "arkit_hooks: item-keyed virtual adapter update failed: {error}"
            ));
            return;
        }
        effect_state.borrow_mut().item_keys = next_item_keys;
    }));

    let adapter = state.borrow().adapter.clone();
    VirtualNodeAdapterHandle { adapter }
}

fn changed_item_range<K: PartialEq>(previous: &[K], next: &[K]) -> Option<(u32, u32)> {
    debug_assert_eq!(previous.len(), next.len());
    let first = previous
        .iter()
        .zip(next)
        .position(|(previous, next)| previous != next)?;
    let last = previous
        .iter()
        .zip(next)
        .rposition(|(previous, next)| previous != next)
        .unwrap_or(first);
    Some((first as u32, (last - first + 1) as u32))
}

#[cfg(test)]
mod tests {
    use super::changed_item_range;

    #[test]
    fn item_key_diff_returns_one_atomic_bounding_range() {
        let previous = [0, 1, 2, 3, 4, 5, 6];
        let next = [9, 1, 8, 7, 4, 5, 0];
        assert_eq!(changed_item_range(&previous, &next), Some((0, 7)));
    }

    #[test]
    fn unchanged_item_keys_do_not_reload_rows() {
        assert_eq!(changed_item_range(&[1, 2, 3], &[1, 2, 3]), None);
    }

    #[test]
    fn adjacent_changes_keep_the_reload_range_tight() {
        assert_eq!(
            changed_item_range(&[1, 2, 3, 4, 5], &[1, 8, 9, 4, 5]),
            Some((1, 2))
        );
    }
}

/// Backwards-compatible container-generic hook.
///
/// Despite its original name this accepts all [`VirtualKind`] values,
/// including [`VirtualKind::WaterFlow`]. New code should use
/// [`use_virtual_node_adapter`] or [`use_virtual_water_flow`].
#[track_caller]
pub fn use_virtual_list(
    kind: VirtualKind,
    total_count: u32,
    render_item: impl Fn(u32) -> ArkUIResult<ArkUINode> + 'static,
) -> VirtualListHandle {
    use_virtual_node_adapter(kind, total_count, render_item)
}

/// Create a first-class WaterFlow adapter using ArkUI
/// `WaterFlowNodeAdapter` and `FlowItem` wrappers.
#[track_caller]
pub fn use_virtual_water_flow(
    total_count: u32,
    render_item: impl Fn(u32) -> ArkUIResult<ArkUINode> + 'static,
) -> VirtualNodeAdapterHandle {
    use_virtual_node_adapter(VirtualKind::WaterFlow, total_count, render_item)
}
