use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub use arkit_runtime::internal::RuntimeHandle;
use crate::Theme;

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<HashMap<TypeId, Rc<dyn Any>>>> =
        RefCell::new(vec![HashMap::new()]);
}

pub fn current_runtime() -> Option<RuntimeHandle> {
    arkit_runtime::internal::current_runtime()
}

pub fn queue_ui_loop(effect: impl FnOnce() + 'static) {
    arkit_runtime::internal::queue_ui_loop(effect);
}

pub fn scope<R>(render: impl FnOnce() -> R) -> R {
    CONTEXT_STACK.with(|stack| {
        let next = stack.borrow().last().cloned().unwrap_or_default();
        stack.borrow_mut().push(next);
    });

    let result = render();

    CONTEXT_STACK.with(|stack| {
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
        stack
            .borrow()
            .last()
            .and_then(|current| current.get(&TypeId::of::<T>()).cloned())
            .and_then(|value| value.downcast_ref::<T>().cloned())
    })
}

pub fn use_local_context<T: Clone + 'static>() -> Option<T> {
    use_context::<T>()
}

pub fn current_theme() -> Theme {
    Theme::default()
}
