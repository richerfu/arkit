use std::{cell::RefCell, rc::Rc, sync::Arc};

use dioxus_history::History;

use crate::state::{PageStateStore, RouteEntryId};

#[derive(Clone, Debug)]
struct HistoryEntry {
    id: RouteEntryId,
    route: String,
}

struct RouteHistoryState {
    current: HistoryEntry,
    history: Vec<HistoryEntry>,
    future: Vec<HistoryEntry>,
    next_entry_id: u64,
}

/// History implementation that gives every navigation visit a stable ID.
pub(crate) struct RouteHistory {
    inner: Rc<dyn History>,
    state: RefCell<RouteHistoryState>,
    page_states: PageStateStore,
}

impl RouteHistory {
    pub(crate) fn new(inner: Rc<dyn History>, page_states: PageStateStore) -> Rc<Self> {
        let initial_path = inner.current_route();
        page_states.activate_entry(RouteEntryId::new(0));
        Rc::new(Self {
            inner,
            state: RefCell::new(RouteHistoryState {
                current: HistoryEntry {
                    id: RouteEntryId::new(0),
                    route: initial_path,
                },
                history: Vec::new(),
                future: Vec::new(),
                next_entry_id: 1,
            }),
            page_states,
        })
    }

    pub(crate) fn current_entry_id(&self) -> RouteEntryId {
        self.reconcile_route(self.inner.current_route());
        self.state.borrow().current.id
    }

    fn capture_current_entry(&self) {
        self.page_states
            .snapshot_entry(self.state.borrow().current.id);
    }

    fn next_entry(state: &mut RouteHistoryState, route: String) -> HistoryEntry {
        let id = RouteEntryId::new(state.next_entry_id);
        state.next_entry_id = state
            .next_entry_id
            .checked_add(1)
            .expect("arkit_router: route history entry ID space exhausted");
        HistoryEntry { id, route }
    }

    fn push_entry(&self, route: String) {
        let (current_id, removed_future) = {
            let mut state = self.state.borrow_mut();
            if state.current.route == route {
                return;
            }

            let next = Self::next_entry(&mut state, route);
            let current = std::mem::replace(&mut state.current, next);
            state.history.push(current);
            (
                state.current.id,
                state
                    .future
                    .drain(..)
                    .map(|entry| entry.id)
                    .collect::<Vec<_>>(),
            )
        };
        self.page_states.activate_entry(current_id);
        self.page_states.remove_entries(removed_future);
    }

    fn replace_entry(&self, route: String) {
        let (current_id, removed_entry) = {
            let mut state = self.state.borrow_mut();
            if state.current.route == route {
                return;
            }
            let next = Self::next_entry(&mut state, route);
            let removed = std::mem::replace(&mut state.current, next).id;
            (state.current.id, removed)
        };
        self.page_states.activate_entry(current_id);
        self.page_states.remove_entry(removed_entry);
    }

    /// Reconcile route changes initiated by the platform history (for example,
    /// browser back/forward) with Arkit's entry identities.
    fn reconcile_route(&self, route: String) {
        let (current_id, removed_future) = {
            let mut state = self.state.borrow_mut();
            if state.current.route == route {
                return;
            }

            if state
                .history
                .last()
                .is_some_and(|entry| entry.route == route)
            {
                let previous = state
                    .history
                    .pop()
                    .expect("history entry checked immediately before pop");
                let current = std::mem::replace(&mut state.current, previous);
                state.future.push(current);
                (state.current.id, Vec::new())
            } else if state
                .future
                .last()
                .is_some_and(|entry| entry.route == route)
            {
                let next = state
                    .future
                    .pop()
                    .expect("future entry checked immediately before pop");
                let current = std::mem::replace(&mut state.current, next);
                state.history.push(current);
                (state.current.id, Vec::new())
            } else {
                let next = Self::next_entry(&mut state, route);
                let current = std::mem::replace(&mut state.current, next);
                state.history.push(current);
                (
                    state.current.id,
                    state.future.drain(..).map(|entry| entry.id).collect(),
                )
            }
        };
        self.page_states.activate_entry(current_id);
        self.page_states.remove_entries(removed_future);
    }
}

impl History for RouteHistory {
    fn current_route(&self) -> String {
        let route = self.inner.current_route();
        self.reconcile_route(route.clone());
        route
    }

    fn current_prefix(&self) -> Option<String> {
        self.inner.current_prefix()
    }

    fn can_go_back(&self) -> bool {
        self.inner.can_go_back()
    }

    fn go_back(&self) {
        self.reconcile_route(self.inner.current_route());
        self.capture_current_entry();
        self.inner.go_back();
        self.reconcile_route(self.inner.current_route());
    }

    fn can_go_forward(&self) -> bool {
        self.inner.can_go_forward()
    }

    fn go_forward(&self) {
        self.reconcile_route(self.inner.current_route());
        self.capture_current_entry();
        self.inner.go_forward();
        self.reconcile_route(self.inner.current_route());
    }

    fn push(&self, route: String) {
        self.reconcile_route(self.inner.current_route());
        self.capture_current_entry();
        self.inner.push(route);
        self.push_entry(self.inner.current_route());
    }

    fn replace(&self, route: String) {
        self.reconcile_route(self.inner.current_route());
        self.capture_current_entry();
        self.inner.replace(route);
        self.replace_entry(self.inner.current_route());
    }

    fn external(&self, url: String) -> bool {
        self.capture_current_entry();
        self.inner.external(url)
    }

    fn updater(&self, callback: Arc<dyn Fn() + Send + Sync>) {
        self.inner.updater(callback);
    }

    fn include_prevent_default(&self) -> bool {
        self.inner.include_prevent_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ScrollPosition, ScrollScopeId};
    use dioxus_history::MemoryHistory;
    use std::cell::Cell;

    fn history(store: PageStateStore) -> Rc<RouteHistory> {
        RouteHistory::new(Rc::new(MemoryHistory::default()), store)
    }

    #[test]
    fn back_and_forward_revisit_the_same_entry() {
        let store = PageStateStore::default();
        let history = history(store);
        let home = history.current_entry_id();

        history.push("/detail".to_string());
        let detail = history.current_entry_id();
        assert_ne!(home, detail);

        history.go_back();
        assert_eq!(history.current_entry_id(), home);
        history.go_forward();
        assert_eq!(history.current_entry_id(), detail);
    }

    #[test]
    fn pushing_after_back_evicts_future_page_state() {
        let store = PageStateStore::default();
        let history = history(store.clone());
        history.push("/first".to_string());
        let removed = history.current_entry_id();
        store.save_position(removed, ScrollScopeId::Page, ScrollPosition::new(0.0, 80.0));

        history.go_back();
        history.push("/second".to_string());

        assert!(!store.contains_entry(removed));
    }

    #[test]
    fn replace_gets_fresh_identity_and_evicts_replaced_state() {
        let store = PageStateStore::default();
        let history = history(store.clone());
        let replaced = history.current_entry_id();
        store.save_position(
            replaced,
            ScrollScopeId::Page,
            ScrollPosition::new(0.0, 64.0),
        );

        history.replace("/login".to_string());

        assert_ne!(history.current_entry_id(), replaced);
        assert!(!store.contains_entry(replaced));
        assert!(!history.can_go_back());
    }

    #[test]
    fn pushing_the_current_route_is_a_noop() {
        let store = PageStateStore::default();
        let history = history(store);
        let original = history.current_entry_id();

        history.push("/".to_string());

        assert_eq!(history.current_entry_id(), original);
        assert!(!history.can_go_back());
    }

    #[test]
    fn navigation_snapshots_event_sourced_position_before_route_change() {
        let store = PageStateStore::default();
        let history = history(store.clone());
        let home = history.current_entry_id();
        let position = Rc::new(Cell::new(ScrollPosition::default()));
        let current = position.clone();
        let _token = store.register_scroll_snapshot(
            home,
            ScrollScopeId::Page,
            Rc::new(move || current.get()),
        );

        position.set(ScrollPosition::new(0.0, 240.0));
        history.push("/detail".to_string());

        assert_eq!(
            store.position(home, &ScrollScopeId::Page),
            Some(ScrollPosition::new(0.0, 240.0))
        );
    }

    #[test]
    fn platform_back_navigation_reuses_the_original_entry() {
        let store = PageStateStore::default();
        let platform = Rc::new(MemoryHistory::default());
        let history = RouteHistory::new(platform.clone(), store);
        let home = history.current_entry_id();

        platform.push("/detail".to_string());
        assert_eq!(history.current_route(), "/detail");
        let detail = history.current_entry_id();
        assert_ne!(detail, home);

        platform.go_back();
        assert_eq!(history.current_route(), "/");
        assert_eq!(history.current_entry_id(), home);
    }
}
