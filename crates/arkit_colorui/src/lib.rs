//! ColorUI restyle of [`arkit_component`] primitives.
//!
//! Same component surface as `arkit_shadcn`. Paint follows ColorUI. Theme is
//! only the default primary hue and light/dark tokens, not a second component
//! tree.

pub use arkit_component::components as primitives;
pub use arkit_component::style::{PaletteColor, ThemeMode};

pub mod components;
pub mod kit;
pub mod spec;
pub mod theme;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::kit::ColorUiKit;
    pub use crate::spec;
    pub use crate::theme::{
        provide_colorui_tokens, swatch, use_colorui, use_colorui_theme, use_theme, ColorUiProvider,
        ColorUiSwatch, ColorUiTheme, GradualColor,
    };
    pub use crate::PaletteColor;
    pub use arkit_component::style::{spacing, typography, with_alpha, ThemeMode};
}
