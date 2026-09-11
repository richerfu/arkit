//! Virtual List, Grid, and WaterFlow containers backed by ArkUI `NodeAdapter`.
//!
//! [`use_virtual_source`] accepts either an RSX [`Element`] or an
//! [`ArkUIResult<OwnedNativeNode>`] from its item callback. Assign the returned
//! source to the container's `virtual_source` attribute; the renderer owns
//! attachment and detachment. ArkUI then requests only visible items.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;

use arkit_prelude::{dioxus_core, use_effect, use_hook, use_reactive, Element};
use dioxus_core::{AttributeValue, IntoAttributeValue};
use ohos_arkui_binding::common::error::ArkUIResult;

use arkit_arkui::{
    MountItem, OwnedNativeNode, RenderItem, VirtualItemMount, VirtualKind, VirtualSource,
};

use crate::virtual_diff::{
    validate_virtual_item_ids, virtual_item_updates, VirtualItemStamp, VirtualItemUpdate,
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
    let render_item = props.render_item.borrow().clone();
    render_item(props.index.get())
}

/// Create a manually controlled virtual List, Grid, or WaterFlow.
///
/// `initial_count` is used only to create the source. Later backing-data
/// changes must be paired explicitly with [`VirtualSource`] insert/remove/
/// move/reload operations. Use [`use_virtual_items`] when a stamp snapshot
/// should be the single authority for these mutations.
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
    initial_count: u32,
    render_item: impl Fn(u32) -> I + 'static,
) -> VirtualSource
where
    I: VirtualSourceItem,
{
    use_fixed_virtual_kind(kind, "use_virtual_source");
    I::use_source(kind, initial_count, Rc::new(render_item))
}

fn use_fixed_virtual_kind(kind: VirtualKind, hook: &'static str) {
    let initial = use_hook(|| Rc::new(Cell::new(kind)));
    assert_eq!(
        initial.get(),
        kind,
        "{hook} cannot change VirtualKind during one hook lifetime"
    );
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

/// A high-level virtual-items binding with data mutations owned by its hook.
///
/// Assign this value to a container's `virtual_source` attribute. It purposely
/// does not expose [`VirtualSource`]'s manual insert/remove/reload methods, so a
/// caller cannot mutate the adapter independently of the stamps passed to
/// [`use_virtual_items`].
#[derive(Clone, PartialEq, Eq)]
pub struct VirtualItems {
    source: VirtualSource,
}

impl IntoAttributeValue for VirtualItems {
    fn into_value(self) -> AttributeValue {
        self.source.into_value()
    }
}

/// Create a virtual adapter driven by stable identities and visual revisions.
///
/// `id` is the item's stable identity. Structural changes are derived solely
/// from IDs, preserving item-local state across moves while that item remains
/// retained by the adapter. `revision` covers every visual input captured by
/// `render_item`; changing it reloads that same item without changing its
/// identity. IDs must be unique within each snapshot; duplicates are rejected
/// as a programmer error before the adapter mutates.
#[track_caller]
pub fn use_virtual_items<Id, Revision, I>(
    kind: VirtualKind,
    stamps: Vec<VirtualItemStamp<Id, Revision>>,
    render_item: impl Fn(u32) -> I + 'static,
) -> VirtualItems
where
    Id: Clone + Eq + Hash + 'static,
    Revision: Clone + PartialEq + 'static,
    I: VirtualSourceItem,
{
    use_fixed_virtual_kind(kind, "use_virtual_items");
    if let Err(error) = validate_virtual_item_ids(&stamps) {
        panic!("use_virtual_items requires unique item IDs: {error}");
    }
    let token_state = use_hook(|| Rc::new(RefCell::new(StableItemTokens::new())));
    let item_ids = token_state.borrow_mut().resolve(&stamps);
    let total_count = u32::try_from(stamps.len()).expect("virtual item count exceeds u32");
    let source = I::use_source(kind, total_count, Rc::new(render_item));
    use_virtual_item_stamps(source.clone(), stamps, item_ids, token_state);
    VirtualItems { source }
}

struct StableItemTokens<Id> {
    current: HashMap<Id, u32>,
    next: u32,
}

impl<Id> StableItemTokens<Id>
where
    Id: Clone + Eq + Hash,
{
    fn new() -> Self {
        Self {
            current: HashMap::new(),
            next: 0,
        }
    }

    fn resolve<Revision>(&mut self, stamps: &[VirtualItemStamp<Id, Revision>]) -> Vec<u32> {
        let mut item_ids = Vec::with_capacity(stamps.len());
        for stamp in stamps {
            let token = self.current.get(&stamp.id).copied().unwrap_or_else(|| {
                let token = self.next;
                self.next = self
                    .next
                    .checked_add(1)
                    .expect("virtual item identity token space exhausted");
                self.current.insert(stamp.id.clone(), token);
                token
            });
            item_ids.push(token);
        }
        item_ids
    }

    fn commit<Revision>(&mut self, stamps: &[VirtualItemStamp<Id, Revision>]) {
        let retained = stamps.iter().map(|stamp| &stamp.id).collect::<HashSet<_>>();
        self.current.retain(|id, _| retained.contains(id));
    }
}

fn use_virtual_item_stamps<Id, Revision>(
    source: VirtualSource,
    stamps: Vec<VirtualItemStamp<Id, Revision>>,
    item_ids: Vec<u32>,
    token_state: Rc<RefCell<StableItemTokens<Id>>>,
) where
    Id: Clone + Eq + Hash + 'static,
    Revision: Clone + PartialEq + 'static,
{
    let initial_source = source.clone();
    let initial_stamps = stamps.clone();
    let initial_item_ids = item_ids.clone();
    let previous_stamps = use_hook(move || {
        initial_source.set_stable_item_ids(initial_item_ids);
        Rc::new(RefCell::new(initial_stamps))
    });
    let effect_previous_stamps = previous_stamps.clone();

    use_effect(use_reactive(
        (&stamps, &item_ids),
        move |(next_stamps, next_item_ids)| {
            let previous_stamps = effect_previous_stamps.borrow().clone();
            source.begin_stable_item_update(next_item_ids);

            let updates =
                virtual_item_updates(&previous_stamps, &next_stamps).unwrap_or_else(|error| {
                    panic!("use_virtual_items requires unique item IDs: {error}")
                });
            let mut revision_updates = Vec::new();
            let mut recovered_by_reset = false;
            let mut unrecoverable = None;
            for update in updates {
                let result = match update {
                    VirtualItemUpdate::Insert { start, count } => source.insert_items(start, count),
                    VirtualItemUpdate::Remove { start, count } => source.remove_items(start, count),
                    VirtualItemUpdate::Move { from, to } => source.move_item(from, to),
                    VirtualItemUpdate::Reload { .. } => {
                        revision_updates.push(update);
                        continue;
                    }
                };
                if let Err(error) = result {
                    ohos_hilog_binding::error(format!(
                        "arkit_hooks: virtual-items adapter update failed: {error}"
                    ));
                    match reset_virtual_items(&source, next_stamps.len()) {
                        Ok(()) => recovered_by_reset = true,
                        Err(reset_error) => {
                            unrecoverable = Some(format!(
                                "virtual adapter update failed ({error}) and recovery failed ({reset_error})"
                            ));
                        }
                    }
                    break;
                }
            }
            if let Err(error) = source.finish_stable_item_update() {
                unrecoverable = Some(format!(
                    "virtual adapter could not finish its identity update: {error}"
                ));
            }
            if unrecoverable.is_none() && !recovered_by_reset {
                for update in revision_updates {
                    let VirtualItemUpdate::Reload { start, count } = update else {
                        unreachable!("only reload updates are deferred")
                    };
                    if let Err(error) = source.reload_items_preserving_mounted_state(start, count) {
                        ohos_hilog_binding::error(format!(
                            "arkit_hooks: virtual item revision reload failed: {error}"
                        ));
                        match reset_virtual_items(&source, next_stamps.len()) {
                            Ok(()) => break,
                            Err(reset_error) => {
                                unrecoverable = Some(format!(
                                    "virtual item reload failed ({error}) and recovery failed ({reset_error})"
                                ));
                                break;
                            }
                        }
                    }
                }
            }
            if let Some(error) = unrecoverable {
                panic!("use_virtual_items entered an unrecoverable state: {error}");
            }
            token_state.borrow_mut().commit(&next_stamps);
            *effect_previous_stamps.borrow_mut() = next_stamps;
        },
    ));
}

fn reset_virtual_items(source: &VirtualSource, next_len: usize) -> ArkUIResult<()> {
    let next_total = u32::try_from(next_len).expect("virtual item count exceeds u32");
    if source.total_count() == next_total {
        source.reload_all_items()
    } else {
        source.set_total_count(next_total)?;
        source.reload_all_items()
    }
}
