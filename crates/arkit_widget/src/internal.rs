use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::render_impl::Element;
use crate::Theme;
pub use arkit_runtime::internal::RuntimeHandle;

type ContextFrame = HashMap<TypeId, Rc<dyn Any>>;
type Cleanup = Box<dyn FnOnce()>;

#[derive(Clone, Default)]
struct ScopeFrame {
    path: Vec<usize>,
    next_child: usize,
}

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<ContextFrame>> =
        RefCell::new(vec![HashMap::new()]);
    static PERSISTENT_CONTEXTS: RefCell<HashMap<Vec<usize>, ContextFrame>> =
        RefCell::new(HashMap::new());
    static SCOPE_STACK: RefCell<Vec<ScopeFrame>> =
        RefCell::new(vec![ScopeFrame::default()]);
    static CLEANUP_STACK: RefCell<Vec<Vec<Cleanup>>> =
        RefCell::new(vec![Vec::new()]);
    static PERSISTENT_CLEANUPS: RefCell<HashMap<Vec<usize>, Vec<Cleanup>>> =
        RefCell::new(HashMap::new());
    static VISITED_SCOPES: RefCell<HashSet<Vec<usize>>> =
        RefCell::new(HashSet::new());
    static REGISTERED_OVERLAYS: RefCell<Vec<Box<dyn Any>>> =
        RefCell::new(Vec::new());
    static OVERLAY_REGISTRATION_ORDER: Cell<usize> = const { Cell::new(0) };
}

struct RegisteredOverlay {
    depth: usize,
    order: usize,
    overlay: Box<dyn Any>,
}

fn sort_registered_overlay_entries(entries: &mut Vec<Box<dyn Any>>) {
    entries.sort_by_key(|entry| {
        let entry = entry
            .downcast_ref::<RegisteredOverlay>()
            .expect("registered overlay payload did not match overlay registry");
        (entry.depth, entry.order)
    });
}

pub fn current_runtime() -> Option<RuntimeHandle> {
    arkit_runtime::internal::current_runtime()
}

pub fn queue_ui_loop(effect: impl FnOnce() + 'static) {
    arkit_runtime::internal::queue_ui_loop(effect);
}

pub fn begin_render_pass() {
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.clear();
        stack.push(HashMap::new());
    });
    SCOPE_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.clear();
        stack.push(ScopeFrame::default());
    });
    CLEANUP_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.clear();
        stack.push(Vec::new());
    });
    VISITED_SCOPES.with(|visited| {
        visited.borrow_mut().clear();
    });
    REGISTERED_OVERLAYS.with(|overlays| {
        overlays.borrow_mut().clear();
    });
    OVERLAY_REGISTRATION_ORDER.with(|order| order.set(0));
}

pub fn end_render_pass() {
    let stale_paths = PERSISTENT_CONTEXTS.with(|contexts| {
        VISITED_SCOPES.with(|visited| {
            let visited = visited.borrow();
            contexts
                .borrow()
                .keys()
                .filter(|path| !visited.contains(*path))
                .cloned()
                .collect::<Vec<_>>()
        })
    });

    PERSISTENT_CONTEXTS.with(|contexts| {
        let mut contexts = contexts.borrow_mut();
        for path in &stale_paths {
            contexts.remove(path);
        }
    });

    PERSISTENT_CLEANUPS.with(|cleanups| {
        let mut cleanups = cleanups.borrow_mut();
        for path in stale_paths {
            if let Some(mut callbacks) = cleanups.remove(&path) {
                while let Some(callback) = callbacks.pop() {
                    callback();
                }
            }
        }
    });

    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.clear();
        stack.push(HashMap::new());
    });
    SCOPE_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.clear();
        stack.push(ScopeFrame::default());
    });
    CLEANUP_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.clear();
        stack.push(Vec::new());
    });
    VISITED_SCOPES.with(|visited| {
        visited.borrow_mut().clear();
    });
    REGISTERED_OVERLAYS.with(|overlays| {
        overlays.borrow_mut().clear();
    });
    OVERLAY_REGISTRATION_ORDER.with(|order| order.set(0));
}

pub fn scope<R>(render: impl FnOnce() -> R) -> R {
    let path = SCOPE_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let parent = stack
            .last_mut()
            .expect("scope stack should always have a root frame");
        let id = parent.next_child;
        parent.next_child += 1;
        let mut path = parent.path.clone();
        path.push(id);
        stack.push(ScopeFrame {
            path: path.clone(),
            next_child: 0,
        });
        path
    });

    VISITED_SCOPES.with(|visited| {
        visited.borrow_mut().insert(path.clone());
    });

    let frame = PERSISTENT_CONTEXTS
        .with(|contexts| contexts.borrow().get(&path).cloned().unwrap_or_default());
    CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().push(frame);
    });
    CLEANUP_STACK.with(|stack| {
        stack.borrow_mut().push(Vec::new());
    });

    let result = render();

    CONTEXT_STACK.with(|stack| {
        let frame = stack.borrow_mut().pop().unwrap_or_default();
        PERSISTENT_CONTEXTS.with(|contexts| {
            contexts.borrow_mut().insert(path.clone(), frame);
        });
    });
    CLEANUP_STACK.with(|stack| {
        let callbacks = stack.borrow_mut().pop().unwrap_or_default();
        PERSISTENT_CLEANUPS.with(|cleanups| {
            cleanups.borrow_mut().insert(path.clone(), callbacks);
        });
    });
    SCOPE_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });

    result
}

pub fn provide_context<T: 'static>(value: T) {
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if let Some(current) = stack.last_mut() {
            current.insert(TypeId::of::<T>(), Rc::new(value));
        }
    });
}

pub fn use_context<T: Clone + 'static>() -> Option<T> {
    CONTEXT_STACK.with(|stack| {
        stack.borrow().iter().rev().find_map(|current| {
            current
                .get(&TypeId::of::<T>())
                .cloned()
                .and_then(|value| value.downcast_ref::<T>().cloned())
        })
    })
}

pub fn use_local_context<T: Clone + 'static>() -> Option<T> {
    CONTEXT_STACK.with(|stack| {
        stack
            .borrow()
            .last()
            .and_then(|current| current.get(&TypeId::of::<T>()).cloned())
            .and_then(|value| value.downcast_ref::<T>().cloned())
    })
}

pub fn on_cleanup(cleanup: impl FnOnce() + 'static) {
    CLEANUP_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if let Some(current) = stack.last_mut() {
            current.push(Box::new(cleanup));
        }
    });
}

pub fn register_overlay<Message: 'static, AppTheme: 'static>(overlay: Element<Message, AppTheme>) {
    let depth = SCOPE_STACK.with(|stack| {
        stack
            .borrow()
            .last()
            .map(|frame| frame.path.len())
            .unwrap_or_default()
    });
    let order = OVERLAY_REGISTRATION_ORDER.with(|next| {
        let order = next.get();
        next.set(order + 1);
        order
    });
    REGISTERED_OVERLAYS.with(|overlays| {
        overlays.borrow_mut().push(Box::new(RegisteredOverlay {
            depth,
            order,
            overlay: Box::new(overlay),
        }));
    });
}

pub fn take_registered_overlays<Message: 'static, AppTheme: 'static>(
) -> Vec<Element<Message, AppTheme>> {
    REGISTERED_OVERLAYS.with(|overlays| {
        let mut overlays = overlays.borrow_mut().drain(..).collect::<Vec<_>>();
        sort_registered_overlay_entries(&mut overlays);
        overlays
            .into_iter()
            .map(|entry| {
                let entry = match entry.downcast::<RegisteredOverlay>() {
                    Ok(entry) => *entry,
                    Err(_) => panic!("registered overlay payload did not match overlay registry"),
                };
                match entry.overlay.downcast::<Element<Message, AppTheme>>() {
                    Ok(overlay) => *overlay,
                    Err(_) => panic!("registered overlay type did not match current render pass"),
                }
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_overlays_render_after_parent_depths() {
        let mut overlays: Vec<Box<dyn Any>> = vec![
            Box::new(RegisteredOverlay {
                depth: 2,
                order: 1,
                overlay: Box::new(()),
            }),
            Box::new(RegisteredOverlay {
                depth: 1,
                order: 2,
                overlay: Box::new(()),
            }),
            Box::new(RegisteredOverlay {
                depth: 3,
                order: 0,
                overlay: Box::new(()),
            }),
        ];

        sort_registered_overlay_entries(&mut overlays);

        let depths = overlays
            .iter()
            .map(|entry| {
                entry
                    .downcast_ref::<RegisteredOverlay>()
                    .expect("registered overlay payload did not match overlay registry")
                    .depth
            })
            .collect::<Vec<_>>();

        assert_eq!(depths, vec![1, 2, 3]);
    }

    #[test]
    fn overlays_keep_registration_order_within_same_depth() {
        let mut overlays: Vec<Box<dyn Any>> = vec![
            Box::new(RegisteredOverlay {
                depth: 2,
                order: 3,
                overlay: Box::new(()),
            }),
            Box::new(RegisteredOverlay {
                depth: 2,
                order: 1,
                overlay: Box::new(()),
            }),
            Box::new(RegisteredOverlay {
                depth: 2,
                order: 2,
                overlay: Box::new(()),
            }),
        ];

        sort_registered_overlay_entries(&mut overlays);

        let orders = overlays
            .iter()
            .map(|entry| {
                entry
                    .downcast_ref::<RegisteredOverlay>()
                    .expect("registered overlay payload did not match overlay registry")
                    .order
            })
            .collect::<Vec<_>>();

        assert_eq!(orders, vec![1, 2, 3]);
    }

    #[test]
    fn use_context_inherits_from_parent_scope() {
        begin_render_pass();

        scope(|| {
            provide_context(String::from("parent"));

            scope(|| {
                assert_eq!(use_context::<String>(), Some(String::from("parent")));
                assert_eq!(use_local_context::<String>(), None);
            });
        });

        end_render_pass();
    }

    #[test]
    fn use_context_prefers_nearest_scope_value() {
        begin_render_pass();

        scope(|| {
            provide_context(String::from("parent"));

            scope(|| {
                provide_context(String::from("child"));

                assert_eq!(use_context::<String>(), Some(String::from("child")));
                assert_eq!(use_local_context::<String>(), Some(String::from("child")));
            });
        });

        end_render_pass();
    }
}

pub fn current_theme() -> Theme {
    Theme::default()
}
