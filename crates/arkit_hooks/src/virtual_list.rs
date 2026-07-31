//! Virtual List, Grid, and WaterFlow containers backed by ArkUI `NodeAdapter`.
//!
//! [`use_virtual_source`] accepts either an RSX [`Element`] or an
//! [`ArkUIResult<OwnedNativeNode>`] from its item callback. Assign the returned
//! source to the container's `virtual_source` attribute; the renderer owns
//! attachment and detachment. ArkUI then requests only visible items.

use std::cell::RefCell;
use std::rc::Rc;

use arkit_prelude::{dioxus_core, use_effect, use_hook, use_reactive, Element};
use ohos_arkui_binding::common::error::ArkUIResult;

use arkit_arkui::{
    MountItem, OwnedNativeNode, RenderItem, VirtualItemMount, VirtualKind, VirtualSource,
};

type RsxRenderItem = Rc<dyn Fn(u32) -> Element>;

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
        use_virtual_source_count(adapter.clone(), total_count);
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
        use_virtual_source_count(adapter.clone(), total_count);
        adapter
    }
}

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
    I::use_source(kind, total_count, Rc::new(render_item))
}

fn rsx_mount_item(
    render_item: RsxRenderItem,
    runtime_handle: arkit_runtime::RuntimeHandle,
) -> MountItem {
    let window_metrics = dioxus_core::try_consume_context::<arkit_runtime::WindowMetricsHandle>();
    let application_lifecycle =
        dioxus_core::try_consume_context::<arkit_runtime::ApplicationLifecycleHandle>();
    let safe_area_policy = dioxus_core::try_consume_context::<arkit_runtime::SafeAreaPolicy>();

    Rc::new(move |index, wrapper| {
        let dom = arkit_runtime::VirtualDom::new_with_props(
            virtual_rsx_item_root,
            VirtualRsxItemProps {
                index,
                render_item: render_item.clone(),
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
        Ok(VirtualItemMount::retain_with_abandon(runtime, |runtime| {
            runtime.abandon();
        }))
    })
}

fn use_virtual_source_count(source: VirtualSource, total_count: u32) {
    // Count changes mutate ArkUI and may synchronously emit adapter events.
    // Defer them until after Dioxus commits the render that supplied the new
    // callback and backing data.
    use_effect(use_reactive((&total_count,), move |(next_total,)| {
        if let Err(error) = source.set_total_count(next_total) {
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
/// handled by [`use_virtual_source`]. Keeping distant changes separate
/// is important for selection updates: reloading the entire range between the
/// previous and next selection can disturb a List's scroll anchor.
#[track_caller]
pub fn use_virtual_source_items_keyed<K, I>(
    kind: VirtualKind,
    item_keys: Vec<K>,
    render_item: impl Fn(u32) -> I + 'static,
) -> VirtualSource
where
    K: Clone + PartialEq + 'static,
    I: VirtualSourceItem,
{
    let total_count = item_keys.len() as u32;
    let source = use_virtual_source(kind, total_count, render_item);
    use_virtual_item_keys(source.clone(), item_keys);
    source
}

fn use_virtual_item_keys<K>(source: VirtualSource, item_keys: Vec<K>)
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
            if let Err(error) = source.reload_items(start, count) {
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
