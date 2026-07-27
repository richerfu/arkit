use std::{cell::RefCell, rc::Rc, sync::Arc};

use rustc_hash::{FxHashMap, FxHashSet};

/// Stable identity for one location in the route history.
///
/// Two visits to the same route have different entry IDs. Scroll restoration
/// is therefore tied to a navigation visit instead of a route string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RouteEntryId(u64);

impl RouteEntryId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Return the monotonically increasing value backing this ID.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Absolute offset for an ArkUI scroll viewport, in vp.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollPosition {
    /// Horizontal offset in vp.
    pub x: f32,
    /// Vertical offset in vp.
    pub y: f32,
}

impl ScrollPosition {
    /// Create a finite, non-negative absolute scroll position.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: sanitize_offset(x),
            y: sanitize_offset(y),
        }
    }
}

fn sanitize_offset(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// Stable identity for a named scroll viewport within one route entry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScrollRestorationKey(Arc<str>);

impl ScrollRestorationKey {
    /// Create a restoration key.
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }

    /// Access the key text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ScrollRestorationKey {
    fn from(value: &str) -> Self {
        Self::new(Arc::<str>::from(value))
    }
}

impl From<String> for ScrollRestorationKey {
    fn from(value: String) -> Self {
        Self::new(Arc::<str>::from(value))
    }
}

impl From<Arc<str>> for ScrollRestorationKey {
    fn from(value: Arc<str>) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ScrollScopeId {
    Page,
    Named(ScrollRestorationKey),
}

#[derive(Default)]
struct RouteEntryState {
    scroll_positions: FxHashMap<ScrollScopeId, ScrollPosition>,
}

type ScrollSnapshot = Rc<dyn Fn() -> ScrollPosition>;

struct ActiveScrollSnapshot {
    token: u64,
    snapshot: ScrollSnapshot,
}

#[derive(Default)]
struct PageStateStoreInner {
    entries: FxHashMap<RouteEntryId, RouteEntryState>,
    snapshots: FxHashMap<(RouteEntryId, ScrollScopeId), ActiveScrollSnapshot>,
    active_entries: FxHashSet<RouteEntryId>,
    next_snapshot_token: u64,
}

#[derive(Clone, Default)]
pub(crate) struct PageStateStore {
    inner: Rc<RefCell<PageStateStoreInner>>,
}

impl PageStateStore {
    pub(crate) fn activate_entry(&self, entry_id: RouteEntryId) {
        self.inner.borrow_mut().active_entries.insert(entry_id);
    }

    pub(crate) fn position(
        &self,
        entry_id: RouteEntryId,
        scope: &ScrollScopeId,
    ) -> Option<ScrollPosition> {
        self.inner
            .borrow()
            .entries
            .get(&entry_id)
            .and_then(|entry| entry.scroll_positions.get(scope).copied())
    }

    pub(crate) fn save_position(
        &self,
        entry_id: RouteEntryId,
        scope: ScrollScopeId,
        position: ScrollPosition,
    ) {
        let mut inner = self.inner.borrow_mut();
        if !inner.active_entries.contains(&entry_id) {
            return;
        }
        inner
            .entries
            .entry(entry_id)
            .or_default()
            .scroll_positions
            .insert(scope, position);
    }

    pub(crate) fn register_scroll_snapshot(
        &self,
        entry_id: RouteEntryId,
        scope: ScrollScopeId,
        snapshot: ScrollSnapshot,
    ) -> u64 {
        let mut inner = self.inner.borrow_mut();
        let token = inner.next_snapshot_token;
        inner.next_snapshot_token = inner
            .next_snapshot_token
            .checked_add(1)
            .expect("arkit_router: scroll snapshot token space exhausted");
        inner
            .snapshots
            .insert((entry_id, scope), ActiveScrollSnapshot { token, snapshot });
        token
    }

    pub(crate) fn unregister_scroll_snapshot(
        &self,
        entry_id: RouteEntryId,
        scope: &ScrollScopeId,
        token: u64,
    ) {
        let key = (entry_id, scope.clone());
        let mut inner = self.inner.borrow_mut();
        if inner
            .snapshots
            .get(&key)
            .is_some_and(|snapshot| snapshot.token == token)
        {
            inner.snapshots.remove(&key);
        }
    }

    /// Snapshot all mounted scroll viewports before the route starts changing.
    pub(crate) fn snapshot_entry(&self, entry_id: RouteEntryId) {
        let snapshots = self
            .inner
            .borrow()
            .snapshots
            .iter()
            .filter(|((candidate, _), _)| *candidate == entry_id)
            .map(|((_, scope), active)| (scope.clone(), active.snapshot.clone()))
            .collect::<Vec<_>>();

        for (scope, snapshot) in snapshots {
            self.save_position(entry_id, scope, snapshot());
        }
    }

    pub(crate) fn remove_entry(&self, entry_id: RouteEntryId) {
        let mut inner = self.inner.borrow_mut();
        inner.entries.remove(&entry_id);
        inner.active_entries.remove(&entry_id);
        inner
            .snapshots
            .retain(|(candidate, _), _| *candidate != entry_id);
    }

    pub(crate) fn remove_entries(&self, entry_ids: impl IntoIterator<Item = RouteEntryId>) {
        for entry_id in entry_ids {
            self.remove_entry(entry_id);
        }
    }

    #[cfg(test)]
    pub(crate) fn contains_entry(&self, entry_id: RouteEntryId) -> bool {
        self.inner.borrow().entries.contains_key(&entry_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_positions_are_sanitized_before_storage() {
        assert_eq!(
            ScrollPosition::new(f32::NAN, -12.0),
            ScrollPosition { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            ScrollPosition::new(12.5, 24.25),
            ScrollPosition { x: 12.5, y: 24.25 }
        );
    }

    #[test]
    fn page_and_named_scroll_positions_do_not_collide() {
        let store = PageStateStore::default();
        let entry = RouteEntryId::new(7);
        let page = ScrollScopeId::Page;
        let nested = ScrollScopeId::Named("nested".into());
        store.activate_entry(entry);

        store.save_position(entry, page.clone(), ScrollPosition::new(0.0, 40.0));
        store.save_position(entry, nested.clone(), ScrollPosition::new(0.0, 120.0));

        assert_eq!(
            store.position(entry, &page),
            Some(ScrollPosition::new(0.0, 40.0))
        );
        assert_eq!(
            store.position(entry, &nested),
            Some(ScrollPosition::new(0.0, 120.0))
        );
    }

    #[test]
    fn navigation_snapshot_uses_the_latest_mounted_registration() {
        let store = PageStateStore::default();
        let entry = RouteEntryId::new(7);
        let scope = ScrollScopeId::Page;
        store.activate_entry(entry);

        let stale_token = store.register_scroll_snapshot(
            entry,
            scope.clone(),
            Rc::new(|| ScrollPosition::new(0.0, 40.0)),
        );
        let current_token = store.register_scroll_snapshot(
            entry,
            scope.clone(),
            Rc::new(|| ScrollPosition::new(0.0, 120.0)),
        );
        store.unregister_scroll_snapshot(entry, &scope, stale_token);
        store.snapshot_entry(entry);

        assert_eq!(
            store.position(entry, &scope),
            Some(ScrollPosition::new(0.0, 120.0))
        );

        store.unregister_scroll_snapshot(entry, &scope, current_token);
    }

    #[test]
    fn retired_history_entries_cannot_be_recreated_by_late_drop() {
        let store = PageStateStore::default();
        let entry = RouteEntryId::new(7);
        let scope = ScrollScopeId::Page;
        store.activate_entry(entry);
        store.save_position(entry, scope.clone(), ScrollPosition::new(0.0, 40.0));

        store.remove_entry(entry);
        store.save_position(entry, scope.clone(), ScrollPosition::new(0.0, 120.0));

        assert_eq!(store.position(entry, &scope), None);
    }
}
