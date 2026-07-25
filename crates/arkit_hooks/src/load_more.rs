//! Shared load-more state and trigger control for regular and virtual scrolling.
//!
//! A regular ArkUI `Scroll` calls [`LoadMoreController::reach_end`]. A virtual
//! `List`/`WaterFlow` can forward its typed `on_scroll` payload to
//! [`LoadMoreController::on_virtual_scroll`] or notify the index requested by
//! its NodeAdapter through [`LoadMoreController::on_virtual_item`]. All paths
//! share the same request gate, so a burst of native events cannot request the
//! same data page more than once.

use std::cell::Cell;
use std::rc::Rc;

use arkit_prelude::dioxus_elements::event::ScrollData;
use arkit_prelude::{use_hook, EventHandler};

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
/// render between an event and the caller switching to `Loading`.
#[derive(Clone)]
pub struct LoadMoreController {
    gate: Rc<LoadMoreGate>,
    item_count: u32,
    state: LoadMoreState,
    preload_items: u32,
    on_load_more: EventHandler<()>,
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
        if should_request_from_virtual_range(data, self.item_count, self.preload_items) {
            self.request_if_ready();
        }
    }

    /// Handle one item index requested by a virtual NodeAdapter.
    ///
    /// This is the reliable fallback for containers or platform versions that
    /// do not emit visible-range events for adapter-backed content. Call it
    /// outside the native adapter callback (for example through the
    /// framework's UI-loop queue) to avoid renderer re-entry.
    pub fn on_virtual_item(&self, index: u32) {
        if should_request_from_virtual_index(index, self.item_count, self.preload_items) {
            self.request_if_ready();
        }
    }

    /// Retry a failed request. Other states deliberately ignore this call.
    pub fn retry(&self) {
        if self.gate.try_retry(self.item_count, self.state) {
            self.on_load_more.call(());
        }
    }

    /// Re-arm the current item count after replacing or refreshing the data.
    pub fn reset(&self) {
        self.gate.reset();
    }

    fn request_if_ready(&self) {
        if self.gate.try_request(self.item_count, self.state) {
            self.on_load_more.call(());
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
    let gate = use_hook(|| Rc::new(LoadMoreGate::default()));
    gate.observe(state);
    LoadMoreController {
        gate,
        item_count,
        state,
        preload_items,
        on_load_more,
    }
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
}
