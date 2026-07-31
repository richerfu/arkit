use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use arkit_prelude::*;

use crate::provider::RouteScrollStore;

struct PageScroll {
    route: String,
    store: RouteScrollStore,
    position: Rc<Cell<f32>>,
    pending_restore: Rc<Cell<f32>>,
    restored: Option<f32>,
}

impl PageScroll {
    fn new(route: String, store: RouteScrollStore) -> Self {
        let restored = store.take(&route);
        let position = restored.unwrap_or_default();
        Self {
            route,
            store,
            position: Rc::new(Cell::new(position)),
            pending_restore: Rc::new(Cell::new(position)),
            restored,
        }
    }

    fn sync(&mut self, route: String) {
        if self.route == route {
            return;
        }

        self.commit();
        let restored = self.store.take(&route);
        let position = restored.unwrap_or_default();
        self.route = route;
        self.position.set(position);
        self.pending_restore.set(position);
        self.restored = restored;
    }

    fn commit(&self) {
        self.store.save(self.route.clone(), self.position.get());
    }
}

impl Drop for PageScroll {
    fn drop(&mut self) {
        self.commit();
    }
}

/// Props for [`RouteProvider`].
#[derive(Props, Clone, PartialEq)]
pub struct RouteProviderProps {
    /// Page content rendered inside the router's default native Scroll.
    pub children: Element,
}

/// Default scroll viewport for a routed page.
///
/// Render one `RouteProvider` at each route page root. It records ArkUI's
/// per-frame scroll deltas without rerendering and restores the route's saved
/// position when navigating back.
#[component]
pub fn RouteProvider(props: RouteProviderProps) -> Element {
    let runtime = arkit_runtime::use_runtime_handle();
    let store = use_context::<RouteScrollStore>();
    let route = dioxus_router::router().full_route_string();
    let initial_route = route.clone();
    let initial_store = store;
    let page_scroll =
        use_hook(move || Rc::new(RefCell::new(PageScroll::new(initial_route, initial_store))));
    page_scroll.borrow_mut().sync(route.clone());
    let (position, pending_restore, restored) = {
        let page_scroll = page_scroll.borrow();
        (
            page_scroll.position.clone(),
            page_scroll.pending_restore.clone(),
            page_scroll.restored,
        )
    };

    let restore_command = use_signal(|| None::<(String, f32)>);
    let last_effect_route = use_hook(|| RefCell::new(String::new()));
    let effect_route = route.clone();
    let effect_runtime = runtime.clone();
    let mut restore_effect = use_effect(move || {
        let desired = restored.map(|position| (effect_route.clone(), position));
        let mut command = restore_command;
        effect_runtime.queue_ui(move || {
            let Ok(current) = command.try_peek() else {
                return;
            };
            if *current == desired {
                return;
            }
            drop(current);
            if let Ok(mut current) = command.try_write() {
                *current = desired;
            }
        });
    });
    if *last_effect_route.borrow() != route {
        *last_effect_route.borrow_mut() = route.clone();
        restore_effect.mark_dirty();
    }

    let offset = restore_command()
        .filter(|(command_route, _)| command_route == &route)
        .map(|(_, position)| format!("0,{position},0"));

    rsx! {
        scroll {
            key: "{route}",
            width: "100%",
            height: "100%",
            layout_weight: 1.0,
            scroll_bar: "auto",
            scroll_enabled: true,
            scroll_offset: offset,
            onscroll: move |event| {
                record_scroll_delta(
                    &position,
                    &pending_restore,
                    *event.data(),
                );
            },
            {props.children}
        }
    }
}

fn record_scroll_delta(
    position: &Cell<f32>,
    pending_restore: &Cell<f32>,
    data: dioxus_elements::event::ScrollData,
) {
    if !data.has_offset {
        return;
    }

    let (delta, pending) = consume_restoration_delta(data.offset_y, pending_restore.get());
    pending_restore.set(pending);
    position.set(sanitize_offset(position.get() + delta));
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

fn sanitize_offset(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
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
