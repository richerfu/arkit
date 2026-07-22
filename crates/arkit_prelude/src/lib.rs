//! Shared prelude for the arkit dioxus stack.
//!
//! Aggregates the dioxus core pieces (`rsx!`, hooks, signals, `Element`,
//! `EventHandler`, `Props`) and the `dioxus_elements` ArkUI element/event
//! descriptors.
//!
//! Both the `arkit` facade and the crates that the facade re-exports
//! (`arkit_shadcn`, `arkit_icon`, `arkit_animation`) depend on this so they do
//! not need to depend on the `arkit` facade itself (which would be cyclic).

// dioxus core runtime types + the rsx!/component/Props macros.
pub use dioxus_core::{
    current_scope_id, use_drop, use_hook, Element, ElementId, EventHandler, Properties, ScopeId,
};
pub use dioxus_core_macro::{component, rsx, Props};

// Hooks.
pub use dioxus_hooks::{
    try_use_context, use_context, use_context_provider, use_coroutine, use_effect, use_future,
    use_memo, use_reactive, use_resource, use_signal,
};

// Signals. (`ReadOnlySignal` is deprecated in 0.7 in favor of `ReadSignal`.)
pub use dioxus_signals::{Memo, ReadSignal, ReadableExt, Signal, WritableExt};

// The crate names themselves — `rsx!`-emitted paths (`dioxus_core::...`,
// `dioxus_elements::...`) must resolve at the call site.
pub use dioxus_core;
pub use dioxus_core_macro;
pub use dioxus_elements;
pub use dioxus_hooks;
pub use dioxus_signals;

// ArkUI element/event descriptors (e.g. `column`, `text`, `onclick`).
pub use dioxus_elements::events::*;
pub use dioxus_elements::*;
