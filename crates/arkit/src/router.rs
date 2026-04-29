pub use arkit_router::*;

use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

use crate::{advanced, BackPressDecision, Element, Renderer, Task, Theme};

type BoxRouteGuardFuture = Pin<Box<dyn Future<Output = RouteGuardDecision> + Send + 'static>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteStateKey {
    Raw,
    Path,
    Pattern,
}

impl Default for RouteStateKey {
    fn default() -> Self {
        Self::Raw
    }
}

#[derive(Clone)]
pub struct RouteContext {
    router: Router,
    route: Route,
}

impl RouteContext {
    pub fn new(router: Router, route: Route) -> Self {
        Self { router, route }
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub fn raw(&self) -> &Route {
        &self.route
    }

    pub fn path(&self) -> &str {
        self.route.path()
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.route.param(key)
    }

    pub fn parse_param<T>(&self, key: &str) -> Option<T>
    where
        T: FromStr,
    {
        self.param(key)?.parse::<T>().ok()
    }

    pub fn query(&self, key: &str) -> Option<&str> {
        self.route.query(key)
    }

    pub fn state<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.route.state::<T>()
    }

    pub fn state_cloned<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.route.state_cloned::<T>()
    }
}

pub struct Outlet<Message> {
    element: Element<Message>,
}

impl<Message> Outlet<Message> {
    fn new(element: Element<Message>) -> Self {
        Self { element }
    }

    pub fn into_element(self) -> Element<Message> {
        self.element
    }
}

impl<Message> From<Outlet<Message>> for Element<Message> {
    fn from(value: Outlet<Message>) -> Self {
        value.into_element()
    }
}

pub struct Routes<Message> {
    nodes: Vec<RouteNode<Message>>,
}

impl<Message: 'static> Routes<Message> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn index<F>(self, render: F) -> Self
    where
        F: Fn(RouteContext) -> Element<Message> + 'static,
    {
        self.route("", render)
    }

    pub fn route<F>(mut self, path: impl Into<String>, render: F) -> Self
    where
        F: Fn(RouteContext) -> Element<Message> + 'static,
    {
        self.nodes.push(RouteNode {
            path: path.into(),
            full_pattern: String::new(),
            kind: RouteNodeKind::Leaf(Box::new(render)),
            guards: Vec::new(),
            children: Vec::new(),
        });
        self
    }

    pub fn nest<F, C>(mut self, path: impl Into<String>, render: F, children: C) -> Self
    where
        F: Fn(RouteContext, Outlet<Message>) -> Element<Message> + 'static,
        C: FnOnce(Routes<Message>) -> Routes<Message>,
    {
        let children = children(Routes::new()).nodes;
        self.nodes.push(RouteNode {
            path: path.into(),
            full_pattern: String::new(),
            kind: RouteNodeKind::Layout(Box::new(render)),
            guards: Vec::new(),
            children,
        });
        self
    }

    pub fn guard<G, C>(mut self, guard: G, children: C) -> Self
    where
        G: Fn(RouteGuardContext) -> RouteGuardDecision + Send + Sync + 'static,
        C: FnOnce(Routes<Message>) -> Routes<Message>,
    {
        let children = children(Routes::new()).nodes;
        self.nodes.push(RouteNode {
            path: String::new(),
            full_pattern: String::new(),
            kind: RouteNodeKind::Scope,
            guards: vec![RouteGuardRegistration::Sync(Box::new(guard))],
            children,
        });
        self
    }

    pub fn guard_async<G, Fut, C>(mut self, guard: G, children: C) -> Self
    where
        G: Fn(RouteGuardContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = RouteGuardDecision> + Send + 'static,
        C: FnOnce(Routes<Message>) -> Routes<Message>,
    {
        let children = children(Routes::new()).nodes;
        self.nodes.push(RouteNode {
            path: String::new(),
            full_pattern: String::new(),
            kind: RouteNodeKind::Scope,
            guards: vec![RouteGuardRegistration::Async(Box::new(move |context| {
                Box::pin(guard(context))
            }))],
            children,
        });
        self
    }

    pub fn fallback<F>(mut self, path: impl Into<String>, render: F) -> Self
    where
        F: Fn(RouteContext) -> Element<Message> + 'static,
    {
        self.nodes.push(RouteNode {
            path: path.into(),
            full_pattern: String::new(),
            kind: RouteNodeKind::Fallback(Box::new(render)),
            guards: Vec::new(),
            children: Vec::new(),
        });
        self
    }
}

impl<Message: 'static> Default for Routes<Message> {
    fn default() -> Self {
        Self::new()
    }
}

struct RouteNode<Message> {
    path: String,
    full_pattern: String,
    kind: RouteNodeKind<Message>,
    guards: Vec<RouteGuardRegistration>,
    children: Vec<RouteNode<Message>>,
}

enum RouteNodeKind<Message> {
    Leaf(Box<dyn Fn(RouteContext) -> Element<Message>>),
    Fallback(Box<dyn Fn(RouteContext) -> Element<Message>>),
    Layout(Box<dyn Fn(RouteContext, Outlet<Message>) -> Element<Message>>),
    Scope,
}

enum RouteGuardRegistration {
    Sync(Box<dyn Fn(RouteGuardContext) -> RouteGuardDecision + Send + Sync>),
    Async(Box<dyn Fn(RouteGuardContext) -> BoxRouteGuardFuture + Send + Sync>),
}

enum RouteRegistration {
    Definition(RouteDefinition),
    Fallback(RouteDefinition),
}

pub struct RouterOutlet<Message> {
    router: Router,
    routes: Vec<RouteNode<Message>>,
    key: RouteStateKey,
}

struct RouteOutletPage<Message> {
    key: String,
    element: Element<Message>,
}

impl<Message: 'static> RouterOutlet<Message> {
    pub fn new(router: Router, routes: Routes<Message>) -> Self {
        let mut registrations = Vec::new();
        let routes = compile_nodes(routes.nodes, "/", &router, &[], &mut registrations);

        for registration in registrations {
            match registration {
                RouteRegistration::Definition(definition) => {
                    router
                        .register_definition(definition)
                        .expect("route registered by RouterOutlet");
                }
                RouteRegistration::Fallback(definition) => {
                    router
                        .register_fallback_definition(definition)
                        .expect("fallback route registered by RouterOutlet");
                }
            }
        }

        Self {
            router,
            routes,
            key: RouteStateKey::Raw,
        }
    }

    pub fn keyed(mut self, key: RouteStateKey) -> Self {
        self.key = key;
        self
    }
}

impl<Message: 'static> advanced::Widget<Message, Theme, Renderer> for RouterOutlet<Message> {
    fn state(&self) -> advanced::widget::State {
        advanced::widget::State::new(Box::new(RouterOutletState::default()))
    }

    fn body(
        &self,
        tree: &mut advanced::widget::Tree,
        _renderer: &Renderer,
    ) -> Option<Element<Message>> {
        tree.state()
            .get_or_insert_with(RouterOutletState::default)
            .ensure_subscription(&self.router);

        let route = self.router.current_route();
        let page = render_nodes(&self.routes, &self.router, &route, self.key)?;
        Some(keyed_element(page.key, page.element))
    }
}

impl<Message: 'static> From<RouterOutlet<Message>> for Element<Message> {
    fn from(value: RouterOutlet<Message>) -> Self {
        Element::new(value)
    }
}

#[derive(Default)]
struct RouterOutletState {
    subscription: Option<RouterOutletSubscription>,
}

impl RouterOutletState {
    fn ensure_subscription(&mut self, router: &Router) {
        let needs_subscription = self
            .subscription
            .as_ref()
            .is_none_or(|subscription| !subscription.router.ptr_eq(router));

        if !needs_subscription {
            return;
        }

        self.subscription = None;
        let id = router.subscribe(|_| {
            arkit_runtime::queue_ui_loop(|| {
                if let Some(runtime) = arkit_runtime::current_runtime() {
                    runtime.request_rerender();
                }
            });
        });
        self.subscription = Some(RouterOutletSubscription {
            router: router.clone(),
            id,
        });
    }
}

struct RouterOutletSubscription {
    router: Router,
    id: usize,
}

impl Drop for RouterOutletSubscription {
    fn drop(&mut self) {
        let _ = self.router.unsubscribe(self.id);
    }
}

fn compile_nodes<Message>(
    nodes: Vec<RouteNode<Message>>,
    parent_pattern: &str,
    router: &Router,
    active_guards: &[GuardRef],
    registrations: &mut Vec<RouteRegistration>,
) -> Vec<RouteNode<Message>> {
    nodes
        .into_iter()
        .map(|mut node| {
            let full_pattern = join_route_paths(parent_pattern, &node.path);
            let mut guard_chain = active_guards.to_vec();
            for guard in node.guards.drain(..) {
                match guard {
                    RouteGuardRegistration::Sync(guard) => {
                        guard_chain.push(GuardRef::Sync(router.add_route_guard(guard)));
                    }
                    RouteGuardRegistration::Async(guard) => {
                        guard_chain.push(GuardRef::Async(router.add_route_async_guard(guard)));
                    }
                }
            }

            node.children = compile_nodes(
                node.children,
                &full_pattern,
                router,
                &guard_chain,
                registrations,
            );
            node.full_pattern = full_pattern;

            if matches!(
                &node.kind,
                RouteNodeKind::Leaf(_) | RouteNodeKind::Fallback(_)
            ) {
                let definition = RouteDefinition::new(node.full_pattern.clone())
                    .expect("route registered by RouterOutlet")
                    .with_guard_chain(guard_chain);
                let registration = if matches!(&node.kind, RouteNodeKind::Fallback(_)) {
                    RouteRegistration::Fallback(definition)
                } else {
                    RouteRegistration::Definition(definition)
                };
                registrations.push(registration);
            }

            node
        })
        .collect()
}

fn render_nodes<Message: 'static>(
    nodes: &[RouteNode<Message>],
    router: &Router,
    route: &Route,
    key: RouteStateKey,
) -> Option<RouteOutletPage<Message>> {
    for node in nodes {
        if let Some(page) = render_node(node, router, route, key) {
            return Some(page);
        }
    }
    None
}

fn render_node<Message: 'static>(
    node: &RouteNode<Message>,
    router: &Router,
    route: &Route,
    key: RouteStateKey,
) -> Option<RouteOutletPage<Message>> {
    match &node.kind {
        RouteNodeKind::Leaf(render) | RouteNodeKind::Fallback(render) => {
            if route.pattern() != node.full_pattern {
                return None;
            }

            let context = RouteContext::new(router.clone(), route.clone());
            Some(RouteOutletPage {
                key: route_state_key(route, &node.full_pattern, key, true),
                element: render(context),
            })
        }
        RouteNodeKind::Layout(render) => {
            let child = render_nodes(&node.children, router, route, key)?;
            let child = keyed_element(child.key, child.element);
            let context = RouteContext::new(router.clone(), route.clone());
            Some(RouteOutletPage {
                key: route_state_key(route, &node.full_pattern, key, false),
                element: render(context, Outlet::new(child)),
            })
        }
        RouteNodeKind::Scope => render_nodes(&node.children, router, route, key),
    }
}

fn keyed_element<Message: 'static>(key: String, element: Element<Message>) -> Element<Message> {
    crate::stack_component::<Message, Theme>()
        .persistent_state_key(format!("route:{key}"))
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![element])
        .into()
}

fn route_state_key(route: &Route, pattern: &str, key: RouteStateKey, leaf: bool) -> String {
    match key {
        RouteStateKey::Raw => {
            if leaf {
                route.raw().to_string()
            } else {
                arkit_router::join_raw_path(&pattern_instance(pattern, route), route.query_params())
            }
        }
        RouteStateKey::Path => pattern_instance(pattern, route),
        RouteStateKey::Pattern => pattern.to_string(),
    }
}

fn pattern_instance(pattern: &str, route: &Route) -> String {
    let mut out = String::new();
    for segment in pattern.split('/').filter(|segment| !segment.is_empty()) {
        out.push('/');
        if let Some(param) = segment.strip_prefix(':') {
            out.push_str(route.param(param).unwrap_or(segment));
        } else if let Some(param) = segment.strip_prefix('*') {
            let param = if param.is_empty() { "wildcard" } else { param };
            out.push_str(route.param(param).unwrap_or_default());
        } else {
            out.push_str(segment);
        }
    }

    if out.is_empty() {
        "/".to_string()
    } else {
        out
    }
}

fn join_route_paths(parent: &str, child: &str) -> String {
    let child = child.trim();
    if child.is_empty() {
        return normalize_route_path(parent);
    }

    if child.starts_with('/') {
        return normalize_route_path(child);
    }

    let parent = normalize_route_path(parent);
    if parent == "/" {
        normalize_route_path(&format!("/{child}"))
    } else {
        normalize_route_path(&format!("{parent}/{child}"))
    }
}

fn normalize_route_path(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }

    let mut out = String::new();
    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        out.push('/');
        out.push_str(segment);
    }
    if out.is_empty() {
        "/".to_string()
    } else {
        out
    }
}

pub trait RouterNavigationExt {
    fn handle<Message>(
        &self,
        message: RouterMessage,
        map: impl FnOnce(RouterMessage) -> Message + Send + 'static,
    ) -> Task<Message>
    where
        Message: Send + 'static;

    fn handle_system_back<Message>(
        &self,
        map: impl FnOnce(RouterMessage) -> Message + Send + 'static,
    ) -> BackPressDecision<Message>
    where
        Message: Send + 'static;
}

impl RouterNavigationExt for Router {
    fn handle<Message>(
        &self,
        message: RouterMessage,
        map: impl FnOnce(RouterMessage) -> Message + Send + 'static,
    ) -> Task<Message>
    where
        Message: Send + 'static,
    {
        match message {
            RouterMessage::Navigate(navigation) => {
                let event = self.commit_navigation_sync(navigation.clone());
                if matches!(event.result, Err(RouteError::AsyncGuardRequired)) {
                    match self.begin_navigation(navigation.clone()) {
                        Ok(task) => Task::perform(task.run(), move |result| {
                            map(RouterMessage::Complete(result))
                        }),
                        Err(error) => Task::done(map(RouterMessage::Event(NavigationEvent::new(
                            navigation,
                            Err(error),
                        )))),
                    }
                } else {
                    Task::done(map(RouterMessage::Event(event)))
                }
            }
            RouterMessage::Complete(result) => {
                Task::done(map(RouterMessage::Event(self.complete_navigation(result))))
            }
            RouterMessage::Event(_) => Task::none(),
        }
    }

    fn handle_system_back<Message>(
        &self,
        map: impl FnOnce(RouterMessage) -> Message + Send + 'static,
    ) -> BackPressDecision<Message>
    where
        Message: Send + 'static,
    {
        if self.can_go_back() {
            BackPressDecision::task(self.handle(RouterMessage::back(), map))
        } else {
            BackPressDecision::pass_through()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_page(label: &'static str) -> impl Fn(RouteContext) -> Element<()> {
        move |_| crate::text(label).into()
    }

    #[test]
    fn outlet_renders_matching_route() {
        let router = Router::new("/");
        let outlet = RouterOutlet::new(router, Routes::new().route("/", text_page("home")));
        let mut tree = advanced::tree_of::<(), Theme, Renderer>(&Element::new(outlet));
        let outlet = RouterOutlet::new(
            Router::new("/"),
            Routes::new().route("/", text_page("home")),
        );

        assert!(advanced::Widget::body(&outlet, &mut tree, &Renderer).is_some());
    }

    #[test]
    fn outlet_fallback_does_not_shadow_root_route() {
        let router = Router::new("/");
        let _outlet = RouterOutlet::new(
            router.clone(),
            Routes::new()
                .route("/", text_page("home"))
                .fallback("*rest", text_page("not-found")),
        );

        assert_eq!(router.current_route().pattern(), "/");

        let unknown = router.resolve("/missing/page").expect("fallback route");
        assert_eq!(unknown.pattern(), "/*rest");
        assert_eq!(unknown.param("rest"), Some("missing/page"));
    }

    #[test]
    fn nested_outlet_renders_child_branch() {
        let router = Router::new("/users/7/settings");
        let routes = Routes::new().nest(
            "/users/:id",
            |context, outlet| {
                assert_eq!(context.param("id"), Some("7"));
                crate::column(vec![outlet.into()]).into()
            },
            |users| users.route("settings", text_page("settings")),
        );
        let outlet = RouterOutlet::new(router, routes);
        let mut tree = advanced::tree_of::<(), Theme, Renderer>(&Element::new(RouterOutlet::new(
            Router::new("/users/7/settings"),
            Routes::new().route("/users/:id/settings", text_page("settings")),
        )));

        assert!(advanced::Widget::body(&outlet, &mut tree, &Renderer).is_some());
    }

    #[test]
    fn relative_and_absolute_child_paths_are_normalized() {
        let routes = Routes::new().nest(
            "/users/:id",
            |_, outlet| outlet.into(),
            |users| {
                users
                    .route("settings", text_page("relative"))
                    .route("/settings", text_page("absolute"))
            },
        );

        let mut registrations = Vec::new();
        let router = Router::new("/");
        let nodes = compile_nodes(routes.nodes, "/", &router, &[], &mut registrations);
        let patterns = registrations
            .iter()
            .map(|registration| match registration {
                RouteRegistration::Definition(definition)
                | RouteRegistration::Fallback(definition) => definition.pattern().to_string(),
            })
            .collect::<Vec<_>>();
        assert_eq!(patterns, vec!["/users/:id/settings", "/settings"]);
        assert_eq!(nodes[0].children[0].full_pattern, "/users/:id/settings");
        assert_eq!(nodes[0].children[1].full_pattern, "/settings");
    }

    #[test]
    fn route_state_key_modes_are_stable() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/users/:id").expect("route"))
            .expect("register route");
        let route = router
            .resolve("/users/7?tab=profile")
            .expect("resolve route");

        assert_eq!(
            route_state_key(&route, "/users/:id", RouteStateKey::Raw, true),
            "/users/7?tab=profile"
        );
        assert_eq!(
            route_state_key(&route, "/users/:id", RouteStateKey::Path, true),
            "/users/7"
        );
        assert_eq!(
            route_state_key(&route, "/users/:id", RouteStateKey::Pattern, true),
            "/users/:id"
        );
    }

    #[test]
    fn router_message_handle_commits_sync_navigation_and_emits_event() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/").expect("home"))
            .expect("register home");
        router
            .register_definition(RouteDefinition::new("/about").expect("about"))
            .expect("register about");

        let messages = router
            .handle(RouterMessage::push("/about"), |message| message)
            .into_messages();

        assert_eq!(router.current_path(), "/about");
        assert_eq!(messages.len(), 1);
        let RouterMessage::Event(event) = &messages[0] else {
            panic!("expected navigation event");
        };
        assert_eq!(event.result.as_ref().expect("route").path(), "/about");
    }

    #[test]
    fn system_back_passes_through_when_router_cannot_go_back() {
        let router = Router::new("/");

        let decision = router.handle_system_back(|message| message);

        assert!(!decision.is_intercepted());
    }

    #[test]
    fn system_back_dispatches_router_back_when_stack_can_go_back() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/").expect("home"))
            .expect("register home");
        router
            .register_definition(RouteDefinition::new("/about").expect("about"))
            .expect("register about");

        router
            .handle(RouterMessage::push("/about"), |message| message)
            .into_messages();

        let BackPressDecision::Intercept(task) = router.handle_system_back(|message| message)
        else {
            panic!("router should intercept system back when history is available");
        };

        let messages = task.into_messages();
        assert_eq!(router.current_path(), "/");
        let RouterMessage::Event(event) = &messages[0] else {
            panic!("expected navigation event");
        };
        assert!(matches!(event.navigation, Navigation::Back));
        assert_eq!(event.result.as_ref().expect("route").path(), "/");
    }

    #[test]
    fn system_back_supports_async_router_guards() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/").expect("home"))
            .expect("register home");
        router
            .register_definition(RouteDefinition::new("/about").expect("about"))
            .expect("register about");

        router
            .handle(RouterMessage::push("/about"), |message| message)
            .into_messages();
        router.add_async_guard(|_| async { RouteGuardDecision::Allow });

        let BackPressDecision::Intercept(task) = router.handle_system_back(|message| message)
        else {
            panic!("router should intercept async guarded back navigation");
        };

        let mut actions = task.into_actions();
        assert_eq!(actions.len(), 1);
        let arkit_runtime::TaskAction::Future(future) = actions.remove(0) else {
            panic!("async guarded back should run as a task future");
        };

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test tokio runtime");
        let RouterMessage::Complete(result) = runtime.block_on(future) else {
            panic!("async guarded back should complete navigation");
        };

        let messages = router
            .handle(RouterMessage::Complete(result), |message| message)
            .into_messages();
        assert_eq!(router.current_path(), "/");
        let RouterMessage::Event(event) = &messages[0] else {
            panic!("expected completed navigation event");
        };
        assert!(matches!(event.navigation, Navigation::Back));
        assert_eq!(event.result.as_ref().expect("route").path(), "/");
    }
}
