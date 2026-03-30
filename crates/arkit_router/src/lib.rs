use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

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

impl RouteTransitionEvent {
    pub fn new(from: Route, to: Route, direction: RouteTransitionDirection) -> Self {
        Self { from, to, direction }
    }

    pub fn from(&self) -> &Route { &self.from }
    pub fn to(&self) -> &Route { &self.to }
    pub fn direction(&self) -> RouteTransitionDirection { self.direction }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteError {
    EmptyPath,
    InvalidPattern(String),
    UnknownRoute(String),
    GuardBlocked(String),
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
        }
    }
}

impl Error for RouteError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDefinition {
    pattern: String,
    name: Option<String>,
}

impl RouteDefinition {
    pub fn new(pattern: impl Into<String>) -> Result<Self, RouteError> {
        let pattern = normalize_path(pattern.into())?;
        parse_pattern_segments(&pattern)?;
        Ok(Self {
            pattern,
            name: None,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    raw: String,
    path: String,
    pattern: String,
    name: Option<String>,
    params: BTreeMap<String, String>,
    query: BTreeMap<String, String>,
}

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
}

#[derive(Clone)]
pub struct Router {
    inner: Rc<RouterInner>,
}

struct RouterInner {
    definitions: RefCell<Vec<RouteRecord>>,
    fallback: RefCell<Option<RouteRecord>>,
    route_trees: RefCell<Vec<RouteTreeNode>>,
    stack: RefCell<Vec<Route>>,
    observers: RefCell<Vec<(usize, Rc<dyn Fn(Route)>)>>,
    next_observer_id: Cell<usize>,
    transition_observers: RefCell<Vec<(usize, Rc<dyn Fn(RouteTransitionEvent)>)>>,
    next_transition_observer_id: Cell<usize>,
    guards: RefCell<Vec<(usize, Rc<dyn Fn(&str) -> Option<String>>)>>,
    next_guard_id: Cell<usize>,
}

#[derive(Debug, Clone)]
struct RouteRecord {
    pattern: String,
    name: Option<String>,
    segments: Vec<RouteSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RouteSegment {
    Static(String),
    Param(String),
    Wildcard(String),
}

impl Router {
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
                route_trees: RefCell::new(Vec::new()),
                stack: RefCell::new(vec![route]),
                observers: RefCell::new(Vec::new()),
                next_observer_id: Cell::new(1),
                transition_observers: RefCell::new(Vec::new()),
                next_transition_observer_id: Cell::new(1),
                guards: RefCell::new(Vec::new()),
                next_guard_id: Cell::new(1),
            }),
        })
    }

    pub fn register(&self, pattern: impl Into<String>) -> Result<bool, RouteError> {
        self.register_definition(RouteDefinition::new(pattern)?)
    }

    pub fn register_named(
        &self,
        name: impl Into<String>,
        pattern: impl Into<String>,
    ) -> Result<bool, RouteError> {
        self.register_definition(RouteDefinition::named(name, pattern)?)
    }

    pub fn register_definitions<I>(&self, definitions: I) -> Result<(), RouteError>
    where
        I: IntoIterator<Item = RouteDefinition>,
    {
        for definition in definitions {
            let _ = self.register_definition(definition)?;
        }
        Ok(())
    }

    pub fn route_definitions(&self) -> Vec<RouteDefinition> {
        self.inner
            .definitions
            .borrow()
            .iter()
            .map(|record| RouteDefinition {
                pattern: record.pattern.clone(),
                name: record.name.clone(),
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
        });
        Ok(())
    }

    /// Register a nested route tree. Returns an error if any pattern is invalid.
    pub fn register_tree(&self, root: RouteNode) -> Result<(), RouteError> {
        let tree = RouteTreeNode::from_route_node(root)?;
        self.inner.route_trees.borrow_mut().push(tree);
        Ok(())
    }

    /// Resolve a path against registered nested route trees.
    /// Returns `Ok(ResolvedRoute)` with segments from root to leaf if a tree matches,
    /// or `Err(RouteError::UnknownRoute)` if no tree matches.
    pub fn resolve_nested(&self, raw_path: impl Into<String>) -> Result<ResolvedRoute, RouteError> {
        let raw_path = raw_path.into();
        let (path, query) = split_raw_path(&raw_path)?;
        let path_segs = path_segments(&path);
        let trees = self.inner.route_trees.borrow();

        if trees.is_empty() {
            return Err(RouteError::UnknownRoute(raw_path));
        }

        for tree in trees.iter() {
            if let Some(segments) = resolve_tree(tree, &path_segs) {
                return Ok(ResolvedRoute {
                    raw: join_raw_path(&path, &query),
                    path,
                    query,
                    segments,
                });
            }
        }

        Err(RouteError::UnknownRoute(raw_path))
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
        let (path, query) = split_raw_path(&raw_path)?;
        let records = self.inner.definitions.borrow();

        if records.is_empty() {
            return Ok(Route {
                raw: join_raw_path(&path, &query),
                path: path.clone(),
                pattern: path,
                name: None,
                params: BTreeMap::new(),
                query,
            });
        }

        let path_segments = path_segments(&path);
        let mut best_match: Option<(Vec<u8>, Route)> = None;

        for record in records.iter() {
            if let Some(params) = match_segments(&record.segments, &path_segments) {
                let spec = route_specificity(&record.segments);
                let is_better = match &best_match {
                    None => true,
                    Some((existing_spec, _)) => spec > *existing_spec,
                };
                if is_better {
                    best_match = Some((
                        spec,
                        Route {
                            raw: join_raw_path(&path, &query),
                            path: path.clone(),
                            pattern: record.pattern.clone(),
                            name: record.name.clone(),
                            params,
                            query: query.clone(),
                        },
                    ));
                }
            }
        }

        // If no registered pattern matched, try the fallback.
        if best_match.is_none() {
            if let Some(fallback) = self.inner.fallback.borrow().as_ref() {
                if let Some(params) = match_segments(&fallback.segments, &path_segments) {
                    best_match = Some((
                        route_specificity(&fallback.segments),
                        Route {
                            raw: join_raw_path(&path, &query),
                            path: path.clone(),
                            pattern: fallback.pattern.clone(),
                            name: fallback.name.clone(),
                            params,
                            query: query.clone(),
                        },
                    ));
                }
            }
        }

        best_match
            .map(|(_, route)| route)
            .ok_or(RouteError::UnknownRoute(raw_path))
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

    pub fn stack_len(&self) -> usize {
        self.inner.stack.borrow().len()
    }

    pub fn can_go_back(&self) -> bool {
        self.stack_len() > 1
    }

    pub fn stack(&self) -> Vec<Route> {
        self.inner.stack.borrow().clone()
    }

    pub fn push(&self, raw_path: impl Into<String>) -> Result<Route, RouteError> {
        let raw_path = raw_path.into();
        self.run_guards(&raw_path)?;
        let route = self.resolve(&raw_path)?;
        let prev = self.current_route();
        let prev_len = self.stack_len();
        self.inner.stack.borrow_mut().push(route.clone());
        self.notify(route.clone());
        self.notify_transition(RouteTransitionEvent::new(
            prev,
            route.clone(),
            RouteTransitionDirection::Forward,
        ));
        Ok(route)
    }

    pub fn replace(&self, raw_path: impl Into<String>) -> Result<Route, RouteError> {
        let raw_path = raw_path.into();
        self.run_guards(&raw_path)?;
        let route = self.resolve(&raw_path)?;
        let prev = self.current_route();
        let mut stack = self.inner.stack.borrow_mut();
        if let Some(last) = stack.last_mut() {
            *last = route.clone();
        } else {
            stack.push(route.clone());
        }
        drop(stack);
        self.notify(route.clone());
        self.notify_transition(RouteTransitionEvent::new(
            prev,
            route.clone(),
            RouteTransitionDirection::Replace,
        ));
        Ok(route)
    }

    pub fn reset(&self, raw_path: impl Into<String>) -> Result<Route, RouteError> {
        let raw_path = raw_path.into();
        self.run_guards(&raw_path)?;
        let route = self.resolve(&raw_path)?;
        let prev = self.current_route();
        let mut stack = self.inner.stack.borrow_mut();
        stack.clear();
        stack.push(route.clone());
        drop(stack);
        self.notify(route.clone());
        self.notify_transition(RouteTransitionEvent::new(
            prev,
            route.clone(),
            RouteTransitionDirection::Replace,
        ));
        Ok(route)
    }

    pub fn back(&self) -> bool {
        let prev = self.current_route();
        let mut stack = self.inner.stack.borrow_mut();
        if stack.len() <= 1 {
            return false;
        }
        stack.pop();
        let current = stack
            .last()
            .cloned()
            .expect("router stack always has at least one route after pop");
        drop(stack);
        self.notify(current.clone());
        self.notify_transition(RouteTransitionEvent::new(
            prev,
            current,
            RouteTransitionDirection::Backward,
        ));
        true
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

    fn register_definition(&self, definition: RouteDefinition) -> Result<bool, RouteError> {
        if self.is_registered(definition.pattern()) {
            return Ok(false);
        }

        let pattern = definition.pattern;
        let name = definition.name;
        let segments = parse_pattern_segments(&pattern)?;
        self.inner.definitions.borrow_mut().push(RouteRecord {
            pattern,
            name,
            segments,
        });
        Ok(true)
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

    /// Register a navigation guard. Guards are called before navigation.
    /// If any guard returns `Some(reason)`, the navigation is blocked.
    /// Returns a guard ID for removal.
    pub fn add_guard(&self, guard: impl Fn(&str) -> Option<String> + 'static) -> usize {
        let id = self.inner.next_guard_id.get();
        self.inner.next_guard_id.set(id + 1);
        self.inner
            .guards
            .borrow_mut()
            .push((id, Rc::new(guard) as Rc<dyn Fn(&str) -> Option<String>>));
        id
    }

    /// Remove a previously registered guard.
    pub fn remove_guard(&self, id: usize) -> bool {
        let mut guards = self.inner.guards.borrow_mut();
        let before = guards.len();
        guards.retain(|(guard_id, _)| *guard_id != id);
        before != guards.len()
    }

    /// Run all guards against a target path. Returns `Err` if any guard blocks.
    fn run_guards(&self, raw_path: &str) -> Result<(), RouteError> {
        let guards = self.inner.guards.borrow();
        for (_, guard) in guards.iter() {
            if let Some(reason) = guard(raw_path) {
                return Err(RouteError::GuardBlocked(reason));
            }
        }
        Ok(())
    }

    /// Subscribe to route transition events (receives from, to, direction).
    /// Returns a subscription ID for removal.
    pub fn subscribe_transition(
        &self,
        callback: impl Fn(RouteTransitionEvent) + 'static,
    ) -> usize {
        let id = self.inner.next_transition_observer_id.get();
        self.inner.next_transition_observer_id.set(id + 1);
        self.inner.transition_observers.borrow_mut().push((
            id,
            Rc::new(callback) as Rc<dyn Fn(RouteTransitionEvent)>,
        ));
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
}

/// A segment match within a nested route resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteSegmentMatch {
    pattern: String,
    name: Option<String>,
    params: BTreeMap<String, String>,
}

impl RouteSegmentMatch {
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn params(&self) -> &BTreeMap<String, String> {
        &self.params
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(String::as_str)
    }
}

/// The result of resolving a path against a nested route tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRoute {
    raw: String,
    path: String,
    query: BTreeMap<String, String>,
    segments: Vec<RouteSegmentMatch>,
}

impl ResolvedRoute {
    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query_params(&self) -> &BTreeMap<String, String> {
        &self.query
    }

    pub fn query(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(String::as_str)
    }

    /// The matched segments from root to leaf.
    pub fn segments(&self) -> &[RouteSegmentMatch] {
        &self.segments
    }

    pub fn depth(&self) -> usize {
        self.segments.len()
    }
}

/// A node in a nested route tree.
#[derive(Debug, Clone)]
pub struct RouteNode {
    pattern: String,
    name: Option<String>,
    children: Vec<RouteNode>,
}

impl RouteNode {
    pub fn new(pattern: impl Into<String>) -> Result<Self, RouteError> {
        let pattern = normalize_path(pattern.into())?;
        parse_pattern_segments(&pattern)?;
        Ok(Self {
            pattern,
            name: None,
            children: Vec::new(),
        })
    }

    pub fn named(name: impl Into<String>, pattern: impl Into<String>) -> Result<Self, RouteError> {
        let mut node = Self::new(pattern)?;
        node.name = Some(name.into());
        Ok(node)
    }

    pub fn child(mut self, child: RouteNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: Vec<RouteNode>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Internal tree node for nested route matching.
#[derive(Debug, Clone)]
struct RouteTreeNode {
    pattern: String,
    name: Option<String>,
    segments: Vec<RouteSegment>,
    children: Vec<RouteTreeNode>,
}

impl RouteTreeNode {
    fn from_route_node(node: RouteNode) -> Result<Self, RouteError> {
        let segments = parse_pattern_segments(&node.pattern)?;
        let child_prefix = &node.pattern;
        let children = node
            .children
            .into_iter()
            .map(|child| Self::from_route_node_with_prefix(child, child_prefix))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            pattern: node.pattern,
            name: node.name,
            segments,
            children,
        })
    }

    fn from_route_node_with_prefix(node: RouteNode, parent_pattern: &str) -> Result<Self, RouteError> {
        let segments = parse_pattern_segments(&node.pattern)?;
        let relative_segments = if node.pattern.starts_with(parent_pattern) && parent_pattern != "/" {
            let relative = &node.pattern[parent_pattern.len()..];
            if relative.is_empty() {
                Vec::new()
            } else {
                parse_pattern_segments(relative)?
            }
        } else if parent_pattern == "/" {
            segments.clone()
        } else {
            segments.clone()
        };
        let child_prefix = &node.pattern;
        let children = node
            .children
            .into_iter()
            .map(|child| Self::from_route_node_with_prefix(child, child_prefix))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            pattern: node.pattern,
            name: node.name,
            segments: relative_segments,
            children,
        })
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new("/")
    }
}

thread_local! {
    static GLOBAL_ROUTER: RefCell<Router> = RefCell::new(Router::new("/"));
}

pub fn global_router() -> Router {
    GLOBAL_ROUTER.with(|state| state.borrow().clone())
}

pub fn replace_global_router(router: Router) {
    GLOBAL_ROUTER.with(|state| {
        state.replace(router);
    });
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

/// Recursively match a path against a route tree node.
/// Returns `Some(Vec<RouteSegmentMatch>)` if the tree matches (root to leaf),
/// or `None` if it doesn't match.
fn resolve_tree(
    node: &RouteTreeNode,
    path_segs: &[&str],
) -> Option<Vec<RouteSegmentMatch>> {
    // Match the current node's segments against the prefix of path_segs.
    let own_len = node.segments.len();
    let own_params = match_segments(&node.segments, &path_segs[..own_len.min(path_segs.len())])?;

    // If this node's segments don't cover the full path_segs prefix, it's a mismatch.
    if own_len > path_segs.len() {
        return None;
    }

    let remaining = &path_segs[own_len..];
    let seg_match = RouteSegmentMatch {
        pattern: node.pattern.clone(),
        name: node.name.clone(),
        params: own_params,
    };

    // Leaf node: remaining must be empty for a full match.
    if node.children.is_empty() {
        if remaining.is_empty() {
            return Some(vec![seg_match]);
        }
        return None;
    }

    // Try each child with specificity-based priority at this sibling level.
    let mut best: Option<(Vec<u8>, Vec<RouteSegmentMatch>)> = None;
    for child in &node.children {
        if let Some(child_segments) = resolve_tree(child, remaining) {
            let spec = route_specificity(&child.segments);
            let is_better = match &best {
                None => true,
                Some((existing_spec, _)) => spec > *existing_spec,
            };
            if is_better {
                let mut result = vec![seg_match.clone()];
                result.extend(child_segments);
                best = Some((spec, result));
            }
        }
    }

    best.map(|(_, segs)| segs)
}

#[cfg(test)]
mod tests {
    use super::{global_router, replace_global_router, RouteDefinition, Router};

    #[test]
    fn match_and_extract_route_params_and_query() {
        let router = Router::new("/");
        router.register("/users/:id").expect("register route");
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
        router.register("/").expect("register route");
        router.register("/about").expect("register route");
        router.push("/about").expect("push route");
        assert_eq!(router.stack_len(), 2);
        assert_eq!(router.current_path(), "/about");

        router.replace("/").expect("replace route");
        assert_eq!(router.current_path(), "/");

        assert!(router.back());
        assert_eq!(router.stack_len(), 1);
        assert!(!router.back());

        router.reset("/about").expect("reset route");
        assert_eq!(router.stack_len(), 1);
        assert_eq!(router.current_path(), "/about");
    }

    #[test]
    fn register_definitions_works() {
        let router = Router::new("/");
        router
            .register_definitions(vec![
                RouteDefinition::new("/").expect("definition"),
                RouteDefinition::named("detail", "/detail/:id").expect("definition"),
            ])
            .expect("register definitions");

        let detail = router.resolve("/detail/9").expect("resolve");
        assert_eq!(detail.name(), Some("detail"));
        assert_eq!(detail.param("id"), Some("9"));
    }

    #[test]
    fn global_router_replace_isolated() {
        let router = Router::new("/home");
        replace_global_router(router.clone());
        assert_eq!(global_router().current_path(), "/home");
    }

    #[test]
    fn specificity_static_beats_param() {
        let router = Router::new("/");
        // Register param route first, static route second
        router.register("/users/:id").expect("register param");
        router.register("/users/me").expect("register static");

        let route = router.resolve("/users/me").expect("resolve");
        assert_eq!(route.pattern(), "/users/me");

        let route2 = router.resolve("/users/42").expect("resolve param");
        assert_eq!(route2.pattern(), "/users/:id");
        assert_eq!(route2.param("id"), Some("42"));
    }

    #[test]
    fn specificity_param_beats_wildcard() {
        let router = Router::new("/");
        router.register("/files/*rest").expect("register wildcard");
        router.register("/files/:id").expect("register param");

        let route = router.resolve("/files/42").expect("resolve");
        assert_eq!(route.pattern(), "/files/:id");

        let route2 = router.resolve("/files/a/b/c").expect("resolve wildcard");
        assert_eq!(route2.pattern(), "/files/*rest");
    }

    #[test]
    fn fallback_matches_unregistered_paths() {
        let router = Router::new("/");
        router.register("/").expect("register home");
        router.register("/about").expect("register about");
        router.register_fallback("*rest").expect("register fallback");

        // Known route matches normally
        let about = router.resolve("/about").expect("resolve");
        assert_eq!(about.pattern(), "/about");

        // Unknown route falls back to the catch-all
        let unknown = router.resolve("/nonexistent/page").expect("fallback");
        assert_eq!(unknown.pattern(), "/*rest");
        assert_eq!(unknown.param("rest"), Some("nonexistent/page"));
    }

    #[test]
    fn nested_route_tree_basic() {
        use super::{RouteNode, RouteSegmentMatch};

        let router = Router::new("/");
        let tree = RouteNode::new("/")
            .expect("root")
            .child(RouteNode::new("/about").expect("about"))
            .child(RouteNode::new("/users/:id").expect("user"));
        router.register_tree(tree).expect("register tree");

        let resolved = router.resolve_nested("/about").expect("resolve nested");
        assert_eq!(resolved.depth(), 2);
        assert_eq!(resolved.segments()[0].pattern(), "/");
        assert_eq!(resolved.segments()[1].pattern(), "/about");
    }

    #[test]
    fn nested_route_tree_params() {
        use super::RouteNode;

        let router = Router::new("/");
        let tree = RouteNode::new("/")
            .expect("root")
            .child(RouteNode::new("/users/:id").expect("user"));
        router.register_tree(tree).expect("register tree");

        let resolved = router.resolve_nested("/users/42").expect("resolve nested");
        assert_eq!(resolved.depth(), 2);
        assert_eq!(resolved.segments()[1].param("id"), Some("42"));
    }

    #[test]
    fn nested_route_tree_deep() {
        use super::RouteNode;

        let router = Router::new("/");
        let tree = RouteNode::new("/")
            .expect("root")
            .child(
                RouteNode::new("/users")
                    .expect("users")
                    .child(RouteNode::named("detail", "/users/:id").expect("detail")),
            );
        router.register_tree(tree).expect("register tree");

        let resolved = router.resolve_nested("/users/7").expect("resolve nested");
        assert_eq!(resolved.depth(), 3);
        assert_eq!(resolved.segments()[0].pattern(), "/");
        assert_eq!(resolved.segments()[1].pattern(), "/users");
        assert_eq!(resolved.segments()[2].pattern(), "/users/:id");
        assert_eq!(resolved.segments()[2].name(), Some("detail"));
        assert_eq!(resolved.segments()[2].param("id"), Some("7"));
    }
}
