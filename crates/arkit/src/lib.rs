#[cfg(not(feature = "api-22"))]
compile_error!("arkit requires feature `api-22` as the baseline");

mod component;
mod lifecycle;
mod logging;
mod route;
mod runtime;
mod signal;
mod view;

pub use arkit_derive::{component, entry};
pub use lifecycle::LifecycleEvent;
pub use route::{
    back_route, current_route, push_route, register_named_route, register_route, register_routes,
    replace_route, reset_route, router, set_router, use_route, use_route_param, use_route_query,
    use_router, Route, RouteDefinition, RouteError, Router,
};
pub use runtime::{current_app, queue_after_mount, queue_ui_loop, Runtime};
pub use signal::{signal, use_component_lifecycle, use_lifecycle, use_signal, Signal};
pub use view::*;

pub use napi_derive_ohos;
pub use napi_ohos;
pub use ohos_arkui_binding;
pub use openharmony_ability;

pub mod prelude {
    pub use crate::view::prelude::*;
    pub use crate::{
        back_route, component, current_app, current_route, entry, push_route, register_named_route,
        register_route, register_routes, replace_route, reset_route, router, set_router, signal,
        use_component_lifecycle, use_lifecycle, use_route, use_route_param, use_route_query,
        use_router, use_signal, LifecycleEvent, Route, RouteDefinition, Router, Signal,
    };
}
