//! shadcn restyle of [`arkit_component`] primitives.
//!
//! Same unstyled structure, shadcn paint. [`theme::ThemeProvider`] switches
//! Zinc / Neutral / light / dark. Visual result matches the pre-split shadcn
//! kit.
//!
//! Optional features:
//! - `markdown` — native CommonMark/GFM [`components::Markdown`] renderer
//! - `code` — standalone [`components::Code`] + tree-sitter highlighting
//! - `markdown-highlight` — `markdown` + `code`

pub mod components;
pub use arkit_component::icon;
pub use arkit_component::styles;

pub mod kit;
pub mod spec;
pub mod theme;

pub mod prelude {
    pub use crate::components;
    pub use crate::kit::ShadcnKit;
    pub use crate::spec;
    pub use crate::theme::{
        self, use_theme, use_theme_provider, with_alpha, ColorTokens, RadiusTokens, Theme,
        ThemeMode, ThemePreset, ThemeProvider,
    };
    pub use crate::theme::{color, radius, spacing, typography};
}
