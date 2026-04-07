#[cfg(not(feature = "api-22"))]
compile_error!("arkit_runtime requires feature `api-22` as the baseline");

#[path = "../../arkit/src/component/mod.rs"]
mod component;
#[path = "../../arkit/src/context.rs"]
mod context;
#[path = "../../arkit/src/lifecycle.rs"]
mod lifecycle;
#[path = "../../arkit/src/logging.rs"]
mod logging;
#[path = "../../arkit/src/overlay.rs"]
mod overlay;
#[path = "../../arkit/src/owner.rs"]
mod owner;
#[path = "../../arkit/src/portal.rs"]
mod portal;
#[path = "../../arkit/src/route.rs"]
mod route;
#[path = "../../arkit/src/route_animated.rs"]
mod route_animated;
#[path = "../../arkit/src/runtime.rs"]
mod runtime;
#[path = "../../arkit/src/view/mod.rs"]
mod view;

mod application;
mod application_runtime;
mod entry;
mod widget_api;

pub use application::{
    application, dispatch, set_dispatcher, with_dispatcher, Application, Program, Renderer,
    RuntimeDispatcher, Settings, Subscription, Task, Theme,
};
pub use application_runtime::{mount_application, ApplicationRuntime};
pub use arkit_derive::component;
pub use entry::{mount_entry, EntryHandle as MountedEntryHandle, EntryPoint};
pub use lifecycle::LifecycleEvent;
pub use overlay::{
    anchored_overlay, native_overlay, observe_layout_frame, observe_layout_frame_enabled,
    observe_layout_size, LayoutFrame, LayoutSize, NativeOverlayPlacement,
};
pub use owner::{
    create_root, create_scope, enter_scope, on_cleanup, on_mount, provide_context, use_context,
    use_local_context, ScopeGuard,
};
pub use portal::portal_scope;
pub use route::{
    back_route, current_route, push_route, register_named_route, register_route, register_routes,
    replace_route, reset_route, router, router_register_tree, router_view, set_router,
    use_back_handler, use_before_leave, use_outlet, use_route, use_route_context, use_route_param,
    use_route_query, use_route_transition, use_router, ResolvedRoute, Route, RouteContext,
    RouteDefinition, RouteError, RouteNode, RouteSegmentMatch, RouteTransition,
    RouteTransitionDirection, Router,
};
pub use route::{register_fallback_route, use_search_param};
pub use route_animated::{animated_router_view, RouteTransitionConfig};
pub use runtime::{current_app, current_runtime, queue_after_mount, queue_ui_loop, Runtime};
pub use view::prelude::{
    ArkEvent, ArkUINodeAttributeItem, ArkUINodeAttributeType, NodeCustomEvent, NodeCustomEventType,
    NodeEventType,
};
pub use view::ReactiveHost;
pub use view::*;
pub use widget_api::{
    style, ButtonMessageExt, ButtonStyleExt, CheckboxMessageExt, CheckboxStyleExt,
    ContainerAlignmentExt, ContainerStyleExt, Horizontal, Length, Padding, TextAreaMessageExt,
    TextInputMessageExt, TextInputStyleExt, TextStyleExt, Vertical,
};

pub use napi_derive_ohos;
pub use napi_ohos;
pub use ohos_arkui_binding;
pub use openharmony_ability;

pub mod prelude {
    pub use crate::view::prelude::*;
    pub use crate::{
        anchored_overlay, animated_router_view, back_route, component, create_root, create_scope,
        current_app, current_route, native_overlay, observe_layout_frame,
        observe_layout_frame_enabled, observe_layout_size, on_cleanup, on_mount, portal_scope,
        provide_context, push_route, register_fallback_route, register_named_route, register_route,
        register_routes, replace_route, reset_route, router, router_register_tree, router_view,
        set_router, style, use_back_handler, use_before_leave, use_context, use_local_context,
        use_outlet, use_route, use_route_context, use_route_param, use_route_query,
        use_route_transition, use_router, use_search_param, ArkEvent, ArkUINodeAttributeItem, ArkUINodeAttributeType,
        ButtonMessageExt, ButtonStyleExt, CheckboxMessageExt, CheckboxStyleExt,
        ContainerAlignmentExt, ContainerStyleExt, Horizontal, LayoutFrame, LayoutSize, Length,
        LifecycleEvent, NativeOverlayPlacement, NodeCustomEvent, NodeCustomEventType,
        NodeEventType, Padding, ReactiveHost, ResolvedRoute, Route, RouteContext, RouteDefinition,
        RouteNode, RouteSegmentMatch, RouteTransition, RouteTransitionConfig,
        RouteTransitionDirection, Router, TextAreaMessageExt, TextInputMessageExt,
        TextInputStyleExt, TextStyleExt, Vertical,
    };
}
