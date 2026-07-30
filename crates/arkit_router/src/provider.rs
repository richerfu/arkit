use std::{cell::RefCell, rc::Rc};

use arkit_prelude::*;
use dioxus_core::Callback;
use rustc_hash::FxHashMap;

/// Scroll state owned by one Arkit router instance.
///
/// Positions are keyed by the router's complete route string. A mounted page
/// takes its saved position and writes it back when the page unmounts.
#[derive(Clone, Default)]
pub(crate) struct RouteScrollStore {
    positions: Rc<RefCell<FxHashMap<String, f32>>>,
}

impl RouteScrollStore {
    pub(crate) fn take(&self, route: &str) -> Option<f32> {
        self.positions.borrow_mut().remove(route)
    }

    pub(crate) fn save(&self, route: String, position: f32) {
        self.positions.borrow_mut().insert(route, position);
    }
}

/// Props for [`Router`].
#[derive(Props)]
pub struct RouterProps<R: Clone + 'static> {
    /// Build the upstream Dioxus router configuration.
    #[props(default, into)]
    pub config: Callback<(), dioxus_router::RouterConfig<R>>,
}

impl<R: Clone> Clone for RouterProps<R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Clone> Copy for RouterProps<R> {}

impl<R: Clone + 'static> Default for RouterProps<R> {
    fn default() -> Self {
        Self {
            config: Callback::new(|_| dioxus_router::RouterConfig::default()),
        }
    }
}

impl<R: Clone> PartialEq for RouterProps<R> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Arkit router root with default page-scroll restoration.
///
/// This is API-compatible with the upstream `Router::<Route> {}` component.
/// It adds only a router-scoped position store consumed by [`crate::RouteProvider`].
#[allow(non_snake_case)]
pub fn Router<R>(props: RouterProps<R>) -> Element
where
    R: dioxus_router::Routable + Clone,
{
    let scroll = use_hook(RouteScrollStore::default);
    use_context_provider(|| scroll);

    let config = props.config;
    rsx! {
        dioxus_router::Router::<R> {
            config: move |_| config.call(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_position_is_consumed_once_and_can_be_saved_again() {
        let store = RouteScrollStore::default();
        let position = 120.0;
        store.save("/components/button".to_string(), position);

        assert_eq!(store.take("/components/button"), Some(position));
        assert_eq!(store.take("/components/button"), None);

        store.save("/components/button".to_string(), position);
        assert_eq!(store.take("/components/button"), Some(position));
    }
}
