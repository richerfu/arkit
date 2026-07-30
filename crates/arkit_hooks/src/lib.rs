//! Hook-based escape-hatch APIs for the dioxus 0.7 ArkUI framework.
//!
//! This crate replaces the old command-oriented overlay/observer/virtual
//! container machinery with dioxus hooks. The hooks here are the bridge between
//! dioxus components and the raw ArkUI native nodes owned by [`arkit_arkui`]'s
//! renderer.
//!
//! ## Modules
//! - [`node`]: `use_ark_node` — get the native ArkUI node backing the current
//!   dioxus element, plus the `ArkHost` context that wires the renderer to
//!   hooks. Also `OverlayRoot` — render once at the app root to mount overlay
//!   content driven by `use_overlay`.
//! - [`layout`]: `use_layout_frame` / `use_layout_size` — observe the ArkUI
//!   layout of the current element via `onSizeChange`/`onAreaChange`.
//! - [`lifecycle`]: application foreground/background state and native
//!   component show/hide observation.
//! - [`overlay`]: `use_overlay` — floating and modal overlays. Content
//!   is rendered declaratively as a full-screen stack subtree at the app root
//!   via the host's overlay-content signal + `OverlayRoot`.
//! - [`virtual_range`]: `use_virtual_range` — the visible-item-range signal
//!   consumed by List/Grid/WaterFlow integrations.

mod layout;
mod lifecycle;
mod load_more;
mod node;
mod overlay;
mod safe_area;
mod virtual_list;
mod virtual_range;

pub use arkit_runtime::{
    ApplicationLifecycleEvent, ApplicationLifecycleHandle, ApplicationLifecyclePhase,
    ApplicationLifecycleState, ApplicationLifecycleSubscription, EdgeInsets, SafeAreaPolicy,
    WindowMetrics, WindowMetricsHandle, WindowMetricsSubscription,
};
pub use layout::{
    use_layout_frame, use_layout_frame_node, use_layout_size, LayoutFrame, LayoutSize,
};
pub use lifecycle::{
    use_app_foreground, use_application_lifecycle, use_application_lifecycle_event,
    use_component_lifecycle, use_component_visibility, ComponentLifecycleState,
};
pub use load_more::{use_load_more, LoadMoreController, LoadMoreState};
pub use node::{
    use_ark_host_provider, use_ark_node, ArkHost, ArkNodeRef, HitTestMode, HostNode, OverlayRoot,
};
pub use overlay::{
    use_overlay, ModalOverlaySpec, ModalPresentation, OverlayApi, OverlayLayer, OverlayViewport,
};
pub use safe_area::{
    use_safe_area, use_safe_area_policy, use_window_metrics, SafeArea, SafeAreaEdges, SafeAreaProps,
};
pub use virtual_list::{
    use_virtual_node_adapter, use_virtual_node_adapter_items_keyed, VirtualAdapterItem,
};
pub use virtual_range::{use_virtual_range, VirtualVisibleRange};
