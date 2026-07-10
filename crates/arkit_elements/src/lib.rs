//! `dioxus_elements` registry for ArkUI.
//!
//! This crate mirrors the shape of `dioxus-html` so that `rsx!` (which resolves
//! element/attribute/event descriptors against an extern crate named
//! `dioxus_elements`) emits references that resolve here.
//!
//! Element names are lowercase-no-underscore (the rsx! native-element rule):
//! `column`, `row`, `stack`, `flex`, `text`, `button`, `image`, ...
//!
//! Each element module exposes:
//! - `TAG_NAME: &'static str` — the CamelCase ArkUI tag the renderer maps to a
//!   native node (e.g. `"Column"`).
//! - `NAME_SPACE: Option<&'static str>` — always `Some("arkui")`.
//! - `pub const` attribute descriptors of type
//!   `(&'static str, Option<&'static str>, bool)` = `(name, namespace, volatile)`.
//!
//! Events are callables produced by `impl_event!`.

pub mod elements;
pub mod event;
pub mod events;

pub use elements::*;

pub type AttributeDescription = (&'static str, Option<&'static str>, bool);
