use std::rc::Rc;

use arkit_prelude::*;
use dioxus_core::Callback;
use dioxus_history::History;

use crate::{
    history::RouteHistory,
    state::{PageStateStore, RouteEntryId},
};

#[derive(Clone)]
pub(crate) struct RouteStateContext {
    history: Rc<RouteHistory>,
    pub(crate) page_states: PageStateStore,
}

impl RouteStateContext {
    fn new() -> Self {
        let page_states = PageStateStore::default();
        let history = RouteHistory::new(dioxus_history::history(), page_states.clone());
        Self {
            history,
            page_states,
        }
    }

    pub(crate) fn current_entry_id(&self) -> RouteEntryId {
        self.history.current_entry_id()
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
        // Match dioxus-router: changing initial config does not rebuild a live
        // router or its history stack.
        true
    }
}

/// Arkit router root with history-entry-scoped page state.
///
/// This is API-compatible with the upstream `Router::<Route> {}` component.
/// The root instance wraps the renderer's history implementation so every
/// navigation visit gets a stable identity. Nested routers reuse that root
/// state instead of creating a second history.
#[allow(non_snake_case)]
pub fn Router<R>(props: RouterProps<R>) -> Element
where
    R: dioxus_router::Routable + Clone,
{
    if try_use_context::<RouteStateContext>().is_some() {
        let config = props.config;
        return rsx! {
            dioxus_router::Router::<R> {
                config: move |_| config.call(()),
            }
        };
    }

    let state = use_hook(RouteStateContext::new);
    use_context_provider(|| state.clone());

    let history = state.history.clone();
    let config = props.config;
    rsx! {
        dioxus_router::components::HistoryProvider {
            history: move |_| history.clone() as Rc<dyn History>,
            dioxus_router::Router::<R> {
                config: move |_| config.call(()),
            }
        }
    }
}

/// Return the identity of the currently active route history entry.
///
/// The ID is stable while moving backward and forward through the same
/// history visit. A new push or replace receives a new ID.
///
/// # Panics
///
/// Panics when called outside Arkit's [`Router`].
pub fn use_route_entry_id() -> RouteEntryId {
    use_context::<RouteStateContext>().current_entry_id()
}
