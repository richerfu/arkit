//! Virtual List, Grid, and WaterFlow containers backed by ArkUI `NodeAdapter`.
//!
//! [`use_virtual_node_adapter_rsx`] creates a [`VirtualNodeAdapter`] whose
//! visible items are ordinary Dioxus RSX subtrees. Once the host
//! `list`/`grid`/`waterflow` node is resolved (via [`use_ark_node`]), ArkUI
//! requests only the visible items — true virtualization, not full
//! instantiation.
//!
//! [`use_virtual_node_adapter`] remains the lower-level native-node path for
//! callers that deliberately want to build item content with `NodeBuilder`.

use std::cell::RefCell;
use std::rc::Rc;

use arkit_prelude::{dioxus_core, use_effect, use_hook, use_reactive, Element};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;

use arkit_arkui::{MountItem, RenderItem, VirtualItemMount, VirtualKind, VirtualNodeAdapter};

use crate::ArkHost;

type RsxRenderItem = Rc<dyn Fn(u32) -> Element>;

#[derive(Clone, arkit_prelude::Props)]
struct VirtualRsxItemProps {
    index: u32,
    render_item: RsxRenderItem,
}

impl PartialEq for VirtualRsxItemProps {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Rc::ptr_eq(&self.render_item, &other.render_item)
    }
}

fn virtual_rsx_item_root(props: VirtualRsxItemProps) -> Element {
    (props.render_item)(props.index)
}

/// Create a true virtual List, Grid, or WaterFlow whose visible items are RSX.
///
/// Each visible item owns a small Dioxus subtree mounted directly into the
/// adapter-created ListItem/GridItem/FlowItem wrapper. Components, hooks,
/// signals, event handlers, fragments, and `use_ark_node` therefore work inside
/// an item without constructing the full data set.
///
/// ```ignore
/// let adapter = use_virtual_node_adapter_rsx(
///     VirtualKind::List,
///     rows.len() as u32,
///     move |index| {
///         let row = &rows[index as usize];
///         rsx! {
///             row {
///                 width: "100%",
///                 height: 48.0,
///                 text { "{row.title}" }
///             }
///         }
///     },
/// );
/// ```
#[track_caller]
pub fn use_virtual_node_adapter_rsx(
    kind: VirtualKind,
    total_count: u32,
    render_item: impl Fn(u32) -> Element + 'static,
) -> VirtualNodeAdapter {
    let render_item: RsxRenderItem = Rc::new(render_item);
    let mount_item = rsx_mount_item(render_item);
    let initial_mount_item = mount_item.clone();
    let adapter =
        use_hook(move || VirtualNodeAdapter::new_mounted(kind, total_count, initial_mount_item));

    adapter.set_mount_item(mount_item);
    use_virtual_adapter_count(adapter.clone(), total_count);
    adapter
}

fn rsx_mount_item(render_item: RsxRenderItem) -> MountItem {
    let window_metrics = dioxus_core::try_consume_context::<arkit_runtime::WindowMetricsHandle>();
    let application_lifecycle =
        dioxus_core::try_consume_context::<arkit_runtime::ApplicationLifecycleHandle>();
    let safe_area_policy = dioxus_core::try_consume_context::<arkit_runtime::SafeAreaPolicy>();

    Rc::new(move |index, wrapper| {
        let host = ArkHost::new();
        let dom = arkit_runtime::VirtualDom::new_with_props(
            virtual_rsx_item_root,
            VirtualRsxItemProps {
                index,
                render_item: render_item.clone(),
            },
        );
        dom.provide_root_context(host.clone());
        if let Some(window_metrics) = &window_metrics {
            dom.provide_root_context(window_metrics.clone());
        }
        if let Some(application_lifecycle) = &application_lifecycle {
            dom.provide_root_context(application_lifecycle.clone());
        }
        if let Some(safe_area_policy) = safe_area_policy {
            dom.provide_root_context(safe_area_policy);
        }

        let runtime = arkit_runtime::mount_embedded_virtual_dom(wrapper, dom, Some(Rc::new(host)));
        Ok(VirtualItemMount::retain_with_abandon(runtime, |runtime| {
            runtime.abandon();
        }))
    })
}

/// Create a [`VirtualNodeAdapter`] for a virtual List, Grid, or WaterFlow.
///
/// `kind` selects the container; `total_count` is the full item count;
/// `render_item` returns the content node for a given index (invoked on demand
/// by ArkUI as items scroll into view).
///
/// The adapter must be attached to the host node once it is resolved. The
/// simplest pattern is to use this inside a component that renders a `list`/
/// `grid` element and attach in a `use_effect`:
///
/// ```ignore
/// fn my_list() -> Element {
///     let adapter = use_virtual_node_adapter(VirtualKind::List, 10_000, move |index| {
///         let mut node = create_node_by_tag("row")?;
///         // ... set attributes ...
///         Ok(node)
///     });
///     let node_ref = use_ark_node();
///     use_effect(move || {
///         if let Some(node) = node_ref.peek() {
///             let _ = adapter.attach(&node.borrow());
///         }
///     });
///     rsx! { list { width: "100%", height: "100%" } }
/// }
/// ```
#[track_caller]
pub fn use_virtual_node_adapter(
    kind: VirtualKind,
    total_count: u32,
    render_item: impl Fn(u32) -> ArkUIResult<ArkUINode> + 'static,
) -> VirtualNodeAdapter {
    let render_item: RenderItem = Rc::new(render_item);
    let initial_render_item = render_item.clone();
    let adapter = use_hook(move || VirtualNodeAdapter::new(kind, total_count, initial_render_item));

    // The adapter outlives an individual component render, so always replace
    // its callback with the latest closure. This is Rust-owned state only and
    // cannot re-enter the native event receiver.
    adapter.set_render_item(render_item);
    use_virtual_adapter_count(adapter.clone(), total_count);
    adapter
}

fn use_virtual_adapter_count(adapter: VirtualNodeAdapter, total_count: u32) {
    // Count changes mutate ArkUI and may synchronously emit adapter events.
    // Defer them until after Dioxus commits the render that supplied the new
    // callback and backing data.
    use_effect(use_reactive((&total_count,), move |(next_total,)| {
        if let Err(error) = adapter.set_total_count(next_total) {
            ohos_hilog_binding::error(format!(
                "arkit_hooks: virtual adapter count update failed: {error}"
            ));
        }
    }));
}

/// Create a virtual adapter with item-local invalidation.
///
/// `item_keys[index]` must cover every visual input for that item. Equal-size
/// updates reload only the changed contiguous runs, while count changes are
/// handled by [`use_virtual_node_adapter`]. Keeping distant changes separate
/// is important for selection updates: reloading the entire range between the
/// previous and next selection can disturb a List's scroll anchor.
#[track_caller]
pub fn use_virtual_node_adapter_items_keyed<K>(
    kind: VirtualKind,
    item_keys: Vec<K>,
    render_item: impl Fn(u32) -> ArkUIResult<ArkUINode> + 'static,
) -> VirtualNodeAdapter
where
    K: Clone + PartialEq + 'static,
{
    let total_count = item_keys.len() as u32;
    let adapter = use_virtual_node_adapter(kind, total_count, render_item);
    use_virtual_item_keys(adapter.clone(), item_keys);
    adapter
}

/// RSX virtual adapter with item-local invalidation.
///
/// `item_keys[index]` must cover every non-reactive visual input used to build
/// that item. Equal-size updates rebuild only changed contiguous ranges.
#[track_caller]
pub fn use_virtual_node_adapter_rsx_items_keyed<K>(
    kind: VirtualKind,
    item_keys: Vec<K>,
    render_item: impl Fn(u32) -> Element + 'static,
) -> VirtualNodeAdapter
where
    K: Clone + PartialEq + 'static,
{
    let total_count = item_keys.len() as u32;
    let adapter = use_virtual_node_adapter_rsx(kind, total_count, render_item);
    use_virtual_item_keys(adapter.clone(), item_keys);
    adapter
}

fn use_virtual_item_keys<K>(adapter: VirtualNodeAdapter, item_keys: Vec<K>)
where
    K: Clone + PartialEq + 'static,
{
    let previous_item_keys = use_hook(|| Rc::new(RefCell::new(item_keys.clone())));
    let effect_previous_item_keys = previous_item_keys.clone();

    use_effect(use_reactive((&item_keys,), move |(next_item_keys,)| {
        let previous_item_keys = effect_previous_item_keys.borrow().clone();

        // The base hook owns structural count changes and reloads visible
        // content after updating the native adapter. This effect only handles
        // equal-size item-local changes.
        if previous_item_keys.len() != next_item_keys.len() {
            *effect_previous_item_keys.borrow_mut() = next_item_keys;
            return;
        }
        let changed_ranges = changed_item_ranges(&previous_item_keys, &next_item_keys);
        if changed_ranges.is_empty() {
            return;
        }
        for (start, count) in changed_ranges {
            if let Err(error) = adapter.reload_items(start, count) {
                ohos_hilog_binding::error(format!(
                    "arkit_hooks: item-keyed virtual adapter update failed: {error}"
                ));
                return;
            }
        }
        *effect_previous_item_keys.borrow_mut() = next_item_keys;
    }));
}

fn changed_item_ranges<K: PartialEq>(previous: &[K], next: &[K]) -> Vec<(u32, u32)> {
    debug_assert_eq!(previous.len(), next.len());
    let mut ranges = Vec::new();
    let mut start = None;
    for (index, (previous, next)) in previous.iter().zip(next).enumerate() {
        if previous != next {
            start.get_or_insert(index);
        } else if let Some(range_start) = start.take() {
            ranges.push((range_start as u32, (index - range_start) as u32));
        }
    }
    if let Some(range_start) = start {
        ranges.push((range_start as u32, (previous.len() - range_start) as u32));
    }
    ranges
}

#[cfg(test)]
mod item_key_tests {
    use super::changed_item_ranges;

    #[test]
    fn item_key_diff_keeps_distant_changes_separate() {
        let previous = [0, 1, 2, 3, 4, 5, 6];
        let next = [9, 1, 8, 7, 4, 5, 0];
        assert_eq!(
            changed_item_ranges(&previous, &next),
            vec![(0, 1), (2, 2), (6, 1)]
        );
    }

    #[test]
    fn unchanged_item_keys_do_not_reload_rows() {
        assert!(changed_item_ranges(&[1, 2, 3], &[1, 2, 3]).is_empty());
    }

    #[test]
    fn adjacent_changes_keep_the_reload_range_tight() {
        assert_eq!(
            changed_item_ranges(&[1, 2, 3, 4, 5], &[1, 8, 9, 4, 5]),
            vec![(1, 2)]
        );
    }

    #[test]
    fn moving_selection_reloads_only_previous_and_next_rows() {
        assert_eq!(
            changed_item_ranges(
                &[false, true, false, false, false, false],
                &[false, false, false, false, false, true],
            ),
            vec![(1, 1), (5, 1)]
        );
    }
}
