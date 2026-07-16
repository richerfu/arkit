//! arkit_shadcn — shadcn-style components migrated to dioxus 0.7
//! `#[component]` + `rsx!`.
//!
//! Components live under `src/components/` and compose the ArkUI Dioxus
//! elements (`column`, `row`, `stack`, `flex`, `text`, `button`, `image`) with
//! theme tokens from [`crate::theme`].
//!
//! The native Markdown renderer is available behind the opt-in `markdown`
//! feature so its parser dependencies are not linked by the base component
//! library.

pub mod components;
pub mod icon;
pub mod styles;
pub mod theme;

pub mod prelude {
    pub use crate::components;
    pub use crate::theme::{
        self, use_theme, use_theme_provider, with_alpha, ColorTokens, RadiusTokens, Theme,
        ThemeMode, ThemePreset, ThemeProvider,
    };
    pub use crate::theme::{color, radius, spacing, typography};
}
