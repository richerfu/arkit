//! Hover/focus handoff between a trigger and its projected panel.

use arkit_prelude::*;

const CLOSE_DELAY_MS: u64 = 150;

#[derive(Clone, Copy, Default)]
struct HoverState {
    trigger_hover: bool,
    panel_hover: bool,
    trigger_focus: bool,
    panel_focus: bool,
    pinned: bool,
    dismissed: bool,
    suppress_return_focus: bool,
    revision: u64,
}

impl HoverState {
    fn holds_open(self) -> bool {
        !self.dismissed
            && (self.trigger_hover
                || self.panel_hover
                || self.trigger_focus
                || self.panel_focus
                || self.pinned)
    }
}

#[derive(Clone)]
pub(crate) struct HoverSurface {
    state: Signal<HoverState>,
    on_open_change: EventHandler<bool>,
    runtime: tokio::runtime::Handle,
}

pub(crate) fn use_hover_surface(on_open_change: EventHandler<bool>) -> HoverSurface {
    HoverSurface {
        state: use_signal(HoverState::default),
        on_open_change,
        runtime: arkit_runtime::use_runtime_handle().tokio(),
    }
}

impl HoverSurface {
    pub fn pinned(&self) -> bool {
        (self.state)().pinned
    }
    pub fn trigger_hover(&self, hovering: bool) {
        self.update(move |state| {
            if hovering && !state.trigger_hover {
                state.dismissed = false;
            }
            state.trigger_hover = hovering;
        });
    }
    pub fn panel_hover(&self, hovering: bool) {
        self.update(move |state| state.panel_hover = hovering);
    }
    pub fn trigger_focus(&self, focused: bool) {
        self.update(move |state| {
            if focused && !state.trigger_focus {
                if state.suppress_return_focus {
                    state.suppress_return_focus = false;
                } else {
                    state.dismissed = false;
                }
            }
            state.trigger_focus = focused;
        });
    }
    pub fn panel_focus(&self, focused: bool) {
        self.update(move |state| state.panel_focus = focused);
    }
    pub fn toggle_pin(&self, current: bool) {
        if current && self.state.peek().pinned {
            self.dismiss();
        } else {
            self.update(|state| {
                state.pinned = true;
                state.dismissed = false;
            });
        }
    }
    pub fn dismiss(&self) {
        let mut state = self.state;
        let mut next = *state.peek();
        next.suppress_return_focus = next.pinned;
        next.pinned = false;
        next.dismissed = true;
        next.revision = next.revision.wrapping_add(1);
        state.set(next);
        self.on_open_change.call(false);
    }
    fn update(&self, mutation: impl FnOnce(&mut HoverState)) {
        let mut state = self.state;
        let mut next = *state.peek();
        mutation(&mut next);
        next.revision = next.revision.wrapping_add(1);
        state.set(next);
        if next.holds_open() {
            self.on_open_change.call(true);
            return;
        }
        let revision = next.revision;
        let runtime = self.runtime.clone();
        let on_open_change = self.on_open_change;
        dioxus_core::spawn(async move {
            let _ = runtime
                .spawn(async {
                    tokio::time::sleep(std::time::Duration::from_millis(CLOSE_DELAY_MS)).await;
                })
                .await;
            let latest = *state.peek();
            if latest.revision == revision && !latest.holds_open() {
                on_open_change.call(false);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::HoverState;
    #[test]
    fn panel_hover_and_focus_keep_the_surface_open_after_trigger_exit() {
        assert!(HoverState {
            panel_hover: true,
            ..Default::default()
        }
        .holds_open());
        assert!(HoverState {
            panel_focus: true,
            ..Default::default()
        }
        .holds_open());
        assert!(HoverState {
            trigger_focus: true,
            ..Default::default()
        }
        .holds_open());
        assert!(!HoverState {
            trigger_hover: true,
            dismissed: true,
            ..Default::default()
        }
        .holds_open());
        assert!(!HoverState::default().holds_open());
    }
}
