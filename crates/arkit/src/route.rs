use std::cell::RefCell;
use std::rc::Rc;

use arkit_router::{global_router, replace_global_router};
pub use arkit_router::{
    ResolvedRoute, Route, RouteDefinition, RouteError, RouteNode, RouteSegmentMatch, Router,
};

use crate::owner::{provide_context, use_context};
use crate::view::keyed_scope;
use crate::view::Element;
use crate::{on_cleanup, queue_ui_loop};

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

pub fn register_fallback_route(pattern: impl Into<String>) -> Result<(), RouteError> {
    router().register_fallback(pattern)
}

pub fn current_route() -> Route {
    router().current_route()
}

pub fn use_router() -> Router {
    router()
}

pub fn use_route() -> Route {
    ensure_route_subscription().route.borrow().clone()
}

pub fn use_route_transition() -> RouteTransition {
    let active_router = router();
    if let Some(state) = use_context::<RouteTransitionState>() {
        return state.value.borrow().clone();
    }

    let initial_route = active_router.current_route();
    let initial_stack_len = active_router.stack_len();
    let state = RouteTransitionState {
        value: Rc::new(RefCell::new(RouteTransition::new(
            initial_route.clone(),
            RouteTransitionDirection::None,
        ))),
    };
    let previous_route = Rc::new(RefCell::new(initial_route));
    let previous_stack_len = Rc::new(RefCell::new(initial_stack_len));
    let subscription_id = Rc::new(RefCell::new(None::<usize>));

    {
        let router = active_router.clone();
        let value = state.value.clone();
        let sub_id = subscription_id.clone();
        let previous_route = previous_route.clone();
        let previous_stack_len = previous_stack_len.clone();
        let id = active_router.subscribe(move |route| {
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
            value.replace(RouteTransition::new(route, direction));
            request_router_rerender();
        });
        *sub_id.borrow_mut() = Some(id);
    }

    on_cleanup({
        let router = active_router.clone();
        let sub_id = subscription_id.clone();
        move || {
            if let Some(id) = sub_id.borrow_mut().take() {
                router.unsubscribe(id);
            }
        }
    });

    provide_context(state.clone());
    let transition = state.value.borrow().clone();
    transition
}

pub fn use_route_param(name: &str) -> Option<String> {
    use_route().param(name).map(ToOwned::to_owned)
}

pub fn use_route_query(name: &str) -> Option<String> {
    use_route().query(name).map(ToOwned::to_owned)
}

/// Returns the current query parameter and a setter that navigates to the
/// current path with the updated value.
pub fn use_search_param(name: &str) -> (Option<String>, Rc<dyn Fn(Option<String>)>) {
    let getter = use_route_query(name);
    let name = name.to_owned();
    let setter: Rc<dyn Fn(Option<String>)> = Rc::new(move |value| {
        let current = current_route();
        let mut query = current.query_params().clone();
        match value {
            Some(v) => {
                query.insert(name.clone(), v);
            }
            None => {
                query.remove(&name);
            }
        }
        let new_raw = arkit_router::join_raw_path(current.path(), &query);
        let _ = replace_route(&new_raw);
    });
    (getter, setter)
}

pub fn use_before_leave(guard: impl Fn(&str) -> Option<String> + 'static) {
    let active_router = router();
    let guard_id = active_router.add_guard(guard);
    on_cleanup(move || {
        active_router.remove_guard(guard_id);
    });
}

pub fn use_back_handler() {
    let app = crate::runtime::current_app();
    let Some(app) = app else {
        return;
    };
    let active_router = router();
    app.on_back_press_intercept(move || {
        if active_router.can_go_back() {
            active_router.back();
            true
        } else {
            false
        }
    });
}

#[derive(Clone)]
struct RouteState {
    route: Rc<RefCell<Route>>,
}

#[derive(Clone)]
struct RouteTransitionState {
    value: Rc<RefCell<RouteTransition>>,
}

fn ensure_route_subscription() -> RouteState {
    let active_router = router();
    // Route state is intentionally inherited by descendants so the whole route
    // subtree shares one subscription and a consistent current route snapshot.
    if let Some(state) = use_context::<RouteState>() {
        return state;
    }

    let state = RouteState {
        route: Rc::new(RefCell::new(active_router.current_route())),
    };
    let subscription_id = Rc::new(RefCell::new(None::<usize>));
    on_cleanup({
        let router = active_router.clone();
        let sub_id = subscription_id.clone();
        move || {
            if let Some(id) = sub_id.borrow_mut().take() {
                router.unsubscribe(id);
            }
        }
    });

    {
        let route_state = state.route.clone();
        let sub_id = subscription_id.clone();
        let id = active_router.subscribe(move |route| {
            route_state.replace(route);
            request_router_rerender();
        });
        *sub_id.borrow_mut() = Some(id);
    }

    provide_context(state.clone());
    state
}

fn request_router_rerender() {
    queue_ui_loop(|| {
        if let Some(runtime) = crate::runtime::current_runtime() {
            let _ = runtime.request_rerender();
        }
    });
}

// ---------------------------------------------------------------------------
// Nested routes / Layout + Outlet
// ---------------------------------------------------------------------------

/// Context provided by [`router_view`] to enable nested layout rendering.
#[derive(Clone)]
pub struct RouteContext {
    resolved: ResolvedRoute,
    depth: usize,
}

impl RouteContext {
    pub fn resolved(&self) -> &ResolvedRoute {
        &self.resolved
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}

/// Access the current [`RouteContext`] provided by [`router_view`].
pub fn use_route_context() -> Option<RouteContext> {
    use_context::<RouteContext>()
}

/// Register a nested route tree with the global router.
pub fn router_register_tree(root: RouteNode) -> Result<(), RouteError> {
    router().register_tree(root)
}

/// Render the nested outlet at `depth + 1`.
///
/// Returns `None` if there is no deeper segment to render.
pub fn use_outlet() -> Option<Element> {
    let ctx = use_route_context()?;
    let next_depth = ctx.depth + 1;
    if next_depth >= ctx.resolved.depth() {
        return None;
    }
    let resolved = ctx.resolved.clone();
    Some(keyed_scope(format!("outlet:{}", next_depth), move || {
        provide_context(RouteContext {
            resolved,
            depth: next_depth,
        });
        // The caller should use `use_outlet` inside their `router_view`
        // render function. This returns an empty placeholder; the actual
        // rendering is driven by the `router_view` callback.
        crate::view::column_component()
            .percent_width(1.0)
            .percent_height(1.0)
            .into()
    }))
}

/// Renders the current route as a nested layout using registered route trees.
///
/// The `render_fn` is called for each matched segment, receiving the segment
/// match and an outlet element for the next depth level (or `None` for leaf).
///
/// # Example
/// ```ignore
/// fn app() -> Element {
///     let tree = RouteNode::new("/")
///         .child(RouteNode::named("components", "/components/:name"));
///     router_register_tree(tree);
///     reset_route("/");
///
///     router_view(|segment, outlet| {
///         if segment.pattern() == "/" {
///             outlet.expect("root needs children")
///         } else {
///             column(vec![nav_bar("App", true), outlet.expect("leaf needs outlet")])
///         }
///     })
/// }
/// ```
pub fn router_view(
    render_fn: impl Fn(&RouteSegmentMatch, Option<Element>) -> Element + 'static + Clone,
) -> Element {
    let _ = use_route();
    let active_router = router();
    let resolved = match active_router.resolve_nested(active_router.current_route().raw()) {
        Ok(r) => r,
        Err(_) => {
            return crate::view::column_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .into();
        }
    };

    render_nested_layout(0, &resolved, Rc::new(render_fn))
}

fn render_nested_layout(
    depth: usize,
    resolved: &ResolvedRoute,
    render_fn: Rc<dyn Fn(&RouteSegmentMatch, Option<Element>) -> Element>,
) -> Element {
    let segment = match resolved.segments().get(depth) {
        Some(s) => s.clone(),
        None => {
            return crate::view::column_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .into();
        }
    };

    let outlet = if depth + 1 < resolved.depth() {
        let resolved_clone = resolved.clone();
        let render_fn_clone = render_fn.clone();
        Some(keyed_scope(format!("outlet:{}", depth + 1), move || {
            provide_context(RouteContext {
                resolved: resolved_clone.clone(),
                depth: depth + 1,
            });
            render_nested_layout(depth + 1, &resolved_clone, render_fn_clone)
        }))
    } else {
        None
    };

    keyed_scope(format!("route-segment:{}", depth), move || {
        render_fn(&segment, outlet)
    })
}
