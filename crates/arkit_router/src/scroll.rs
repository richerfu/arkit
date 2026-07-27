use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use arkit_prelude::*;

use crate::{
    provider::RouteStateContext,
    state::{PageStateStore, RouteEntryId, ScrollPosition, ScrollRestorationKey, ScrollScopeId},
};

struct ScrollRegistration {
    entry_id: Option<RouteEntryId>,
    scope: ScrollScopeId,
    page_states: Option<PageStateStore>,
    position: Rc<Cell<ScrollPosition>>,
    pending_restore: Rc<Cell<ScrollPosition>>,
    snapshot_token: Option<u64>,
    restored: bool,
}

impl ScrollRegistration {
    fn new(context: Option<RouteStateContext>, scope: ScrollScopeId) -> Self {
        let entry_id = context.as_ref().map(RouteStateContext::current_entry_id);
        let restored_position = context.as_ref().and_then(|context| {
            context.page_states.position(
                entry_id.expect("entry ID exists with route context"),
                &scope,
            )
        });
        let mut registration = Self {
            entry_id,
            scope,
            page_states: context.map(|context| context.page_states),
            position: Rc::new(Cell::new(restored_position.unwrap_or_default())),
            pending_restore: Rc::new(Cell::new(restored_position.unwrap_or_default())),
            snapshot_token: None,
            restored: restored_position.is_some(),
        };
        registration.register_snapshot();
        registration
    }

    fn commit(&self) {
        let (Some(entry_id), Some(page_states)) = (self.entry_id, self.page_states.as_ref()) else {
            return;
        };
        page_states.save_position(entry_id, self.scope.clone(), self.position.get());
    }

    fn clear_snapshot(&mut self) {
        let (Some(entry_id), Some(page_states), Some(token)) = (
            self.entry_id,
            self.page_states.as_ref(),
            self.snapshot_token.take(),
        ) else {
            return;
        };
        page_states.unregister_scroll_snapshot(entry_id, &self.scope, token);
    }

    fn register_snapshot(&mut self) {
        self.clear_snapshot();
        let (Some(entry_id), Some(page_states)) = (self.entry_id, self.page_states.as_ref()) else {
            return;
        };

        let position = self.position.clone();
        let snapshot = Rc::new(move || position.get());
        self.snapshot_token =
            Some(page_states.register_scroll_snapshot(entry_id, self.scope.clone(), snapshot));
    }

    fn sync(&mut self, context: Option<RouteStateContext>, scope: ScrollScopeId) {
        let entry_id = context.as_ref().map(RouteStateContext::current_entry_id);
        if self.entry_id == entry_id && self.scope == scope {
            return;
        }

        self.clear_snapshot();
        self.commit();
        let restored_position = context.as_ref().and_then(|context| {
            context.page_states.position(
                entry_id.expect("entry ID exists with route context"),
                &scope,
            )
        });
        self.entry_id = entry_id;
        self.scope = scope;
        self.page_states = context.map(|context| context.page_states);
        self.position.set(restored_position.unwrap_or_default());
        self.pending_restore
            .set(restored_position.unwrap_or_default());
        self.restored = restored_position.is_some();
        self.register_snapshot();
    }
}

impl Drop for ScrollRegistration {
    fn drop(&mut self) {
        self.clear_snapshot();
        self.commit();
    }
}

/// Non-reactive scroll recorder bound to one route history entry.
///
/// Recording updates a local [`Cell`] only. It does not update a Dioxus signal
/// or rerender the page on each scroll frame.
#[derive(Clone)]
pub struct ScrollRestorationHandle {
    _registration: Rc<RefCell<ScrollRegistration>>,
    position: Rc<Cell<ScrollPosition>>,
    pending_restore: Rc<Cell<ScrollPosition>>,
    entry_id: Option<RouteEntryId>,
    scope_key: String,
    restore_replay: Signal<Option<(RouteEntryId, ScrollPosition)>>,
}

impl ScrollRestorationHandle {
    /// One-shot value for the native `scroll_offset` command.
    ///
    /// `None` is returned during the mount frame. When this history entry has
    /// saved state, the command is published on the next UI-loop tick, after
    /// the Scroll node and its content have been attached and laid out.
    pub fn offset_attribute(&self) -> Option<String> {
        let (replay_entry, position) = (self.restore_replay)()?;
        (Some(replay_entry) == self.entry_id).then(|| format!("{},{},0", position.x, position.y))
    }

    /// Record an offset delta emitted by an ArkUI Scroll node.
    pub fn record(&self, data: dioxus_elements::event::ScrollData) {
        if data.has_offset {
            let pending = self.pending_restore.get();
            let (delta_x, pending_x) = consume_restoration_delta(data.offset_x, pending.x);
            let (delta_y, pending_y) = consume_restoration_delta(data.offset_y, pending.y);
            self.pending_restore
                .set(ScrollPosition::new(pending_x, pending_y));

            let current = self.position.get();
            self.position.set(ScrollPosition::new(
                current.x + delta_x,
                current.y + delta_y,
            ));
        }
    }

    /// Return the latest position without subscribing the component to it.
    pub fn position(&self) -> ScrollPosition {
        self.position.get()
    }

    /// Key for the native scroll node.
    ///
    /// Assign this to `key` when binding a custom Scroll. This guarantees a
    /// fresh native viewport when the history entry changes, even when Dioxus
    /// reuses the same route component for different parameters.
    pub fn node_key(&self) -> String {
        match self.entry_id {
            Some(entry_id) => {
                format!("route-scroll-{}-{}", entry_id.get(), self.scope_key)
            }
            None => format!("route-scroll-unmanaged-{}", self.scope_key),
        }
    }
}

fn use_scroll_scope(scope: ScrollScopeId) -> ScrollRestorationHandle {
    let context = try_use_context::<RouteStateContext>();
    let initial_context = context.clone();
    let initial_scope = scope.clone();
    let registration = use_hook(move || {
        Rc::new(RefCell::new(ScrollRegistration::new(
            initial_context,
            initial_scope,
        )))
    });
    registration.borrow_mut().sync(context, scope);
    let (position, pending_restore, entry_id, restored, scope_key) = {
        let registration = registration.borrow();
        let scope_key = match &registration.scope {
            ScrollScopeId::Page => "page".to_string(),
            ScrollScopeId::Named(key) => key.as_str().to_string(),
        };
        (
            registration.position.clone(),
            registration.pending_restore.clone(),
            registration.entry_id,
            registration.restored,
            scope_key,
        )
    };
    let last_effect_entry = use_hook(|| Cell::new(None::<RouteEntryId>));
    let restore_replay = use_signal(|| None::<(RouteEntryId, ScrollPosition)>);
    let restore_position = position.clone();
    let mut restore_effect = use_effect(move || {
        let desired = entry_id
            .filter(|_| restored)
            .map(|entry_id| (entry_id, restore_position.get()));
        let mut replay = restore_replay;
        arkit_runtime::queue_ui_loop(move || {
            let Ok(current) = replay.try_peek() else {
                return;
            };
            if *current == desired {
                return;
            }
            drop(current);
            if let Ok(mut current) = replay.try_write() {
                *current = desired;
            }
        });
    });
    if last_effect_entry.replace(entry_id) != entry_id {
        restore_effect.mark_dirty();
    }

    ScrollRestorationHandle {
        _registration: registration,
        position,
        pending_restore,
        entry_id,
        scope_key,
        restore_replay,
    }
}

/// Bind a named nested Scroll node to the current route history entry.
///
/// Use the returned handle for `key`, `scroll_offset`, and `onscroll`:
///
/// ```ignore
/// let restoration = use_scroll_restoration("results");
/// let node_key = restoration.node_key();
/// let offset = restoration.offset_attribute();
/// let recorder = restoration.clone();
///
/// scroll {
///     key: "{node_key}",
///     scroll_offset: offset,
///     onscroll: move |event| recorder.record(*event.data()),
///     // ...
/// }
/// ```
pub fn use_scroll_restoration(key: impl Into<ScrollRestorationKey>) -> ScrollRestorationHandle {
    use_scroll_scope(ScrollScopeId::Named(key.into()))
}

fn use_page_scroll_restoration() -> ScrollRestorationHandle {
    use_scroll_scope(ScrollScopeId::Page)
}

/// Props for [`RouteProvider`].
#[derive(Props, Clone, PartialEq)]
pub struct RouteProviderProps {
    /// A single flow-content root rendered inside the native Scroll node.
    pub children: Element,
    #[props(default = "100%".to_string())]
    pub width: String,
    #[props(default = "100%".to_string())]
    pub height: String,
    /// Flex weight of the viewport in its parent page column.
    #[props(default = 1.0)]
    pub layout_weight: f32,
    #[props(default = "auto".to_string())]
    pub scroll_bar: String,
    #[props(default = true)]
    pub scroll_enabled: bool,
    /// Optional observer for the current absolute position, in vp.
    #[props(default)]
    pub on_scroll: Option<EventHandler<ScrollPosition>>,
}

/// Route page root with automatic back/forward scroll restoration.
///
/// The component supplies the actual native Scroll node, so route components
/// can return ordinary flow content without making Scroll their root. Scroll
/// samples are kept outside reactive state, avoiding rerenders while scrolling.
/// Use one `RouteProvider` for the primary viewport of a route; bind additional
/// Scroll nodes with [`use_scroll_restoration`].
///
/// ```ignore
/// #[component]
/// fn Home() -> Element {
///     rsx! {
///         RouteProvider {
///             column {
///                 // Long page content...
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn RouteProvider(props: RouteProviderProps) -> Element {
    let restoration = use_page_scroll_restoration();
    let node_key = restoration.node_key();
    let offset = restoration.offset_attribute();
    let recorder = restoration;
    let on_scroll = props.on_scroll;

    rsx! {
        scroll {
            key: "{node_key}",
            width: props.width,
            height: props.height,
            layout_weight: props.layout_weight.max(0.0),
            scroll_bar: props.scroll_bar,
            scroll_enabled: props.scroll_enabled,
            scroll_offset: offset,
            onscroll: move |event| {
                recorder.record(*event.data());
                if let Some(handler) = on_scroll {
                    handler.call(recorder.position());
                }
            },
            {props.children}
        }
    }
}

fn consume_restoration_delta(delta: f32, pending: f32) -> (f32, f32) {
    if !delta.is_finite() {
        return (0.0, pending);
    }
    if pending <= 0.0 || delta <= 0.0 {
        return (delta, pending);
    }

    let consumed = delta.min(pending);
    (delta - consumed, pending - consumed)
}

#[cfg(test)]
mod tests {
    use super::consume_restoration_delta;

    #[test]
    fn restore_deltas_do_not_double_count_the_saved_position() {
        assert_eq!(consume_restoration_delta(40.0, 100.0), (0.0, 60.0));
        assert_eq!(consume_restoration_delta(80.0, 60.0), (20.0, 0.0));
    }

    #[test]
    fn user_delta_passes_through_without_a_pending_restore() {
        assert_eq!(consume_restoration_delta(24.0, 0.0), (24.0, 0.0));
        assert_eq!(consume_restoration_delta(-12.0, 80.0), (-12.0, 80.0));
    }
}
