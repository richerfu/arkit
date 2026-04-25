pub use arkit_runtime::internal::RuntimeHandle;

pub fn current_runtime() -> Option<RuntimeHandle> {
    arkit_runtime::internal::current_runtime()
}

pub fn queue_ui_loop(effect: impl FnOnce() + 'static) {
    arkit_runtime::internal::queue_ui_loop(effect);
}
