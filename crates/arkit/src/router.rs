pub use arkit_router::*;

use crate::{advanced, Element, Renderer, Theme};

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
pub struct RouteContext<R>
where
    R: StructuredRoute,
{
    router: Router,
    route: Route,
    typed: R,
}

impl<R> RouteContext<R>
where
    R: StructuredRoute,
{
    pub fn new(router: Router, route: Route, typed: R) -> Self {
        Self {
            router,
            route,
            typed,
        }
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub fn raw(&self) -> &Route {
        &self.route
    }

    pub fn route(&self) -> &R {
        &self.typed
    }

    pub fn into_route(self) -> R {
        self.typed
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.route.param(key)
    }

    pub fn query(&self, key: &str) -> Option<&str> {
        self.route.query(key)
    }

    pub fn state<T: 'static>(&self) -> Option<&T> {
        self.route.state::<T>()
    }

    pub fn state_cloned<T: Clone + 'static>(&self) -> Option<T> {
        self.route.state_cloned::<T>()
    }
}

#[derive(Clone)]
pub struct FallbackRouteContext {
    router: Router,
    route: Route,
}

impl FallbackRouteContext {
    pub fn new(router: Router, route: Route) -> Self {
        Self { router, route }
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub fn raw(&self) -> &Route {
        &self.route
    }

    pub fn query(&self, key: &str) -> Option<&str> {
        self.route.query(key)
    }

    pub fn state<T: 'static>(&self) -> Option<&T> {
        self.route.state::<T>()
    }

    pub fn state_cloned<T: Clone + 'static>(&self) -> Option<T> {
        self.route.state_cloned::<T>()
    }
}

pub trait RoutePage<Message, R>: Sized
where
    R: StructuredRoute,
{
    fn from_route(context: RouteContext<R>) -> Self;
}

pub trait FallbackRoutePage<Message>: Sized {
    fn from_route(context: FallbackRouteContext) -> Self;
}

pub struct RouterOutlet<Message> {
    router: Router,
    routes: Vec<RouteBinding<Message>>,
    fallback: Option<FallbackBinding<Message>>,
}

struct RouteBinding<Message> {
    render: Box<dyn Fn(&Router, &Route) -> Option<RouteOutletPage<Message>>>,
}

struct FallbackBinding<Message> {
    render: Box<dyn Fn(&Router, &Route) -> RouteOutletPage<Message>>,
}

struct RouteOutletPage<Message> {
    key: String,
    element: Element<Message>,
}

impl<Message> RouterOutlet<Message> {
    pub fn new(router: Router) -> Self {
        Self {
            router,
            routes: Vec::new(),
            fallback: None,
        }
    }

    pub fn route<R, P>(self) -> Self
    where
        R: StructuredRoute + 'static,
        P: RoutePage<Message, R> + advanced::Widget<Message, Theme, Renderer> + 'static,
        Message: 'static,
    {
        self.route_keyed::<R, P>(RouteStateKey::Raw)
    }

    pub fn route_keyed<R, P>(mut self, key: RouteStateKey) -> Self
    where
        R: StructuredRoute + 'static,
        P: RoutePage<Message, R> + advanced::Widget<Message, Theme, Renderer> + 'static,
        Message: 'static,
    {
        self.router
            .register::<R>()
            .expect("route registered by RouterOutlet");
        self.routes.push(RouteBinding {
            render: Box::new(move |router, route| {
                let typed = R::from_route(route)?;
                let key = route_state_key::<R>(route, key);
                let context = RouteContext::new(router.clone(), route.clone(), typed);
                Some(RouteOutletPage {
                    key,
                    element: Element::new(P::from_route(context)),
                })
            }),
        });
        self
    }

    pub fn fallback<P>(mut self) -> Self
    where
        P: FallbackRoutePage<Message> + advanced::Widget<Message, Theme, Renderer> + 'static,
        Message: 'static,
    {
        self.fallback = Some(FallbackBinding {
            render: Box::new(move |router, route| RouteOutletPage {
                key: format!("fallback:{}", route.raw()),
                element: Element::new(P::from_route(FallbackRouteContext::new(
                    router.clone(),
                    route.clone(),
                ))),
            }),
        });
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
        let page = self
            .routes
            .iter()
            .find_map(|binding| (binding.render)(&self.router, &route))
            .or_else(|| {
                self.fallback
                    .as_ref()
                    .map(|binding| (binding.render)(&self.router, &route))
            })?;

        Some(
            crate::stack_component::<Message, Theme>()
                .persistent_state_key(format!("route:{}", page.key))
                .percent_width(1.0)
                .percent_height(1.0)
                .children(vec![page.element])
                .into(),
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
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

fn route_state_key<R>(route: &Route, key: RouteStateKey) -> String
where
    R: StructuredRoute + 'static,
{
    match key {
        RouteStateKey::Raw => route.raw().to_string(),
        RouteStateKey::Path => route.path().to_string(),
        RouteStateKey::Pattern => {
            format!("{}:{}", std::any::type_name::<R>(), route.pattern())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct HomeRoute;

    impl StructuredRoute for HomeRoute {
        fn definition() -> RouteDefinition {
            RouteDefinition::named("home", "/").expect("home route")
        }

        fn path(&self) -> String {
            "/".to_string()
        }

        fn from_route(route: &Route) -> Option<Self> {
            (route.name() == Some("home")).then_some(Self)
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct UserRoute {
        id: String,
    }

    impl StructuredRoute for UserRoute {
        fn definition() -> RouteDefinition {
            RouteDefinition::named("user", "/users/:id").expect("user route")
        }

        fn path(&self) -> String {
            format!("/users/{}", self.id)
        }

        fn from_route(route: &Route) -> Option<Self> {
            if route.name() != Some("user") {
                return None;
            }
            Some(Self {
                id: route.param("id")?.to_string(),
            })
        }
    }

    struct HomePage;

    impl RoutePage<(), HomeRoute> for HomePage {
        fn from_route(_context: RouteContext<HomeRoute>) -> Self {
            Self
        }
    }

    impl advanced::Widget<(), Theme, Renderer> for HomePage {
        fn body(
            &self,
            _tree: &mut advanced::widget::Tree,
            _renderer: &Renderer,
        ) -> Option<Element<()>> {
            Some(crate::text("home").into())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
            self
        }
    }

    struct FallbackPage;

    impl FallbackRoutePage<()> for FallbackPage {
        fn from_route(_context: FallbackRouteContext) -> Self {
            Self
        }
    }

    impl advanced::Widget<(), Theme, Renderer> for FallbackPage {
        fn body(
            &self,
            _tree: &mut advanced::widget::Tree,
            _renderer: &Renderer,
        ) -> Option<Element<()>> {
            Some(crate::text("fallback").into())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
            self
        }
    }

    #[test]
    fn outlet_renders_matching_route() {
        let router = Router::new("/");
        let outlet = RouterOutlet::new(router).route::<HomeRoute, HomePage>();
        let mut tree = advanced::tree_of::<(), Theme, Renderer>(&Element::new(outlet));
        let outlet = RouterOutlet::new(Router::new("/")).route::<HomeRoute, HomePage>();

        assert!(advanced::Widget::body(&outlet, &mut tree, &Renderer).is_some());
    }

    #[test]
    fn outlet_uses_fallback_for_unmatched_route() {
        let router = Router::new("/missing");
        let outlet = RouterOutlet::new(router).fallback::<FallbackPage>();
        let mut tree = advanced::tree_of::<(), Theme, Renderer>(&Element::new(
            RouterOutlet::new(Router::new("/missing")).fallback::<FallbackPage>(),
        ));

        assert!(advanced::Widget::body(&outlet, &mut tree, &Renderer).is_some());
    }

    #[test]
    fn route_state_key_modes_are_stable() {
        let router = Router::new("/");
        router.register::<UserRoute>().expect("register user");
        let route = router
            .resolve("/users/7?tab=profile")
            .expect("resolve route");

        assert_eq!(
            route_state_key::<UserRoute>(&route, RouteStateKey::Raw),
            "/users/7?tab=profile"
        );
        assert_eq!(
            route_state_key::<UserRoute>(&route, RouteStateKey::Path),
            "/users/7"
        );
        assert!(
            route_state_key::<UserRoute>(&route, RouteStateKey::Pattern).ends_with(":/users/:id")
        );
    }
}
