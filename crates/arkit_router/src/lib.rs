use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteError {
    EmptyPath,
    InvalidPattern(String),
    UnknownRoute(String),
}

impl Display for RouteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteError::EmptyPath => write!(f, "route path cannot be empty"),
            RouteError::InvalidPattern(pattern) => {
                write!(f, "invalid route pattern: {pattern}")
            }
            RouteError::UnknownRoute(path) => write!(f, "route is not registered: {path}"),
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
    stack: RefCell<Vec<Route>>,
    observers: RefCell<Vec<(usize, Rc<dyn Fn(Route)>)>>,
    next_observer_id: Cell<usize>,
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
                stack: RefCell::new(vec![route]),
                observers: RefCell::new(Vec::new()),
                next_observer_id: Cell::new(1),
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
        for record in records.iter() {
            if let Some(params) = match_segments(&record.segments, &path_segments) {
                return Ok(Route {
                    raw: join_raw_path(&path, &query),
                    path: path.clone(),
                    pattern: record.pattern.clone(),
                    name: record.name.clone(),
                    params,
                    query,
                });
            }
        }

        Err(RouteError::UnknownRoute(raw_path))
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
        let route = self.resolve(raw_path)?;
        self.inner.stack.borrow_mut().push(route.clone());
        self.notify(route.clone());
        Ok(route)
    }

    pub fn replace(&self, raw_path: impl Into<String>) -> Result<Route, RouteError> {
        let route = self.resolve(raw_path)?;
        let mut stack = self.inner.stack.borrow_mut();
        if let Some(last) = stack.last_mut() {
            *last = route.clone();
        } else {
            stack.push(route.clone());
        }
        drop(stack);
        self.notify(route.clone());
        Ok(route)
    }

    pub fn reset(&self, raw_path: impl Into<String>) -> Result<Route, RouteError> {
        let route = self.resolve(raw_path)?;
        let mut stack = self.inner.stack.borrow_mut();
        stack.clear();
        stack.push(route.clone());
        drop(stack);
        self.notify(route.clone());
        Ok(route)
    }

    pub fn back(&self) -> bool {
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
        self.notify(current);
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

fn join_raw_path(path: &str, query: &BTreeMap<String, String>) -> String {
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
}
