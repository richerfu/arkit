use std::cell::RefCell;
use std::rc::Rc;

use arkit_router::{global_router, replace_global_router};
pub use arkit_router::{Route, RouteDefinition, RouteError, Router};

use crate::{use_component_lifecycle, use_signal};

pub fn router() -> Router {
    global_router()
}

pub fn set_router(next_router: Router) {
    replace_global_router(next_router);
}

pub fn register_route(pattern: impl Into<String>) -> Result<bool, RouteError> {
    router().register(pattern)
}

pub fn register_named_route(
    name: impl Into<String>,
    pattern: impl Into<String>,
) -> Result<bool, RouteError> {
    router().register_named(name, pattern)
}

pub fn register_routes<I>(definitions: I) -> Result<(), RouteError>
where
    I: IntoIterator<Item = RouteDefinition>,
{
    router().register_definitions(definitions)
}

pub fn push_route(path: impl Into<String>) -> Result<Route, RouteError> {
    router().push(path)
}

pub fn replace_route(path: impl Into<String>) -> Result<Route, RouteError> {
    router().replace(path)
}

pub fn reset_route(path: impl Into<String>) -> Result<Route, RouteError> {
    router().reset(path)
}

pub fn back_route() -> bool {
    router().back()
}

pub fn current_route() -> Route {
    router().current_route()
}

pub fn use_router() -> Router {
    router()
}

pub fn use_route() -> Route {
    let active_router = router();
    let route_state = use_signal({
        let active_router = active_router.clone();
        move || active_router.current_route()
    });

    let subscription_id = Rc::new(RefCell::new(None::<usize>));
    let mount_router = active_router.clone();
    let mount_route_state = route_state.clone();
    let mount_subscription_id = subscription_id.clone();
    let unmount_router = active_router.clone();

    use_component_lifecycle(
        move || {
            let signal = mount_route_state.clone();
            let id = mount_router.subscribe(move |route| {
                signal.set(route);
            });
            *mount_subscription_id.borrow_mut() = Some(id);
        },
        move || {
            if let Some(id) = subscription_id.borrow_mut().take() {
                unmount_router.unsubscribe(id);
            }
        },
    );

    route_state.get()
}

pub fn use_route_param(name: &str) -> Option<String> {
    use_route().param(name).map(ToOwned::to_owned)
}

pub fn use_route_query(name: &str) -> Option<String> {
    use_route().query(name).map(ToOwned::to_owned)
}
