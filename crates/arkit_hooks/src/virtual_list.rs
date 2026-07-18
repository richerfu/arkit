//! Virtual List, Grid, and WaterFlow containers backed by ArkUI `NodeAdapter`.
//!
//! [`use_virtual_node_adapter`] creates a [`VirtualNodeAdapter`] that,
//! once the host `list`/`grid`/`waterflow` node is resolved (via
//! [`use_ark_node`]), attaches the matching ArkUI `NodeAdapter` so only visible
//! items are created on demand — true virtualization, not full instantiation.
//!
//! `render_item` is invoked by ArkUI on demand for each visible index and must
//! return a fresh [`ArkUINode`] for that item's content (the adapter wraps it
//! in a ListItem/GridItem/FlowItem automatically).

use std::rc::Rc;

use arkit_prelude::{use_effect, use_hook, use_reactive};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;

use arkit_arkui::{RenderItem, VirtualKind, VirtualNodeAdapter};

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
///     rsx! { list { percent_width: 1.0, percent_height: 1.0 } }
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

    // Count changes mutate ArkUI and may synchronously emit adapter events.
    // Defer them until after Dioxus commits the render that supplied the new
    // callback and backing data.
    let update_adapter = adapter.clone();
    use_effect(use_reactive((&total_count,), move |(next_total,)| {
        if let Err(error) = update_adapter.set_total_count(next_total) {
            ohos_hilog_binding::error(format!(
                "arkit_hooks: virtual adapter count update failed: {error}"
            ));
        }
    }));

    adapter
}
