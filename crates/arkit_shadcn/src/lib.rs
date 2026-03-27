mod components;
mod styles;
pub mod theme;

pub use components::*;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::theme;

    pub use arkit::prelude::{
        ArkEvent, ArkUINodeAttributeItem, ArkUINodeAttributeType, NodeCustomEvent,
        NodeCustomEventType, NodeEventType,
    };
    pub use arkit::{
        back_route, component, create_signal, current_route, entry, on_cleanup, on_mount,
        push_route, register_named_route, register_route, register_routes, replace_route,
        reset_route, router, set_router, signal, use_context, use_route, use_route_param,
        use_route_query, use_router, Element, LifecycleEvent, Route, RouteDefinition, Router,
        Signal,
    };
}
