#[cfg(not(feature = "api-22"))]
compile_error!("arkit requires feature `api-22` as the baseline");

mod component;
mod context;
mod effect;
mod lifecycle;
mod logging;
mod overlay;
mod owner;
mod portal;
mod route;
mod route_animated;
mod runtime;
mod signal;
mod view;

pub use arkit_derive::{component, entry};
pub use effect::{batch, create_effect, create_effect_on, untrack};
pub use lifecycle::LifecycleEvent;
pub use overlay::{
    anchored_overlay, native_overlay, observe_layout_frame, observe_layout_frame_enabled,
    observe_layout_size, LayoutFrame, LayoutSize, NativeOverlayPlacement,
};
pub use owner::{
    create_root, create_scope, enter_scope, on_cleanup, on_mount, provide_context, use_context,
    ScopeGuard,
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
pub use route::{
    register_fallback_route, use_route_param_signal, use_route_query_signal, use_search_param,
};
pub use route_animated::{animated_router_view, RouteTransitionConfig};
pub use runtime::{current_app, queue_after_mount, queue_ui_loop, Runtime};
pub use signal::{create_memo, create_memo_on, create_signal, signal, ReadSignal, Signal};
pub use view::ReactiveHost;
pub use view::*;

pub use napi_derive_ohos;
pub use napi_ohos;
pub use ohos_arkui_binding;
pub use openharmony_ability;

pub mod prelude {
    pub use crate::view::prelude::*;
    pub use crate::{
        anchored_overlay, animated_router_view, back_route, batch, component, create_effect,
        create_effect_on, create_memo, create_memo_on, create_root, create_scope, create_signal,
        current_app, current_route, entry, native_overlay, observe_layout_frame,
        observe_layout_frame_enabled, observe_layout_size, on_cleanup, on_mount, portal_scope,
        provide_context, push_route, register_fallback_route, register_named_route, register_route,
        register_routes, replace_route, reset_route, router, router_register_tree, router_view,
        set_router, signal, untrack, use_back_handler, use_before_leave, use_context, use_outlet,
        use_route, use_route_context, use_route_param, use_route_param_signal, use_route_query,
        use_route_query_signal, use_route_transition, use_router, use_search_param, LayoutFrame,
        LayoutSize, LifecycleEvent, NativeOverlayPlacement, ReactiveHost, ReadSignal,
        ResolvedRoute, Route, RouteContext, RouteDefinition, RouteNode, RouteSegmentMatch,
        RouteTransition, RouteTransitionConfig, RouteTransitionDirection, Router, Signal,
    };
}
