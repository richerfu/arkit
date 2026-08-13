//! Headless ArkUI component primitives.
//!
//! Components in this crate own structure, state, and interaction. Style
//! crates restyle them by passing an explicit `appearance` (ColorUI) or by
//! mounting a shadcn [`style::StyleKit`] from `ThemeProvider` (preset / light
//! / dark switching inside one design language).

pub mod appearance;
pub mod components;
mod i18n;
pub mod icon;
pub mod style;
pub mod styles;

pub mod prelude {
    pub use crate::appearance;
    pub use crate::components;
    pub use crate::style::{
        self, use_style_kit, use_style_provider, use_theme, use_token_provider, with_alpha,
        ColorTokens, PaletteColor, RadiusTokens, StyleKit, StyleKitHandle, StyleProvider, Theme,
        ThemeMode,
    };
    pub use crate::style::{color, radius, spacing, typography};
}
