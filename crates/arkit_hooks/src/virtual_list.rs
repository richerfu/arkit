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

use std::rc::Rc;

use arkit_prelude::use_hook;
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
