//! Exact-element hooks and declarative root projection for Arkit.
//!
//! Native observation is opt-in and explicit: allocate a
//! [`NativeElementRef`], assign it to one RSX element's `native_ref` attribute,
//! and pass the same handle to layout, lifecycle, or advanced-node hooks.

mod layout;
mod lifecycle;
mod load_more;
mod node;
mod overlay;
mod safe_area;
mod virtual_list;
mod virtual_range;

/// Install reactive projections for runtime-owned lifecycle and window state.
///
/// Entry-point integrations call this once around the application component.
#[doc(hidden)]
pub fn use_runtime_context_providers() {
    lifecycle::use_application_lifecycle_provider();
    safe_area::use_window_metrics_provider();
}

pub use arkit_arkui::{
    MountedNodeLease, NativeElementEvent, NativeElementRef, NativeElementSubscription,
    NativeVisibility,
};
pub use arkit_runtime::{
    ApplicationLifecycleEvent, ApplicationLifecycleHandle, ApplicationLifecyclePhase,
    ApplicationLifecycleState, ApplicationLifecycleSubscription, EdgeInsets, SafeAreaPolicy,
    WindowMetrics, WindowMetricsHandle, WindowMetricsSubscription,
};
pub use layout::{use_layout_frame, use_layout_size, LayoutFrame, LayoutSize};
pub use lifecycle::{
    use_app_foreground, use_application_lifecycle, use_application_lifecycle_event,
    use_component_lifecycle, use_component_visibility, ComponentLifecycleState,
};
pub use load_more::{use_load_more, LoadMoreController, LoadMoreState};
pub use node::{use_mounted_node, use_native_element_ref};
pub use overlay::{
    use_overlay_viewport, ModalPortal, ModalPresentation, OverlayLayer, OverlayViewport, Portal,
};
pub use safe_area::{
    use_safe_area, use_safe_area_policy, use_window_metrics, SafeArea, SafeAreaEdges, SafeAreaProps,
};
pub use virtual_list::{use_virtual_source, use_virtual_source_items_keyed, VirtualSourceItem};
pub use virtual_range::{use_virtual_range, VirtualVisibleRange};
