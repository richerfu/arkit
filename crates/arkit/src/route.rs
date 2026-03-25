use std::cell::RefCell;
use std::rc::Rc;

use arkit_router::{global_router, replace_global_router};
pub use arkit_router::{Route, RouteDefinition, RouteError, Router};

use crate::{use_component_lifecycle, use_signal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteTransitionDirection {
    None,
    Forward,
    Backward,
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteTransition {
    route: Route,
    direction: RouteTransitionDirection,
}

impl RouteTransition {
    pub fn new(route: Route, direction: RouteTransitionDirection) -> Self {
        Self { route, direction }
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn direction(&self) -> RouteTransitionDirection {
        self.direction
    }
}

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

pub fn use_route_transition() -> RouteTransition {
    let active_router = router();
    let initial_route = active_router.current_route();
    let initial_stack_len = active_router.stack_len();
    let transition_state = use_signal({
        let initial_route = initial_route.clone();
        move || RouteTransition::new(initial_route, RouteTransitionDirection::None)
    });

    let subscription_id = Rc::new(RefCell::new(None::<usize>));
    let previous_route = Rc::new(RefCell::new(initial_route));
    let previous_stack_len = Rc::new(RefCell::new(initial_stack_len));
    let mount_router = active_router.clone();
    let mount_transition_state = transition_state.clone();
    let mount_subscription_id = subscription_id.clone();
    let mount_previous_route = previous_route.clone();
    let mount_previous_stack_len = previous_stack_len.clone();
    let unmount_router = active_router.clone();

    use_component_lifecycle(
        move || {
            let signal = mount_transition_state.clone();
            let router = mount_router.clone();
            let previous_route = mount_previous_route.clone();
            let previous_stack_len = mount_previous_stack_len.clone();
            let id = mount_router.subscribe(move |route| {
                let next_stack_len = router.stack_len();
                let prev_stack_len = *previous_stack_len.borrow();
                let prev_route = previous_route.borrow().clone();
                let direction = if next_stack_len > prev_stack_len {
                    RouteTransitionDirection::Forward
                } else if next_stack_len < prev_stack_len {
                    RouteTransitionDirection::Backward
                } else if route.raw() != prev_route.raw() {
                    RouteTransitionDirection::Replace
                } else {
                    RouteTransitionDirection::None
                };

                *previous_stack_len.borrow_mut() = next_stack_len;
                *previous_route.borrow_mut() = route.clone();
                signal.set(RouteTransition::new(route, direction));
            });
            *mount_subscription_id.borrow_mut() = Some(id);
        },
        move || {
            if let Some(id) = subscription_id.borrow_mut().take() {
                unmount_router.unsubscribe(id);
            }
        },
    );

    transition_state.get()
}

pub fn use_route_param(name: &str) -> Option<String> {
    use_route().param(name).map(ToOwned::to_owned)
}

pub fn use_route_query(name: &str) -> Option<String> {
    use_route().query(name).map(ToOwned::to_owned)
}
