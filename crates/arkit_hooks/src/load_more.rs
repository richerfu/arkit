//! Shared load-more state and trigger control for regular and virtual scrolling.
//!
//! A regular ArkUI `Scroll` calls [`LoadMoreController::reach_end`]. A virtual
//! `List`/`WaterFlow` can forward its typed `on_scroll` payload to
//! [`LoadMoreController::on_virtual_scroll`] or notify the index requested by
//! its NodeAdapter through [`LoadMoreController::on_virtual_item`]. All paths
//! share the same request gate, so a burst of native events cannot request the
//! same data page more than once.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arkit_prelude::dioxus_elements::event::ScrollData;
use arkit_prelude::{use_drop, use_hook, EventHandler};

/// Externally controlled state for an incremental data source.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadMoreState {
    /// More data can be requested.
    #[default]
    Idle,
    /// A request is currently in flight.
    Loading,
    /// The last request failed and may be retried explicitly.
    Failed,
    /// The data source has no more items.
    NoMore,
}

#[derive(Debug, Default)]
struct LoadMoreGate {
    requested_item_count: Cell<Option<u32>>,
    observed_state: Cell<LoadMoreState>,
    retry_in_flight: Cell<bool>,
}

impl LoadMoreGate {
    fn observe(&self, state: LoadMoreState) {
        let previous = self.observed_state.replace(state);
        if previous != state && state == LoadMoreState::Failed {
            self.retry_in_flight.set(false);
        }
    }

    fn try_request(&self, item_count: u32, state: LoadMoreState) -> bool {
        if state != LoadMoreState::Idle || self.requested_item_count.get() == Some(item_count) {
            return false;
        }
        self.requested_item_count.set(Some(item_count));
        true
    }

    fn try_retry(&self, item_count: u32, state: LoadMoreState) -> bool {
        if state != LoadMoreState::Failed || self.retry_in_flight.replace(true) {
            return false;
        }
        self.requested_item_count.set(Some(item_count));
        true
    }

    fn reset(&self) {
        self.requested_item_count.set(None);
        self.retry_in_flight.set(false);
    }
}

/// One load-more trigger shared by regular and virtual scroll containers.
///
/// The controller is controlled by [`LoadMoreState`]. After requesting at a
/// given `item_count`, it remains latched until the count changes or
/// [`Self::reset`] is called. This prevents duplicate requests during the
/// render between an event and the caller switching to `Loading`. Retained
/// clones read the latest render's count, state, preload window, and callback;
/// all clones become inert when the owning hook unmounts.
#[derive(Clone)]
pub struct LoadMoreController {
    shared: Rc<LoadMoreShared>,
}

type LoadMoreCallback = Rc<dyn Fn()>;

#[derive(Clone)]
struct LoadMoreSnapshot {
    item_count: u32,
    state: LoadMoreState,
    preload_items: u32,
    on_load_more: Option<LoadMoreCallback>,
}

struct LoadMoreShared {
    gate: LoadMoreGate,
    latest: RefCell<LoadMoreSnapshot>,
}

impl LoadMoreShared {
    fn new(
        item_count: u32,
        state: LoadMoreState,
        preload_items: u32,
        on_load_more: LoadMoreCallback,
    ) -> Self {
        let gate = LoadMoreGate::default();
        gate.observe(state);
        Self {
            gate,
            latest: RefCell::new(LoadMoreSnapshot {
                item_count,
                state,
                preload_items,
                on_load_more: Some(on_load_more),
            }),
        }
    }

    fn update(
        &self,
        item_count: u32,
        state: LoadMoreState,
        preload_items: u32,
        on_load_more: LoadMoreCallback,
    ) {
        self.gate.observe(state);
        *self.latest.borrow_mut() = LoadMoreSnapshot {
            item_count,
            state,
            preload_items,
            on_load_more: Some(on_load_more),
        };
    }

    fn snapshot(&self) -> Option<LoadMoreSnapshot> {
        let snapshot = self.latest.borrow();
        snapshot.on_load_more.as_ref()?;
        Some(snapshot.clone())
    }

    fn deactivate(&self) {
        self.latest.borrow_mut().on_load_more = None;
    }
}

impl LoadMoreController {
    /// Handle ArkUI `ScrollEventOnReachEnd` for a regular scroll container.
    pub fn reach_end(&self) {
        self.request_if_ready();
    }

    /// Handle a virtual List/WaterFlow `on_scroll` visible-index event.
    ///
    /// Loading starts when the last visible data item enters the configured
    /// preload window. Offset-only events and empty data sets are ignored.
    pub fn on_virtual_scroll(&self, data: ScrollData) {
        let Some(snapshot) = self.shared.snapshot() else {
            return;
        };
        if should_request_from_virtual_range(data, snapshot.item_count, snapshot.preload_items) {
            self.request_snapshot(snapshot);
        }
    }

    /// Handle one item index requested by a virtual NodeAdapter.
    ///
    /// This is the reliable fallback for containers or platform versions that
    /// do not emit visible-range events for adapter-backed content. Call it
    /// outside the native adapter callback (for example through the
    /// framework's UI-loop queue) to avoid renderer re-entry.
    pub fn on_virtual_item(&self, index: u32) {
        let Some(snapshot) = self.shared.snapshot() else {
            return;
        };
        if should_request_from_virtual_index(index, snapshot.item_count, snapshot.preload_items) {
            self.request_snapshot(snapshot);
        }
    }

    /// Retry a failed request. Other states deliberately ignore this call.
    pub fn retry(&self) {
        let Some(snapshot) = self.shared.snapshot() else {
            return;
        };
        if self
            .shared
            .gate
            .try_retry(snapshot.item_count, snapshot.state)
        {
            snapshot
                .on_load_more
                .expect("active load-more snapshot lost its callback")();
        }
    }

    /// Re-arm the current item count after replacing or refreshing the data.
    pub fn reset(&self) {
        self.shared.gate.reset();
    }

    fn request_if_ready(&self) {
        if let Some(snapshot) = self.shared.snapshot() {
            self.request_snapshot(snapshot);
        }
    }

    fn request_snapshot(&self, snapshot: LoadMoreSnapshot) {
        if self
            .shared
            .gate
            .try_request(snapshot.item_count, snapshot.state)
        {
            snapshot
                .on_load_more
                .expect("active load-more snapshot lost its callback")();
        }
    }
}

/// Create a load-more controller that works with regular and virtual scrolling.
///
/// `preload_items` only affects [`LoadMoreController::on_virtual_scroll`].
/// Pass `0` to wait until the final data item is visible.
#[track_caller]
pub fn use_load_more(
    item_count: u32,
    state: LoadMoreState,
    preload_items: u32,
    on_load_more: EventHandler<()>,
) -> LoadMoreController {
    let callback: LoadMoreCallback = Rc::new(move || on_load_more.call(()));
    let initial_callback = callback.clone();
    let shared = use_hook(move || {
        Rc::new(LoadMoreShared::new(
            item_count,
            state,
            preload_items,
            initial_callback,
        ))
    });
    shared.update(item_count, state, preload_items, callback);
    let cleanup = shared.clone();
    use_drop(move || cleanup.deactivate());
    LoadMoreController { shared }
}

fn should_request_from_virtual_range(
    data: ScrollData,
    item_count: u32,
    preload_items: u32,
) -> bool {
    if data.has_offset || data.last_index < 0 || item_count == 0 {
        return false;
    }
    let trigger_index = item_count.saturating_sub(preload_items.saturating_add(1));
    u32::try_from(data.last_index).is_ok_and(|last| last >= trigger_index)
}

fn should_request_from_virtual_index(index: u32, item_count: u32, preload_items: u32) -> bool {
    if item_count == 0 || index >= item_count {
        return false;
    }
    let trigger_index = item_count.saturating_sub(preload_items.saturating_add(1));
    index >= trigger_index
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visible(last_index: i32) -> ScrollData {
        ScrollData {
            last_index,
            ..ScrollData::default()
        }
    }

    #[test]
    fn virtual_range_uses_preload_window() {
        assert!(!should_request_from_virtual_range(visible(6), 10, 2));
        assert!(should_request_from_virtual_range(visible(7), 10, 2));
        assert!(should_request_from_virtual_range(visible(9), 10, 0));
    }

    #[test]
    fn virtual_range_rejects_offsets_empty_data_and_negative_indices() {
        assert!(!should_request_from_virtual_range(
            ScrollData {
                has_offset: true,
                last_index: 99,
                ..ScrollData::default()
            },
            100,
            2,
        ));
        assert!(!should_request_from_virtual_range(visible(-1), 10, 2));
        assert!(!should_request_from_virtual_range(visible(0), 0, 2));
    }

    #[test]
    fn virtual_item_requests_use_the_same_preload_window() {
        assert!(!should_request_from_virtual_index(6, 10, 2));
        assert!(should_request_from_virtual_index(7, 10, 2));
        assert!(should_request_from_virtual_index(9, 10, 0));
        assert!(!should_request_from_virtual_index(10, 10, 2));
        assert!(!should_request_from_virtual_index(0, 0, 2));
    }

    #[test]
    fn gate_suppresses_duplicate_requests_for_the_same_data_page() {
        let gate = LoadMoreGate::default();
        assert!(gate.try_request(20, LoadMoreState::Idle));
        assert!(!gate.try_request(20, LoadMoreState::Idle));
        assert!(!gate.try_request(21, LoadMoreState::Loading));
        assert!(gate.try_request(21, LoadMoreState::Idle));
    }

    #[test]
    fn reset_rearms_an_unchanged_data_page() {
        let gate = LoadMoreGate::default();
        assert!(gate.try_request(20, LoadMoreState::Idle));
        gate.reset();
        assert!(gate.try_request(20, LoadMoreState::Idle));
    }

    #[test]
    fn retry_is_latched_until_state_leaves_and_reenters_failed() {
        let gate = LoadMoreGate::default();
        gate.observe(LoadMoreState::Failed);
        assert!(gate.try_retry(20, LoadMoreState::Failed));
        assert!(!gate.try_retry(20, LoadMoreState::Failed));
        gate.observe(LoadMoreState::Loading);
        gate.observe(LoadMoreState::Failed);
        assert!(gate.try_retry(20, LoadMoreState::Failed));
    }

    #[test]
    fn retained_controller_reads_latest_count_state_and_callback() {
        let first_calls = Rc::new(Cell::new(0));
        let count_first = first_calls.clone();
        let shared = Rc::new(LoadMoreShared::new(
            10,
            LoadMoreState::Idle,
            0,
            Rc::new(move || count_first.set(count_first.get() + 1)),
        ));
        let retained = LoadMoreController {
            shared: shared.clone(),
        };
        let latest_calls = Rc::new(Cell::new(0));
        let count_latest = latest_calls.clone();

        shared.update(
            20,
            LoadMoreState::Loading,
            0,
            Rc::new(move || count_latest.set(count_latest.get() + 1)),
        );
        retained.on_virtual_item(19);
        shared.update(20, LoadMoreState::NoMore, 0, Rc::new(|| {}));
        retained.on_virtual_item(19);
        let count_latest = latest_calls.clone();
        shared.update(
            20,
            LoadMoreState::Idle,
            0,
            Rc::new(move || count_latest.set(count_latest.get() + 1)),
        );
        retained.on_virtual_item(9);
        retained.on_virtual_item(19);

        assert_eq!(first_calls.get(), 0);
        assert_eq!(latest_calls.get(), 1);
    }

    #[test]
    fn callback_runs_after_the_latest_snapshot_borrow_is_released() {
        let shared = Rc::new(LoadMoreShared::new(
            10,
            LoadMoreState::Idle,
            0,
            Rc::new(|| {}),
        ));
        let deactivate = shared.clone();
        shared.update(
            10,
            LoadMoreState::Idle,
            0,
            Rc::new(move || deactivate.deactivate()),
        );
        let controller = LoadMoreController {
            shared: shared.clone(),
        };

        controller.reach_end();
        assert!(shared.snapshot().is_none());
    }

    #[test]
    fn retained_controller_is_inert_after_owner_deactivation() {
        let calls = Rc::new(Cell::new(0));
        let count_calls = calls.clone();
        let shared = Rc::new(LoadMoreShared::new(
            1,
            LoadMoreState::Idle,
            0,
            Rc::new(move || count_calls.set(count_calls.get() + 1)),
        ));
        let retained = LoadMoreController {
            shared: shared.clone(),
        };

        shared.deactivate();
        retained.reach_end();
        retained.on_virtual_item(0);
        retained.retry();

        assert_eq!(calls.get(), 0);
    }
}
