//! arkit_shadcn — shadcn-style components migrated to dioxus 0.7
//! `#[component]` + `rsx!`.
//!
//! Components live under `src/components/` and compose the ArkUI Dioxus
//! elements (`column`, `row`, `stack`, `flex`, `text`, `button`, `image`) with
//! theme tokens from [`crate::theme`].
//!
//! Optional features:
//! - `markdown` — native CommonMark/GFM [`components::Markdown`] renderer
//! - `code` — standalone [`components::Code`] + tree-sitter highlighting and
//!   language registration (no Markdown required)
//!
//! Enable both `markdown` and `code` for syntax-highlighted fenced blocks,
//! because Markdown reuses the Code pipeline when both features are present.
//!
//! ## Controlled state
//!
//! Components with `Option<T>` control props treat `Some(value)` as the only
//! source of truth. User dismissal emits `on_close` / `on_open_change(false)`;
//! the owner must update the controlled value before the declarative portal is
//! removed. Omitting the control prop enables the component's internal state.

pub mod components;
mod i18n;
pub mod icon;
pub mod theme;

pub mod prelude {
    pub use crate::components;
    pub use crate::theme::{
        self, use_theme, use_theme_provider, with_alpha, ColorTokens, RadiusTokens, Theme,
        ThemeMode, ThemePreset, ThemeProvider,
    };
    pub use crate::theme::{color, radius, spacing, typography};
}
