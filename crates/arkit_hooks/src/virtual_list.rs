//! Virtual List, Grid, and WaterFlow containers backed by ArkUI `NodeAdapter`.
//!
//! [`use_virtual_source`] accepts either an RSX [`Element`] or an
//! [`ArkUIResult<OwnedNativeNode>`] from its item callback. Assign the returned
//! source to the container's `virtual_source` attribute; the renderer owns
//! attachment and detachment. ArkUI then requests only visible items.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use arkit_prelude::{dioxus_core, use_effect, use_hook, use_reactive, Element};
use ohos_arkui_binding::common::error::ArkUIResult;

use arkit_arkui::{
    MountItem, OwnedNativeNode, RenderItem, VirtualItemMount, VirtualKind, VirtualSource,
};

type RsxRenderItem = Rc<dyn Fn(u32) -> Element>;
type SharedRsxRenderItem = Rc<RefCell<RsxRenderItem>>;

mod sealed {
    pub trait Sealed {}
}

/// A supported item result for [`use_virtual_source`].
///
/// The framework implements this sealed trait for [`Element`] and
/// [`ArkUIResult<OwnedNativeNode>`]. It exists so one virtual-list hook can select
/// the RSX or native `NodeBuilder` path from the callback's return type.
pub trait VirtualSourceItem: sealed::Sealed + 'static {
    #[doc(hidden)]
    fn use_source(
        kind: VirtualKind,
        total_count: u32,
        render_item: Rc<dyn Fn(u32) -> Self>,
    ) -> VirtualSource;
}

impl sealed::Sealed for Element {}

impl VirtualSourceItem for Element {
    fn use_source(
        kind: VirtualKind,
        total_count: u32,
        render_item: Rc<dyn Fn(u32) -> Self>,
    ) -> VirtualSource {
        let runtime = arkit_runtime::use_runtime_handle();
        let mount_item = rsx_mount_item(render_item, runtime);
        let initial_mount_item = mount_item.clone();
        let adapter =
            use_hook(move || VirtualSource::new_mounted(kind, total_count, initial_mount_item));

        adapter.set_mount_item(mount_item);
        adapter
    }
}

impl sealed::Sealed for ArkUIResult<OwnedNativeNode> {}

impl VirtualSourceItem for ArkUIResult<OwnedNativeNode> {
    fn use_source(
        kind: VirtualKind,
        total_count: u32,
        render_item: Rc<dyn Fn(u32) -> Self>,
    ) -> VirtualSource {
        let render_item: RenderItem = render_item;
        let initial_render_item = render_item.clone();
        let adapter = use_hook(move || VirtualSource::new(kind, total_count, initial_render_item));

        // The adapter outlives an individual component render, so always
        // replace its callback with the latest closure. This is Rust-owned
        // state only and cannot re-enter the native event receiver.
        adapter.set_render_item(render_item);
        adapter
    }
}

#[derive(Clone, arkit_prelude::Props)]
struct VirtualRsxItemProps {
    index: Rc<Cell<u32>>,
    render_item: SharedRsxRenderItem,
}

impl PartialEq for VirtualRsxItemProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.index, &other.index) && Rc::ptr_eq(&self.render_item, &other.render_item)
    }
}

fn virtual_rsx_item_root(props: VirtualRsxItemProps) -> Element {
    // Each embedded item tree installs its own reactive metrics/lifecycle
    // signal providers, fed by the handles forwarded from the host tree in
    // `rsx_mount_item`. Window/keyboard changes reach item components through
    // precise signal subscriptions — there is no whole-tree invalidation.
    crate::use_runtime_context_providers();
    let render_item = props.render_item.borrow().clone();
    render_item(props.index.get())
}

/// Create a true virtual List, Grid, or WaterFlow.
///
/// The callback can return either an RSX [`Element`] or an
/// [`ArkUIResult<OwnedNativeNode>`]. Each visible RSX item owns a small Dioxus subtree
/// mounted directly into the adapter-created ListItem/GridItem/FlowItem
/// wrapper. Native results use the same adapter lifecycle without an embedded
/// Dioxus runtime.
///
/// ```ignore
/// let source = use_virtual_source(
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
///
/// The same hook accepts native items without a mode flag:
///
/// ```ignore
/// let source = use_virtual_source(VirtualKind::List, 10_000, move |index| {
///     Ok(NodeBuilder::new("text")?
///         .text_content(format!("Item {index}"))?
///         .build())
/// });
/// ```
#[track_caller]
pub fn use_virtual_source<I>(
    kind: VirtualKind,
    total_count: u32,
    render_item: impl Fn(u32) -> I + 'static,
) -> VirtualSource
where
    I: VirtualSourceItem,
{
    let source = I::use_source(kind, total_count, Rc::new(render_item));
    use_virtual_source_count(source.clone(), total_count);
    source
}

struct VirtualRsxItemOwner {
    index: Rc<Cell<u32>>,
    runtime: arkit_runtime::EmbeddedArkRuntime,
}

fn rsx_mount_item(
    render_item: RsxRenderItem,
    runtime_handle: arkit_runtime::RuntimeHandle,
) -> MountItem {
    let initial_render_item = render_item.clone();
    let shared_render_item = use_hook(move || Rc::new(RefCell::new(initial_render_item)));
    *shared_render_item.borrow_mut() = render_item;
    let window_metrics = dioxus_core::try_consume_context::<arkit_runtime::WindowMetricsHandle>();
    let application_lifecycle =
        dioxus_core::try_consume_context::<arkit_runtime::ApplicationLifecycleHandle>();
    let safe_area_policy = dioxus_core::try_consume_context::<arkit_runtime::SafeAreaPolicy>();

    Rc::new(move |index, wrapper| {
        let item_index = Rc::new(Cell::new(index));
        let dom = arkit_runtime::VirtualDom::new_with_props(
            virtual_rsx_item_root,
            VirtualRsxItemProps {
                index: item_index.clone(),
                render_item: shared_render_item.clone(),
            },
        );
        if let Some(window_metrics) = &window_metrics {
            dom.provide_root_context(window_metrics.clone());
        }
        if let Some(application_lifecycle) = &application_lifecycle {
            dom.provide_root_context(application_lifecycle.clone());
        }
        if let Some(safe_area_policy) = safe_area_policy {
            dom.provide_root_context(safe_area_policy);
        }

        let runtime =
            arkit_runtime::mount_embedded_virtual_dom(wrapper, dom, runtime_handle.clone());
        Ok(VirtualItemMount::retain_indexed_with_abandon(
            VirtualRsxItemOwner {
                index: item_index,
                runtime,
            },
            |owner, index| {
                owner.index.set(index);
                owner.runtime.rerender();
            },
            |owner| owner.runtime.abandon(),
        ))
    })
}

fn use_virtual_source_count(source: VirtualSource, total_count: u32) {
    // Count changes mutate ArkUI and may synchronously emit adapter events.
    // Defer them until after Dioxus commits the render that supplied the new
    // callback and backing data. Grow/shrink is applied as a tail insert or
    // remove so existing rows (and their scroll anchor) are never rebuilt.
    let previous = use_hook(|| std::cell::Cell::new(total_count));
    use_effect(use_reactive((&total_count,), move |(next_total,)| {
        let previous_total = previous.replace(next_total);
        if next_total == previous_total {
            return;
        }
        let result = if next_total > previous_total {
            source.insert_items(previous_total, next_total - previous_total)
        } else {
            source.remove_items(next_total, previous_total - next_total)
        };
        if let Err(error) = result {
            ohos_hilog_binding::error(format!(
                "arkit_hooks: virtual adapter count update failed: {error}"
            ));
        }
    }));
}

/// Create a virtual adapter with item-local invalidation.
///
/// `item_keys[index]` must cover every visual input for that item. Equal-size
/// updates reload only the changed contiguous runs. Unique keys also allow
/// structural inserts, removals, and moves to preserve unaffected native rows
/// and their item-local state. Keeping distant changes separate is important
/// for selection updates: reloading the entire range between the previous and
/// next selection can disturb a List's scroll anchor.
#[track_caller]
pub fn use_virtual_source_items_keyed<K, I>(
    kind: VirtualKind,
    item_keys: Vec<K>,
    render_item: impl Fn(u32) -> I + 'static,
) -> VirtualSource
where
    K: Clone + Eq + Hash + 'static,
    I: VirtualSourceItem,
{
    let total_count = item_keys.len() as u32;
    let source = I::use_source(kind, total_count, Rc::new(render_item));
    use_virtual_item_keys(source.clone(), item_keys);
    source
}

fn use_virtual_item_keys<K>(source: VirtualSource, item_keys: Vec<K>)
where
    K: Clone + Eq + Hash + 'static,
{
    let previous_item_keys = use_hook(|| Rc::new(RefCell::new(item_keys.clone())));
    let effect_previous_item_keys = previous_item_keys.clone();

    use_effect(use_reactive((&item_keys,), move |(next_item_keys,)| {
        let previous_item_keys = effect_previous_item_keys.borrow().clone();

        let updates = keyed_item_updates(&previous_item_keys, &next_item_keys);
        if updates.is_empty() {
            return;
        }
        for update in updates {
            let result = match update {
                KeyedItemUpdate::Insert { start, count } => source.insert_items(start, count),
                KeyedItemUpdate::Remove { start, count } => source.remove_items(start, count),
                KeyedItemUpdate::Move { from, to } => source.move_item(from, to),
                KeyedItemUpdate::Reload { start, count } => source.reload_items(start, count),
                KeyedItemUpdate::Reset => reset_virtual_items(&source, next_item_keys.len()),
            };
            if let Err(error) = result {
                ohos_hilog_binding::error(format!(
                    "arkit_hooks: item-keyed virtual adapter update failed: {error}"
                ));
                if let Err(reset_error) = reset_virtual_items(&source, next_item_keys.len()) {
                    ohos_hilog_binding::error(format!(
                        "arkit_hooks: virtual adapter recovery failed: {reset_error}"
                    ));
                }
                break;
            }
        }
        *effect_previous_item_keys.borrow_mut() = next_item_keys;
    }));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyedItemUpdate {
    Insert { start: u32, count: u32 },
    Remove { start: u32, count: u32 },
    Move { from: u32, to: u32 },
    Reload { start: u32, count: u32 },
    Reset,
}

fn keyed_item_updates<K>(previous: &[K], next: &[K]) -> Vec<KeyedItemUpdate>
where
    K: Clone + Eq + Hash,
{
    if previous == next {
        return Vec::new();
    }
    if !keys_are_unique(previous) || !keys_are_unique(next) {
        if previous.len() == next.len() {
            return changed_item_ranges(previous, next)
                .into_iter()
                .map(|(start, count)| KeyedItemUpdate::Reload { start, count })
                .collect();
        }
        return vec![KeyedItemUpdate::Reset];
    }

    let next_keys = next.iter().collect::<HashSet<_>>();
    let mut current = previous.to_vec();
    let mut updates = Vec::new();

    let mut absent_ranges = Vec::new();
    let mut range_start = None;
    for (index, key) in current.iter().enumerate() {
        if !next_keys.contains(key) {
            range_start.get_or_insert(index);
        } else if let Some(start) = range_start.take() {
            absent_ranges.push((start, index - start));
        }
    }
    if let Some(start) = range_start {
        absent_ranges.push((start, current.len() - start));
    }
    for (start, count) in absent_ranges.into_iter().rev() {
        current.drain(start..start + count);
        updates.push(KeyedItemUpdate::Remove {
            start: start as u32,
            count: count as u32,
        });
    }

    let mut target = 0;
    while target < next.len() {
        if current.get(target) == Some(&next[target]) {
            target += 1;
            continue;
        }
        if let Some(offset) = current[target..]
            .iter()
            .position(|key| key == &next[target])
        {
            let from = target + offset;
            let key = current.remove(from);
            current.insert(target, key);
            updates.push(KeyedItemUpdate::Move {
                from: from as u32,
                to: target as u32,
            });
            target += 1;
            continue;
        }

        let start = target;
        while target < next.len() && !current[target.min(current.len())..].contains(&next[target]) {
            current.insert(target, next[target].clone());
            target += 1;
        }
        updates.push(KeyedItemUpdate::Insert {
            start: start as u32,
            count: (target - start) as u32,
        });
    }

    if current.len() > next.len() {
        let start = next.len();
        let count = current.len() - start;
        current.truncate(start);
        updates.push(KeyedItemUpdate::Remove {
            start: start as u32,
            count: count as u32,
        });
    }
    debug_assert!(current == next);
    updates
}

fn keys_are_unique<K>(keys: &[K]) -> bool
where
    K: Eq + Hash,
{
    let mut unique = HashSet::with_capacity(keys.len());
    keys.iter().all(|key| unique.insert(key))
}

fn reset_virtual_items(source: &VirtualSource, next_len: usize) -> ArkUIResult<()> {
    let next_total = next_len as u32;
    if source.total_count() == next_total {
        source.reload_all_items()
    } else {
        source.set_total_count(next_total)?;
        source.reload_all_items()
    }
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
    use super::{changed_item_ranges, keyed_item_updates, KeyedItemUpdate};

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

    #[test]
    fn expanding_a_group_inserts_only_its_members() {
        assert_eq!(
            keyed_item_updates(
                &["section", "group-a", "group-b"],
                &["section", "group-a", "a-1", "a-2", "group-b"]
            ),
            vec![KeyedItemUpdate::Insert { start: 2, count: 2 }]
        );
    }

    #[test]
    fn collapsing_a_group_removes_only_its_members() {
        assert_eq!(
            keyed_item_updates(
                &["section", "group-a", "a-1", "a-2", "group-b"],
                &["section", "group-a", "group-b"]
            ),
            vec![KeyedItemUpdate::Remove { start: 2, count: 2 }]
        );
    }

    #[test]
    fn changing_expanded_group_preserves_both_group_rows() {
        assert_eq!(
            keyed_item_updates(
                &["section", "group-a", "a-1", "a-2", "group-b", "group-c"],
                &["section", "group-a", "group-b", "b-1", "b-2", "group-c"],
            ),
            vec![
                KeyedItemUpdate::Remove { start: 2, count: 2 },
                KeyedItemUpdate::Insert { start: 3, count: 2 },
            ]
        );
    }

    #[test]
    fn reordering_unique_keys_moves_existing_rows() {
        assert_eq!(
            keyed_item_updates(&["a", "b", "c", "d"], &["a", "c", "d", "b"]),
            vec![
                KeyedItemUpdate::Move { from: 2, to: 1 },
                KeyedItemUpdate::Move { from: 3, to: 2 },
            ]
        );
    }

    #[test]
    fn duplicate_keys_keep_item_local_reload_semantics() {
        assert_eq!(
            keyed_item_updates(&[false, true, false], &[false, false, true]),
            vec![KeyedItemUpdate::Reload { start: 1, count: 2 }]
        );
    }
}
