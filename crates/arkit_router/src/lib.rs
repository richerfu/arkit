use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

type BoxGuardFuture = Pin<Box<dyn Future<Output = RouteGuardDecision> + Send + 'static>>;
type SyncGuard = Arc<dyn Fn(RouteGuardContext) -> RouteGuardDecision + Send + Sync>;
type AsyncGuard = Arc<dyn Fn(RouteGuardContext) -> BoxGuardFuture + Send + Sync>;
const MAX_GUARD_REDIRECTS: usize = 16;

/// Describes the route transition event with from/to routes and direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteTransitionEvent {
    from: Route,
    to: Route,
    direction: RouteTransitionDirection,
}

/// Direction of a route transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteTransitionDirection {
    None,
    Forward,
    Backward,
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteGuardContext {
    pub from: Route,
    pub to: Route,
    pub direction: RouteTransitionDirection,
}

impl RouteGuardContext {
    pub fn new(from: Route, to: Route, direction: RouteTransitionDirection) -> Self {
        Self {
            from,
            to,
            direction,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteGuardDecision {
    Allow,
    Block(String),
    Redirect(String),
}

pub type RouteGuardResult = Result<RouteGuardDecision, RouteError>;

#[derive(Debug, Clone)]
pub struct RouteTarget {
    raw_path: String,
    state: RouteState,
}

impl RouteTarget {
    pub fn new(raw_path: impl Into<String>) -> Self {
        Self {
            raw_path: raw_path.into(),
            state: RouteState::empty(),
        }
    }

    pub fn with_state<S: Send + Sync + 'static>(raw_path: impl Into<String>, state: S) -> Self {
        Self {
            raw_path: raw_path.into(),
            state: RouteState::new(state),
        }
    }

    pub fn raw_path(&self) -> &str {
        &self.raw_path
    }
}

#[derive(Debug, Clone)]
pub enum Navigation {
    Push(RouteTarget),
    Replace(RouteTarget),
    Reset(RouteTarget),
    Back,
}

impl Navigation {
    pub fn push(raw_path: impl Into<String>) -> Self {
        Self::Push(RouteTarget::new(raw_path))
    }

    pub fn push_with_state<S: Send + Sync + 'static>(
        raw_path: impl Into<String>,
        state: S,
    ) -> Self {
        Self::Push(RouteTarget::with_state(raw_path, state))
    }

    pub fn replace(raw_path: impl Into<String>) -> Self {
        Self::Replace(RouteTarget::new(raw_path))
    }

    pub fn replace_with_state<S: Send + Sync + 'static>(
        raw_path: impl Into<String>,
        state: S,
    ) -> Self {
        Self::Replace(RouteTarget::with_state(raw_path, state))
    }

    pub fn reset(raw_path: impl Into<String>) -> Self {
        Self::Reset(RouteTarget::new(raw_path))
    }

    pub fn reset_with_state<S: Send + Sync + 'static>(
        raw_path: impl Into<String>,
        state: S,
    ) -> Self {
        Self::Reset(RouteTarget::with_state(raw_path, state))
    }

    pub fn back() -> Self {
        Self::Back
    }
}

#[derive(Debug, Clone)]
pub struct NavigationEvent {
    pub navigation: Navigation,
    pub result: Result<Route, RouteError>,
}

impl NavigationEvent {
    pub fn new(navigation: Navigation, result: Result<Route, RouteError>) -> Self {
        Self { navigation, result }
    }
}

#[derive(Debug, Clone)]
pub enum RouterMessage {
    Navigate(Navigation),
    #[doc(hidden)]
    Complete(RouteNavigationResult),
    Event(NavigationEvent),
}

impl RouterMessage {
    pub fn push(raw_path: impl Into<String>) -> Self {
        Self::Navigate(Navigation::push(raw_path))
    }

    pub fn push_with_state<S: Send + Sync + 'static>(
        raw_path: impl Into<String>,
        state: S,
    ) -> Self {
        Self::Navigate(Navigation::push_with_state(raw_path, state))
    }

    pub fn replace(raw_path: impl Into<String>) -> Self {
        Self::Navigate(Navigation::replace(raw_path))
    }

    pub fn replace_with_state<S: Send + Sync + 'static>(
        raw_path: impl Into<String>,
        state: S,
    ) -> Self {
        Self::Navigate(Navigation::replace_with_state(raw_path, state))
    }

    pub fn reset(raw_path: impl Into<String>) -> Self {
        Self::Navigate(Navigation::reset(raw_path))
    }

    pub fn reset_with_state<S: Send + Sync + 'static>(
        raw_path: impl Into<String>,
        state: S,
    ) -> Self {
        Self::Navigate(Navigation::reset_with_state(raw_path, state))
    }

    pub fn back() -> Self {
        Self::Navigate(Navigation::back())
    }
}

impl From<Navigation> for RouterMessage {
    fn from(value: Navigation) -> Self {
        Self::Navigate(value)
    }
}

impl RouteTransitionEvent {
    pub fn new(from: Route, to: Route, direction: RouteTransitionDirection) -> Self {
        Self {
            from,
            to,
            direction,
        }
    }

    pub fn from(&self) -> &Route {
        &self.from
    }
    pub fn to(&self) -> &Route {
        &self.to
    }
    pub fn direction(&self) -> RouteTransitionDirection {
        self.direction
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteError {
    EmptyPath,
    InvalidPattern(String),
    UnknownRoute(String),
    GuardBlocked(String),
    AsyncGuardRequired,
    GuardRedirectLoop,
    StaleNavigation,
}

impl Display for RouteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteError::EmptyPath => write!(f, "route path cannot be empty"),
            RouteError::InvalidPattern(pattern) => {
                write!(f, "invalid route pattern: {pattern}")
            }
            RouteError::UnknownRoute(path) => write!(f, "route is not registered: {path}"),
            RouteError::GuardBlocked(reason) => write!(f, "navigation blocked: {reason}"),
            RouteError::AsyncGuardRequired => {
                write!(f, "route has async guards; use guarded navigation task API")
            }
            RouteError::GuardRedirectLoop => write!(f, "guard redirect loop exceeded limit"),
            RouteError::StaleNavigation => write!(f, "navigation result is stale"),
        }
    }
}

impl Error for RouteError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardRef {
    Sync(usize),
    Async(usize),
}

impl GuardRef {
    fn id(self) -> usize {
        match self {
            GuardRef::Sync(id) | GuardRef::Async(id) => id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDefinition {
    pattern: String,
    name: Option<String>,
    guard_chain: Vec<GuardRef>,
}

impl RouteDefinition {
    pub fn new(pattern: impl Into<String>) -> Result<Self, RouteError> {
        let pattern = normalize_path(pattern.into())?;
        parse_pattern_segments(&pattern)?;
        Ok(Self {
            pattern,
            name: None,
            guard_chain: Vec::new(),
        })
    }

    pub fn named(name: impl Into<String>, pattern: impl Into<String>) -> Result<Self, RouteError> {
        let mut definition = Self::new(pattern)?;
        definition.name = Some(name.into());
        Ok(definition)
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn with_guards(mut self, sync_guard_ids: Vec<usize>, async_guard_ids: Vec<usize>) -> Self {
        self.guard_chain
            .extend(sync_guard_ids.into_iter().map(GuardRef::Sync));
        self.guard_chain
            .extend(async_guard_ids.into_iter().map(GuardRef::Async));
        self
    }

    pub fn with_guard_chain(mut self, guard_chain: Vec<GuardRef>) -> Self {
        self.guard_chain = guard_chain;
        self
    }
}

#[derive(Clone, Default)]
pub struct RouteState {
    value: Option<Arc<dyn Any + Send + Sync>>,
}

impl RouteState {
    pub fn empty() -> Self {
        Self { value: None }
    }

    pub fn new<T: Send + Sync + 'static>(value: T) -> Self {
        Self {
            value: Some(Arc::new(value)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    pub fn is<T: Send + Sync + 'static>(&self) -> bool {
        self.value
            .as_ref()
            .is_some_and(|value| value.as_ref().is::<T>())
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.value.as_deref()?.downcast_ref::<T>()
    }

    pub fn get_cloned<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.get::<T>().cloned()
    }

    pub fn get_arc<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let value = self.value.as_ref()?.clone();
        value.downcast::<T>().ok()
    }
}

impl std::fmt::Debug for RouteState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            f.write_str("RouteState(None)")
        } else {
            f.write_str("RouteState(Some(_))")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Route {
    raw: String,
    path: String,
    pattern: String,
    name: Option<String>,
    params: BTreeMap<String, String>,
    query: BTreeMap<String, String>,
    state: RouteState,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
            && self.path == other.path
            && self.pattern == other.pattern
            && self.name == other.name
            && self.params == other.params
            && self.query == other.query
    }
}

impl Eq for Route {}

impl Route {
    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn params(&self) -> &BTreeMap<String, String> {
        &self.params
    }

    pub fn query_params(&self) -> &BTreeMap<String, String> {
        &self.query
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(String::as_str)
    }

    pub fn query(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(String::as_str)
    }

    pub fn state<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.state.get::<T>()
    }

    pub fn state_cloned<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.state.get_cloned::<T>()
    }

    pub fn state_arc<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.state.get_arc::<T>()
    }

    pub fn has_state<T: Send + Sync + 'static>(&self) -> bool {
        self.state.is::<T>()
    }
}

#[derive(Clone)]
pub struct Router {
    inner: Rc<RouterInner>,
}

#[derive(Clone)]
struct NavigationDispatch {
    route: Route,
    transition: RouteTransitionEvent,
}

struct RouterInner {
    definitions: RefCell<Vec<RouteRecord>>,
    fallback: RefCell<Option<RouteRecord>>,
    stack: RefCell<Vec<Route>>,
    observers: RefCell<Vec<(usize, Rc<dyn Fn(Route)>)>>,
    next_observer_id: Cell<usize>,
    transition_observers: RefCell<Vec<(usize, Rc<dyn Fn(RouteTransitionEvent)>)>>,
    next_transition_observer_id: Cell<usize>,
    sync_guards: RefCell<Vec<(usize, SyncGuard)>>,
    async_guards: RefCell<Vec<(usize, AsyncGuard)>>,
    global_guard_chain: RefCell<Vec<GuardRef>>,
    next_guard_id: Cell<usize>,
    active_navigation_token: Cell<u64>,
    pending_dispatches: RefCell<Vec<NavigationDispatch>>,
    dispatching: Cell<bool>,
}

#[derive(Debug, Clone)]
struct RouteRecord {
    pattern: String,
    name: Option<String>,
    segments: Vec<RouteSegment>,
    guard_chain: Vec<GuardRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavigationOperation {
    Push,
    Replace,
    Reset,
    Back,
}

#[derive(Debug, Clone)]
pub struct PendingNavigation {
    token: u64,
    operation: NavigationOperation,
    from: Route,
    route: Route,
    direction: RouteTransitionDirection,
}

#[derive(Debug, Clone)]
pub struct RouteNavigationResult {
    navigation: Navigation,
    pending: Result<PendingNavigation, RouteError>,
}

impl RouteNavigationResult {
    fn new(navigation: Navigation, pending: Result<PendingNavigation, RouteError>) -> Self {
        Self {
            navigation,
            pending,
        }
    }
}

#[derive(Clone)]
pub struct RouteNavigationTask {
    navigation: Navigation,
    token: u64,
    operation: NavigationOperation,
    from: Route,
    raw_path: String,
    state: RouteState,
    direction: RouteTransitionDirection,
    definitions: Vec<RouteRecord>,
    fallback: Option<RouteRecord>,
    global_guard_chain: Vec<GuardRef>,
    sync_guards: Vec<(usize, SyncGuard)>,
    async_guards: Vec<(usize, AsyncGuard)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RouteSegment {
    Static(String),
    Param(String),
    Wildcard(String),
}

impl Router {
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }

    pub fn new(initial_path: impl Into<String>) -> Self {
        Self::try_new(initial_path).unwrap_or_else(|_| {
            Self::try_new("/").expect("`/` must always be a valid route for router bootstrap")
        })
    }

    pub fn try_new(initial_path: impl Into<String>) -> Result<Self, RouteError> {
        let raw = initial_path.into();
        let route = parse_raw_route(&raw)?;

        Ok(Self {
            inner: Rc::new(RouterInner {
                definitions: RefCell::new(Vec::new()),
                fallback: RefCell::new(None),
                stack: RefCell::new(vec![route]),
                observers: RefCell::new(Vec::new()),
                next_observer_id: Cell::new(1),
                transition_observers: RefCell::new(Vec::new()),
                next_transition_observer_id: Cell::new(1),
                sync_guards: RefCell::new(Vec::new()),
                async_guards: RefCell::new(Vec::new()),
                global_guard_chain: RefCell::new(Vec::new()),
                next_guard_id: Cell::new(1),
                active_navigation_token: Cell::new(0),
                pending_dispatches: RefCell::new(Vec::new()),
                dispatching: Cell::new(false),
            }),
        })
    }

    pub fn route_definitions(&self) -> Vec<RouteDefinition> {
        self.inner
            .definitions
            .borrow()
            .iter()
            .map(|record| RouteDefinition {
                pattern: record.pattern.clone(),
                name: record.name.clone(),
                guard_chain: record.guard_chain.clone(),
            })
            .collect()
    }

    /// Register a fallback route that matches when no registered pattern matches.
    /// The pattern must contain a wildcard segment (e.g., `"*404"` or `"/*rest"`).
    pub fn register_fallback(&self, pattern: impl Into<String>) -> Result<(), RouteError> {
        let pattern = normalize_path(pattern.into())?;
        let segments = parse_pattern_segments(&pattern)?;
        *self.inner.fallback.borrow_mut() = Some(RouteRecord {
            pattern,
            name: Some("__fallback__".to_string()),
            segments,
            guard_chain: Vec::new(),
        });
        Ok(())
    }

    pub fn is_registered(&self, pattern: &str) -> bool {
        let Ok(pattern) = normalize_path(pattern.to_string()) else {
            return false;
        };
        self.inner
            .definitions
            .borrow()
            .iter()
            .any(|record| record.pattern == pattern)
    }

    pub fn resolve(&self, raw_path: impl Into<String>) -> Result<Route, RouteError> {
        let raw_path = raw_path.into();
        let records = self.inner.definitions.borrow();
        let fallback = self.inner.fallback.borrow();
        resolve_with_records(&raw_path, &records, fallback.as_ref()).map(|resolved| resolved.route)
    }

    pub fn current_route(&self) -> Route {
        self.inner
            .stack
            .borrow()
            .last()
            .cloned()
            .expect("router stack always has at least one route")
    }

    pub fn current_path(&self) -> String {
        self.current_route().path
    }

    pub fn current_param(&self, key: &str) -> Option<String> {
        self.current_route().param(key).map(ToOwned::to_owned)
    }

    pub fn current_query(&self, key: &str) -> Option<String> {
        self.current_route().query(key).map(ToOwned::to_owned)
    }

    pub fn current_state<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.current_route().state_arc::<T>()
    }

    pub fn current_state_cloned<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.current_route().state_cloned::<T>()
    }

    pub fn stack_len(&self) -> usize {
        self.inner.stack.borrow().len()
    }

    pub fn can_go_back(&self) -> bool {
        self.stack_len() > 1
    }

    pub fn stack(&self) -> Vec<Route> {
        self.inner.stack.borrow().clone()
    }

    pub fn subscribe(&self, callback: impl Fn(Route) + 'static) -> usize {
        let id = self.inner.next_observer_id.get();
        self.inner.next_observer_id.set(id + 1);
        self.inner
            .observers
            .borrow_mut()
            .push((id, Rc::new(callback) as Rc<dyn Fn(Route)>));
        id
    }

    pub fn unsubscribe(&self, id: usize) -> bool {
        let mut observers = self.inner.observers.borrow_mut();
        let before = observers.len();
        observers.retain(|(observer_id, _)| *observer_id != id);
        before != observers.len()
    }

    pub fn register_definition(&self, definition: RouteDefinition) -> Result<bool, RouteError> {
        if self.is_registered(definition.pattern()) {
            return Ok(false);
        }

        let pattern = definition.pattern;
        let name = definition.name;
        let guard_chain = definition.guard_chain;
        let segments = parse_pattern_segments(&pattern)?;
        self.inner.definitions.borrow_mut().push(RouteRecord {
            pattern,
            name,
            segments,
            guard_chain,
        });
        self.refresh_stack_routes();
        Ok(true)
    }

    fn refresh_stack_routes(&self) {
        let entries = self
            .inner
            .stack
            .borrow()
            .iter()
            .map(|route| (route.raw().to_string(), route.state.clone()))
            .collect::<Vec<_>>();

        let refreshed = entries
            .into_iter()
            .map(|(raw, state)| {
                let mut route = self.resolve(raw)?;
                route.state = state;
                Ok::<Route, RouteError>(route)
            })
            .collect::<Vec<_>>();

        let mut stack = self.inner.stack.borrow_mut();
        for (slot, route) in stack.iter_mut().zip(refreshed) {
            if let Ok(route) = route {
                *slot = route;
            }
        }
    }

    fn notify(&self, route: Route) {
        let callbacks = self
            .inner
            .observers
            .borrow()
            .iter()
            .map(|(_, callback)| callback.clone())
            .collect::<Vec<_>>();

        for callback in callbacks {
            callback(route.clone());
        }
    }

    /// Register a synchronous navigation guard.
    pub fn add_guard(
        &self,
        guard: impl Fn(RouteGuardContext) -> RouteGuardDecision + Send + Sync + 'static,
    ) -> usize {
        let id = self.add_route_guard(guard);
        self.inner
            .global_guard_chain
            .borrow_mut()
            .push(GuardRef::Sync(id));
        id
    }

    #[doc(hidden)]
    pub fn add_route_guard(
        &self,
        guard: impl Fn(RouteGuardContext) -> RouteGuardDecision + Send + Sync + 'static,
    ) -> usize {
        let id = self.inner.next_guard_id.get();
        self.inner.next_guard_id.set(id + 1);
        self.inner
            .sync_guards
            .borrow_mut()
            .push((id, Arc::new(guard)));
        id
    }

    /// Register an asynchronous navigation guard.
    pub fn add_async_guard<Fut>(
        &self,
        guard: impl Fn(RouteGuardContext) -> Fut + Send + Sync + 'static,
    ) -> usize
    where
        Fut: Future<Output = RouteGuardDecision> + Send + 'static,
    {
        let id = self.add_route_async_guard(guard);
        self.inner
            .global_guard_chain
            .borrow_mut()
            .push(GuardRef::Async(id));
        id
    }

    #[doc(hidden)]
    pub fn add_route_async_guard<Fut>(
        &self,
        guard: impl Fn(RouteGuardContext) -> Fut + Send + Sync + 'static,
    ) -> usize
    where
        Fut: Future<Output = RouteGuardDecision> + Send + 'static,
    {
        let id = self.inner.next_guard_id.get();
        self.inner.next_guard_id.set(id + 1);
        let guard =
            Arc::new(move |context: RouteGuardContext| Box::pin(guard(context)) as BoxGuardFuture);
        self.inner.async_guards.borrow_mut().push((id, guard));
        id
    }

    /// Remove a previously registered guard.
    pub fn remove_guard(&self, id: usize) -> bool {
        let mut removed = false;

        let mut sync_guards = self.inner.sync_guards.borrow_mut();
        let before = sync_guards.len();
        sync_guards.retain(|(guard_id, _)| *guard_id != id);
        removed |= before != sync_guards.len();
        drop(sync_guards);

        let mut async_guards = self.inner.async_guards.borrow_mut();
        let before = async_guards.len();
        async_guards.retain(|(guard_id, _)| *guard_id != id);
        removed |= before != async_guards.len();
        drop(async_guards);

        self.inner
            .global_guard_chain
            .borrow_mut()
            .retain(|guard_ref| guard_ref.id() != id);

        removed
    }

    #[doc(hidden)]
    pub fn begin_navigation(
        &self,
        navigation: Navigation,
    ) -> Result<RouteNavigationTask, RouteError> {
        let (raw_path, state, direction, operation) = self.navigation_parts(&navigation)?;
        let token = self.next_navigation_token();
        Ok(RouteNavigationTask {
            navigation,
            token,
            operation,
            from: self.current_route(),
            raw_path,
            state,
            direction,
            definitions: self.inner.definitions.borrow().clone(),
            fallback: self.inner.fallback.borrow().clone(),
            global_guard_chain: self.inner.global_guard_chain.borrow().clone(),
            sync_guards: self.inner.sync_guards.borrow().clone(),
            async_guards: self.inner.async_guards.borrow().clone(),
        })
    }

    #[doc(hidden)]
    pub fn commit_navigation_sync(&self, navigation: Navigation) -> NavigationEvent {
        let result = self
            .resolve_sync_navigation(navigation.clone())
            .and_then(|pending| self.commit_navigation(pending));
        NavigationEvent::new(navigation, result)
    }

    #[doc(hidden)]
    pub fn complete_navigation(&self, result: RouteNavigationResult) -> NavigationEvent {
        let RouteNavigationResult {
            navigation,
            pending,
        } = result;
        let result = pending.and_then(|pending| self.commit_navigation(pending));
        NavigationEvent::new(navigation, result)
    }

    fn resolve_sync_navigation(
        &self,
        navigation: Navigation,
    ) -> Result<PendingNavigation, RouteError> {
        let (raw_path, state, direction, operation) = match self.navigation_parts(&navigation) {
            Ok(parts) => parts,
            Err(error) => return Err(error),
        };
        let token = self.next_navigation_token();
        let mut task = RouteNavigationTask {
            navigation,
            token,
            operation,
            from: self.current_route(),
            raw_path,
            state,
            direction,
            definitions: self.inner.definitions.borrow().clone(),
            fallback: self.inner.fallback.borrow().clone(),
            global_guard_chain: self.inner.global_guard_chain.borrow().clone(),
            sync_guards: self.inner.sync_guards.borrow().clone(),
            async_guards: self.inner.async_guards.borrow().clone(),
        };

        task.run_sync()
    }

    fn navigation_parts(
        &self,
        navigation: &Navigation,
    ) -> Result<
        (
            String,
            RouteState,
            RouteTransitionDirection,
            NavigationOperation,
        ),
        RouteError,
    > {
        match navigation {
            Navigation::Push(target) => Ok((
                target.raw_path.clone(),
                target.state.clone(),
                RouteTransitionDirection::Forward,
                NavigationOperation::Push,
            )),
            Navigation::Replace(target) => Ok((
                target.raw_path.clone(),
                target.state.clone(),
                RouteTransitionDirection::Replace,
                NavigationOperation::Replace,
            )),
            Navigation::Reset(target) => Ok((
                target.raw_path.clone(),
                target.state.clone(),
                RouteTransitionDirection::Replace,
                NavigationOperation::Reset,
            )),
            Navigation::Back => {
                let stack = self.inner.stack.borrow();
                if stack.len() <= 1 {
                    return Err(RouteError::UnknownRoute("<back>".to_string()));
                }
                Ok((
                    stack[stack.len() - 2].raw().to_string(),
                    RouteState::empty(),
                    RouteTransitionDirection::Backward,
                    NavigationOperation::Back,
                ))
            }
        }
    }

    fn commit_navigation(&self, navigation: PendingNavigation) -> Result<Route, RouteError> {
        if self.inner.active_navigation_token.get() != navigation.token {
            return Err(RouteError::StaleNavigation);
        }

        let prev = navigation.from;
        let route = navigation.route;
        match navigation.operation {
            NavigationOperation::Push => {
                self.inner.stack.borrow_mut().push(route.clone());
            }
            NavigationOperation::Replace => {
                let mut stack = self.inner.stack.borrow_mut();
                if let Some(last) = stack.last_mut() {
                    *last = route.clone();
                } else {
                    stack.push(route.clone());
                }
            }
            NavigationOperation::Reset => {
                let mut stack = self.inner.stack.borrow_mut();
                stack.clear();
                stack.push(route.clone());
            }
            NavigationOperation::Back => {
                let mut stack = self.inner.stack.borrow_mut();
                if stack.len() <= 1 {
                    return Err(RouteError::UnknownRoute("<back>".to_string()));
                }
                if route.raw() == stack[stack.len() - 2].raw() {
                    stack.pop();
                } else if let Some(last) = stack.last_mut() {
                    *last = route.clone();
                }
            }
        }

        self.dispatch_navigation(
            route.clone(),
            RouteTransitionEvent::new(prev, route.clone(), navigation.direction),
        );
        Ok(route)
    }

    fn next_navigation_token(&self) -> u64 {
        let token = self.inner.active_navigation_token.get().wrapping_add(1);
        self.inner.active_navigation_token.set(token);
        token
    }

    /// Subscribe to route transition events (receives from, to, direction).
    /// Returns a subscription ID for removal.
    pub fn subscribe_transition(&self, callback: impl Fn(RouteTransitionEvent) + 'static) -> usize {
        let id = self.inner.next_transition_observer_id.get();
        self.inner.next_transition_observer_id.set(id + 1);
        self.inner
            .transition_observers
            .borrow_mut()
            .push((id, Rc::new(callback) as Rc<dyn Fn(RouteTransitionEvent)>));
        id
    }

    /// Remove a previously registered transition observer.
    pub fn unsubscribe_transition(&self, id: usize) -> bool {
        let mut observers = self.inner.transition_observers.borrow_mut();
        let before = observers.len();
        observers.retain(|(observer_id, _)| *observer_id != id);
        before != observers.len()
    }

    fn notify_transition(&self, event: RouteTransitionEvent) {
        let callbacks = self
            .inner
            .transition_observers
            .borrow()
            .iter()
            .map(|(_, cb)| cb.clone())
            .collect::<Vec<_>>();
        for cb in callbacks {
            cb(event.clone());
        }
    }

    fn dispatch_navigation(&self, route: Route, transition: RouteTransitionEvent) {
        self.inner
            .pending_dispatches
            .borrow_mut()
            .push(NavigationDispatch { route, transition });

        if self.inner.dispatching.replace(true) {
            return;
        }

        loop {
            let next = {
                let mut pending = self.inner.pending_dispatches.borrow_mut();
                if pending.is_empty() {
                    None
                } else {
                    Some(pending.remove(0))
                }
            };

            let Some(dispatch) = next else {
                self.inner.dispatching.set(false);
                break;
            };

            self.notify(dispatch.route);
            self.notify_transition(dispatch.transition);
        }
    }
}

impl RouteNavigationTask {
    pub async fn run(self) -> RouteNavigationResult {
        let navigation = self.navigation.clone();
        RouteNavigationResult::new(navigation, self.run_pending().await)
    }

    async fn run_pending(self) -> Result<PendingNavigation, RouteError> {
        let mut raw_path = self.raw_path.clone();
        for _ in 0..MAX_GUARD_REDIRECTS {
            let resolved =
                resolve_with_records(&raw_path, &self.definitions, self.fallback.as_ref())?;
            let chain = self.guard_chain_for(&resolved.guard_chain);

            let mut redirected = None;
            for guard_ref in chain {
                let context = RouteGuardContext::new(
                    self.from.clone(),
                    resolved.route.clone(),
                    self.direction,
                );
                let decision = match guard_ref {
                    GuardRef::Sync(id) => {
                        let Some((_, guard)) = self
                            .sync_guards
                            .iter()
                            .find(|(guard_id, _)| *guard_id == id)
                        else {
                            continue;
                        };
                        guard(context)
                    }
                    GuardRef::Async(id) => {
                        let Some((_, guard)) = self
                            .async_guards
                            .iter()
                            .find(|(guard_id, _)| *guard_id == id)
                        else {
                            continue;
                        };
                        guard(context).await
                    }
                };

                match decision {
                    RouteGuardDecision::Allow => {}
                    RouteGuardDecision::Block(reason) => {
                        return Err(RouteError::GuardBlocked(reason));
                    }
                    RouteGuardDecision::Redirect(path) => {
                        redirected = Some(path);
                        break;
                    }
                }
            }

            if let Some(next_path) = redirected {
                raw_path = next_path;
                continue;
            }

            let mut route = resolved.route;
            route.state = self.state;
            return Ok(PendingNavigation {
                token: self.token,
                operation: self.operation,
                from: self.from,
                route,
                direction: self.direction,
            });
        }

        Err(RouteError::GuardRedirectLoop)
    }

    fn run_sync(&mut self) -> Result<PendingNavigation, RouteError> {
        let mut raw_path = self.raw_path.clone();
        for _ in 0..MAX_GUARD_REDIRECTS {
            let resolved =
                resolve_with_records(&raw_path, &self.definitions, self.fallback.as_ref())?;
            let chain = self.guard_chain_for(&resolved.guard_chain);

            let mut redirected = None;
            for guard_ref in chain {
                match guard_ref {
                    GuardRef::Async(_) => {
                        if self
                            .async_guards
                            .iter()
                            .any(|(guard_id, _)| *guard_id == guard_ref.id())
                        {
                            return Err(RouteError::AsyncGuardRequired);
                        }
                    }
                    GuardRef::Sync(id) => {
                        let Some((_, guard)) = self
                            .sync_guards
                            .iter()
                            .find(|(guard_id, _)| *guard_id == id)
                        else {
                            continue;
                        };
                        let context = RouteGuardContext::new(
                            self.from.clone(),
                            resolved.route.clone(),
                            self.direction,
                        );
                        match guard(context) {
                            RouteGuardDecision::Allow => {}
                            RouteGuardDecision::Block(reason) => {
                                return Err(RouteError::GuardBlocked(reason));
                            }
                            RouteGuardDecision::Redirect(path) => {
                                redirected = Some(path);
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(next_path) = redirected {
                raw_path = next_path;
                continue;
            }

            let mut route = resolved.route;
            route.state = self.state.clone();
            return Ok(PendingNavigation {
                token: self.token,
                operation: self.operation,
                from: self.from.clone(),
                route,
                direction: self.direction,
            });
        }

        Err(RouteError::GuardRedirectLoop)
    }

    fn guard_chain_for(&self, route_guard_chain: &[GuardRef]) -> Vec<GuardRef> {
        let mut chain = Vec::with_capacity(self.global_guard_chain.len() + route_guard_chain.len());
        chain.extend(self.global_guard_chain.iter().copied());
        chain.extend(route_guard_chain.iter().copied());
        chain
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new("/")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationStack<Route> {
    stack: Vec<Route>,
}

impl<Route: Clone> NavigationStack<Route> {
    pub fn new(initial: Route) -> Self {
        Self {
            stack: vec![initial],
        }
    }

    pub fn current(&self) -> &Route {
        self.stack
            .last()
            .expect("navigation stack must always contain at least one route")
    }

    pub fn push(&mut self, route: Route) {
        self.stack.push(route);
    }

    pub fn replace(&mut self, route: Route) {
        if let Some(current) = self.stack.last_mut() {
            *current = route;
        } else {
            self.stack.push(route);
        }
    }

    pub fn reset(&mut self, route: Route) {
        self.stack.clear();
        self.stack.push(route);
    }

    pub fn back(&mut self) -> bool {
        if self.stack.len() <= 1 {
            return false;
        }
        self.stack.pop();
        true
    }

    pub fn can_go_back(&self) -> bool {
        self.stack.len() > 1
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn stack(&self) -> &[Route] {
        &self.stack
    }
}

#[derive(Debug, Clone)]
struct ResolvedRoute {
    route: Route,
    guard_chain: Vec<GuardRef>,
}

fn resolve_with_records(
    raw_path: &str,
    records: &[RouteRecord],
    fallback: Option<&RouteRecord>,
) -> Result<ResolvedRoute, RouteError> {
    let (path, query) = split_raw_path(raw_path)?;

    if records.is_empty() {
        return Ok(ResolvedRoute {
            route: Route {
                raw: join_raw_path(&path, &query),
                path: path.clone(),
                pattern: path,
                name: None,
                params: BTreeMap::new(),
                query,
                state: RouteState::empty(),
            },
            guard_chain: Vec::new(),
        });
    }

    let path_segments = path_segments(&path);
    let mut best_match: Option<(Vec<u8>, ResolvedRoute)> = None;

    for record in records {
        if let Some(params) = match_segments(&record.segments, &path_segments) {
            let spec = route_specificity(&record.segments);
            let is_better = match &best_match {
                None => true,
                Some((existing_spec, _)) => spec > *existing_spec,
            };
            if is_better {
                best_match = Some((
                    spec,
                    ResolvedRoute {
                        route: Route {
                            raw: join_raw_path(&path, &query),
                            path: path.clone(),
                            pattern: record.pattern.clone(),
                            name: record.name.clone(),
                            params,
                            query: query.clone(),
                            state: RouteState::empty(),
                        },
                        guard_chain: record.guard_chain.clone(),
                    },
                ));
            }
        }
    }

    if best_match.is_none() {
        if let Some(fallback) = fallback {
            if let Some(params) = match_segments(&fallback.segments, &path_segments) {
                best_match = Some((
                    route_specificity(&fallback.segments),
                    ResolvedRoute {
                        route: Route {
                            raw: join_raw_path(&path, &query),
                            path: path.clone(),
                            pattern: fallback.pattern.clone(),
                            name: fallback.name.clone(),
                            params,
                            query: query.clone(),
                            state: RouteState::empty(),
                        },
                        guard_chain: fallback.guard_chain.clone(),
                    },
                ));
            }
        }
    }

    best_match
        .map(|(_, resolved)| resolved)
        .ok_or_else(|| RouteError::UnknownRoute(raw_path.to_string()))
}

fn parse_raw_route(raw_path: &str) -> Result<Route, RouteError> {
    let (path, query) = split_raw_path(raw_path)?;
    Ok(Route {
        raw: join_raw_path(&path, &query),
        pattern: path.clone(),
        path,
        name: None,
        params: BTreeMap::new(),
        query,
        state: RouteState::empty(),
    })
}

fn split_raw_path(raw_path: &str) -> Result<(String, BTreeMap<String, String>), RouteError> {
    if raw_path.trim().is_empty() {
        return Err(RouteError::EmptyPath);
    }

    let (path_part, query_part) = match raw_path.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (raw_path, None),
    };

    let path = normalize_path(path_part.to_string())?;
    let query = parse_query(query_part.unwrap_or_default());
    Ok((path, query))
}

fn normalize_path(path: String) -> Result<String, RouteError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(RouteError::EmptyPath);
    }

    let with_leading_slash = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    let mut cleaned_segments = with_leading_slash
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if cleaned_segments.is_empty() {
        return Ok("/".to_string());
    }

    let mut normalized = String::new();
    for segment in cleaned_segments.drain(..) {
        normalized.push('/');
        normalized.push_str(segment);
    }

    Ok(normalized)
}

fn parse_pattern_segments(pattern: &str) -> Result<Vec<RouteSegment>, RouteError> {
    let path = normalize_path(pattern.to_string())?;
    if path == "/" {
        return Ok(Vec::new());
    }

    let parts = path_segments(&path);
    let mut segments = Vec::with_capacity(parts.len());
    for part in parts {
        if part.starts_with(':') {
            let name = part.trim_start_matches(':');
            if name.is_empty() {
                return Err(RouteError::InvalidPattern(pattern.to_string()));
            }
            segments.push(RouteSegment::Param(name.to_string()));
            continue;
        }

        if part.starts_with('*') {
            let wildcard = part.trim_start_matches('*');
            let key = if wildcard.is_empty() {
                "wildcard".to_string()
            } else {
                wildcard.to_string()
            };
            segments.push(RouteSegment::Wildcard(key));
            continue;
        }

        segments.push(RouteSegment::Static(part.to_string()));
    }

    Ok(segments)
}

fn path_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn match_segments(
    pattern_segments: &[RouteSegment],
    path_segments: &[&str],
) -> Option<BTreeMap<String, String>> {
    let mut params = BTreeMap::new();
    let mut path_index = 0usize;

    for (segment_index, segment) in pattern_segments.iter().enumerate() {
        match segment {
            RouteSegment::Static(expected) => {
                let got = path_segments.get(path_index)?;
                if *got != expected {
                    return None;
                }
                path_index += 1;
            }
            RouteSegment::Param(name) => {
                let got = path_segments.get(path_index)?;
                params.insert(name.clone(), percent_decode(got));
                path_index += 1;
            }
            RouteSegment::Wildcard(name) => {
                let rest = path_segments[path_index..].join("/");
                params.insert(name.clone(), percent_decode(&rest));
                path_index = path_segments.len();
                if segment_index + 1 != pattern_segments.len() {
                    return None;
                }
                break;
            }
        }
    }

    if path_index != path_segments.len() {
        return None;
    }

    Some(params)
}

fn parse_query(query: &str) -> BTreeMap<String, String> {
    let mut params = BTreeMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = match pair.split_once('=') {
            Some((key, value)) => (key, value),
            None => (pair, ""),
        };
        params.insert(percent_decode(key), percent_decode(value));
    }
    params
}

/// Joins a path with query parameters into a raw path string.
pub fn join_raw_path(path: &str, query: &BTreeMap<String, String>) -> String {
    if query.is_empty() {
        return path.to_string();
    }

    let mut raw = String::from(path);
    raw.push('?');
    for (index, (key, value)) in query.iter().enumerate() {
        if index > 0 {
            raw.push('&');
        }
        raw.push_str(key);
        raw.push('=');
        raw.push_str(value);
    }
    raw
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = bytes[i + 1];
                let lo = bytes[i + 2];
                if let (Some(hi), Some(lo)) = (hex_value(hi), hex_value(lo)) {
                    out.push((hi * 16 + lo) as char);
                    i += 3;
                } else {
                    out.push('%');
                    i += 1;
                }
            }
            b => {
                out.push(b as char);
                i += 1;
            }
        }
    }
    out
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Specificity score for route segments: Static(3) > Param(2) > Wildcard(1).
/// Higher scores win; ties broken lexicographically by segment position.
fn route_specificity(segments: &[RouteSegment]) -> Vec<u8> {
    segments
        .iter()
        .map(|s| match s {
            RouteSegment::Static(_) => 3,
            RouteSegment::Param(_) => 2,
            RouteSegment::Wildcard(_) => 1,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    use super::{
        GuardRef, Navigation, NavigationEvent, RouteDefinition, RouteError, RouteGuardDecision,
        RouteTransitionDirection, Router,
    };

    fn block_on<F: Future>(future: F) -> F::Output {
        fn clone(_: *const ()) -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        fn wake(_: *const ()) {}
        fn wake_by_ref(_: *const ()) {}
        fn drop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut context = Context::from_waker(&waker);
        let mut future = Box::pin(future);
        loop {
            match Pin::new(&mut future).poll(&mut context) {
                Poll::Ready(value) => return value,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    fn navigate(router: &Router, navigation: Navigation) -> NavigationEvent {
        router.commit_navigation_sync(navigation)
    }

    fn route(event: NavigationEvent) -> Result<super::Route, RouteError> {
        event.result
    }

    #[test]
    fn match_and_extract_route_params_and_query() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/users/:id").expect("definition"))
            .expect("register route");
        let route = router
            .resolve("/users/42?tab=profile")
            .expect("resolve route");

        assert_eq!(route.path(), "/users/42");
        assert_eq!(route.pattern(), "/users/:id");
        assert_eq!(route.param("id"), Some("42"));
        assert_eq!(route.query("tab"), Some("profile"));
    }

    #[test]
    fn stack_push_replace_back_and_reset() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::named("home", "/").expect("home"))
            .expect("register home");
        router
            .register_definition(RouteDefinition::named("about", "/about").expect("about"))
            .expect("register about");
        route(navigate(&router, Navigation::push("/about"))).expect("push route");
        assert_eq!(router.stack_len(), 2);
        assert_eq!(router.current_path(), "/about");

        route(navigate(&router, Navigation::replace("/"))).expect("replace route");
        assert_eq!(router.current_path(), "/");

        route(navigate(&router, Navigation::back())).expect("back route");
        assert_eq!(router.stack_len(), 1);
        assert!(route(navigate(&router, Navigation::back())).is_err());

        route(navigate(&router, Navigation::reset("/about"))).expect("reset route");
        assert_eq!(router.stack_len(), 1);
        assert_eq!(router.current_path(), "/about");
    }

    #[test]
    fn sync_global_guard_allows_blocks_and_redirects() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/").expect("home"))
            .expect("register home");
        router
            .register_definition(RouteDefinition::new("/about").expect("about"))
            .expect("register about");
        router
            .register_definition(RouteDefinition::new("/login").expect("login"))
            .expect("register login");
        router.add_guard(|context| match context.to.path() {
            "/about" => RouteGuardDecision::Block("closed".to_string()),
            "/login" => RouteGuardDecision::Allow,
            _ => RouteGuardDecision::Redirect("/login".to_string()),
        });

        assert_eq!(
            route(navigate(&router, Navigation::push("/about"))).expect_err("blocked"),
            RouteError::GuardBlocked("closed".to_string())
        );
        let route = route(navigate(&router, Navigation::push("/"))).expect("redirected");
        assert_eq!(route.path(), "/login");
    }

    #[test]
    fn async_global_guard_allows_blocks_and_redirects() {
        let router = Router::new("/");
        for path in ["/", "/admin", "/blocked", "/login"] {
            router
                .register_definition(RouteDefinition::new(path).expect("route"))
                .expect("register route");
        }
        router.add_async_guard(|context| async move {
            match context.to.path() {
                "/blocked" => RouteGuardDecision::Block("async closed".to_string()),
                "/admin" => RouteGuardDecision::Redirect("/login".to_string()),
                _ => RouteGuardDecision::Allow,
            }
        });

        let blocked = block_on(
            router
                .begin_navigation(Navigation::push("/blocked"))
                .expect("begin blocked")
                .run(),
        );
        assert_eq!(
            router
                .complete_navigation(blocked)
                .result
                .expect_err("blocked"),
            RouteError::GuardBlocked("async closed".to_string())
        );

        let pending = block_on(
            router
                .begin_navigation(Navigation::push("/admin"))
                .expect("begin admin")
                .run(),
        );
        let route = router.complete_navigation(pending).result.expect("finish");
        assert_eq!(route.path(), "/login");
    }

    #[test]
    fn route_guard_inherits_to_nested_child_and_runs_parent_first() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/").expect("home"))
            .expect("register home");

        let order = Arc::new(Mutex::new(Vec::new()));
        let parent_order = order.clone();
        let parent = router.add_route_guard(move |_| {
            parent_order.lock().expect("order").push("parent");
            RouteGuardDecision::Allow
        });
        let child_order = order.clone();
        let child = router.add_route_guard(move |_| {
            child_order.lock().expect("order").push("child");
            RouteGuardDecision::Allow
        });
        router
            .register_definition(
                RouteDefinition::new("/users/:id/settings")
                    .expect("child")
                    .with_guard_chain(vec![GuardRef::Sync(parent), GuardRef::Sync(child)]),
            )
            .expect("register child");

        route(navigate(&router, Navigation::push("/users/7/settings"))).expect("guarded child");
        assert_eq!(order.lock().expect("order").as_slice(), ["parent", "child"]);
    }

    #[test]
    fn redirect_loop_is_limited() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/a").expect("a"))
            .expect("register a");
        router
            .register_definition(RouteDefinition::new("/b").expect("b"))
            .expect("register b");
        router.add_guard(|context| {
            if context.to.path() == "/a" {
                RouteGuardDecision::Redirect("/b".to_string())
            } else {
                RouteGuardDecision::Redirect("/a".to_string())
            }
        });

        assert_eq!(
            route(navigate(&router, Navigation::push("/a"))).expect_err("loop"),
            RouteError::GuardRedirectLoop
        );
    }

    #[test]
    fn stale_pending_navigation_does_not_commit() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/").expect("home"))
            .expect("register home");
        router
            .register_definition(RouteDefinition::new("/first").expect("first"))
            .expect("register first");
        router
            .register_definition(RouteDefinition::new("/second").expect("second"))
            .expect("register second");
        router.add_async_guard(|_| async { RouteGuardDecision::Allow });

        let first = block_on(
            router
                .begin_navigation(Navigation::push("/first"))
                .expect("begin first")
                .run(),
        );
        let second = block_on(
            router
                .begin_navigation(Navigation::push("/second"))
                .expect("begin second")
                .run(),
        );

        assert_eq!(
            router.complete_navigation(first).result.expect_err("stale"),
            RouteError::StaleNavigation
        );
        let route = router
            .complete_navigation(second)
            .result
            .expect("finish second");
        assert_eq!(route.path(), "/second");
    }

    #[test]
    fn sync_command_reports_async_guard_required() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/").expect("home"))
            .expect("register home");
        let guard = router.add_route_async_guard(|_| async { RouteGuardDecision::Allow });
        router
            .register_definition(
                RouteDefinition::new("/admin")
                    .expect("admin")
                    .with_guard_chain(vec![GuardRef::Async(guard)]),
            )
            .expect("register admin");

        assert_eq!(
            route(navigate(&router, Navigation::push("/admin"))).expect_err("async required"),
            RouteError::AsyncGuardRequired
        );
    }

    #[test]
    fn named_route_definitions_resolve() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::named("home", "/").expect("home"))
            .expect("register home");
        router
            .register_definition(RouteDefinition::named("detail", "/detail/:id").expect("detail"))
            .expect("register detail");

        let detail = router.resolve("/detail/9").expect("resolve");
        assert_eq!(detail.name(), Some("detail"));
        assert_eq!(detail.param("id"), Some("9"));
    }

    #[test]
    fn path_navigation_emits_transition() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::named("home", "/").expect("home"))
            .expect("register home");
        router
            .register_definition(RouteDefinition::named("detail", "/detail/:id").expect("detail"))
            .expect("register detail");

        let transition_id = router.subscribe_transition(|event| {
            assert_eq!(event.from().path(), "/");
            assert_eq!(event.to().path(), "/detail/9");
            assert_eq!(event.direction(), RouteTransitionDirection::Forward);
        });

        route(navigate(&router, Navigation::push("/detail/9"))).expect("push path");
        assert_eq!(router.current_param("id"), Some("9".to_string()));
        assert!(router.unsubscribe_transition(transition_id));
        route(navigate(&router, Navigation::back())).expect("back");
        assert_eq!(router.current_path(), "/");
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ScrollState {
        offset: u32,
    }

    #[test]
    fn typed_state_is_stored_per_route_entry() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::named("home", "/").expect("home"))
            .expect("register home");
        router
            .register_definition(RouteDefinition::named("detail", "/detail/:id").expect("detail"))
            .expect("register detail");
        route(navigate(&router, Navigation::replace("/"))).expect("resolve initial home");

        route(navigate(
            &router,
            Navigation::push_with_state("/detail/9", ScrollState { offset: 128 }),
        ))
        .expect("push with state");

        assert_eq!(router.current_param("id"), Some("9".to_string()));
        assert_eq!(
            router.current_state_cloned::<ScrollState>(),
            Some(ScrollState { offset: 128 })
        );

        route(navigate(&router, Navigation::back())).expect("back");
        assert!(router.current_state::<ScrollState>().is_none());
    }

    #[test]
    fn specificity_static_beats_param() {
        let router = Router::new("/");
        // Register param route first, static route second
        router
            .register_definition(RouteDefinition::new("/users/:id").expect("param"))
            .expect("register param");
        router
            .register_definition(RouteDefinition::new("/users/me").expect("static"))
            .expect("register static");

        let route = router.resolve("/users/me").expect("resolve");
        assert_eq!(route.pattern(), "/users/me");

        let route2 = router.resolve("/users/42").expect("resolve param");
        assert_eq!(route2.pattern(), "/users/:id");
        assert_eq!(route2.param("id"), Some("42"));
    }

    #[test]
    fn specificity_param_beats_wildcard() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/files/*rest").expect("wildcard"))
            .expect("register wildcard");
        router
            .register_definition(RouteDefinition::new("/files/:id").expect("param"))
            .expect("register param");

        let route = router.resolve("/files/42").expect("resolve");
        assert_eq!(route.pattern(), "/files/:id");

        let route2 = router.resolve("/files/a/b/c").expect("resolve wildcard");
        assert_eq!(route2.pattern(), "/files/*rest");
    }

    #[test]
    fn fallback_matches_unregistered_paths() {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::named("home", "/").expect("home"))
            .expect("register home");
        router
            .register_definition(RouteDefinition::named("about", "/about").expect("about"))
            .expect("register about");
        router
            .register_fallback("*rest")
            .expect("register fallback");

        // Known route matches normally
        let about = router.resolve("/about").expect("resolve");
        assert_eq!(about.pattern(), "/about");

        // Unknown route falls back to the catch-all
        let unknown = router.resolve("/nonexistent/page").expect("fallback");
        assert_eq!(unknown.pattern(), "/*rest");
        assert_eq!(unknown.param("rest"), Some("nonexistent/page"));
    }
}
