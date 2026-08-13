//! ColorUI palette and the default-hue provider.

use arkit_component::style::{
    use_style_provider, ColorTokens, PaletteColor, RadiusTokens, StyleKitHandle, Theme, ThemeMode,
};
use arkit_prelude::*;
use dioxus_core_macro::{component, Props};

use crate::kit::ColorUiKit;

/// Solid ColorUI hues from `main.css` (`bg-red`, `bg-blue`, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorUiSwatch {
    pub fill: u32,
    pub ink: u32,
    pub light_fill: u32,
    pub light_ink: u32,
}

impl ColorUiSwatch {
    pub const fn new(fill: u32, ink: u32, light_fill: u32, light_ink: u32) -> Self {
        Self {
            fill,
            ink,
            light_fill,
            light_ink,
        }
    }
}

pub fn swatch(color: PaletteColor) -> ColorUiSwatch {
    match color {
        PaletteColor::Default | PaletteColor::Green => {
            ColorUiSwatch::new(0xFF39B54A, 0xFFFFFFFF, 0xFFD7F0DB, 0xFF39B54A)
        }
        PaletteColor::Red => ColorUiSwatch::new(0xFFE54D42, 0xFFFFFFFF, 0xFFFADBD9, 0xFFE54D42),
        PaletteColor::Orange => ColorUiSwatch::new(0xFFF37B1D, 0xFFFFFFFF, 0xFFFDE6D2, 0xFFF37B1D),
        PaletteColor::Yellow => ColorUiSwatch::new(0xFFFBBD08, 0xFF333333, 0xFFFEF2CE, 0xFFFBBD08),
        PaletteColor::Olive => ColorUiSwatch::new(0xFF8DC63F, 0xFFFFFFFF, 0xFFE8F4D9, 0xFF8DC63F),
        PaletteColor::Cyan => ColorUiSwatch::new(0xFF1CBBB4, 0xFFFFFFFF, 0xFFD2F1F0, 0xFF1CBBB4),
        PaletteColor::Blue => ColorUiSwatch::new(0xFF0081FF, 0xFFFFFFFF, 0xFFCCE6FF, 0xFF0081FF),
        PaletteColor::Purple => ColorUiSwatch::new(0xFF6739B6, 0xFFFFFFFF, 0xFFE1D7F0, 0xFF6739B6),
        PaletteColor::Mauve => ColorUiSwatch::new(0xFF9C26B0, 0xFFFFFFFF, 0xFFEBD4EF, 0xFF9C26B0),
        PaletteColor::Pink => ColorUiSwatch::new(0xFFE03997, 0xFFFFFFFF, 0xFFF9D7EA, 0xFFE03997),
        PaletteColor::Brown => ColorUiSwatch::new(0xFFA5673F, 0xFFFFFFFF, 0xFFEDE1D9, 0xFFA5673F),
        PaletteColor::Grey => ColorUiSwatch::new(0xFF8799A3, 0xFFFFFFFF, 0xFFE7EBED, 0xFF8799A3),
        PaletteColor::Gray => ColorUiSwatch::new(0xFFF0F0F0, 0xFF333333, 0xFFF0F0F0, 0xFF333333),
        PaletteColor::Black => ColorUiSwatch::new(0xFF333333, 0xFFFFFFFF, 0xFF666666, 0xFFFFFFFF),
        PaletteColor::White => ColorUiSwatch::new(0xFFFFFFFF, 0xFF666666, 0xFFFFFFFF, 0xFF666666),
    }
}

/// Gradual fills. ArkUI nodes take a solid color, so these use the start stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GradualColor {
    #[default]
    Blue,
    Red,
    Orange,
    Green,
    Purple,
    Pink,
}

impl GradualColor {
    pub const fn fill(self) -> u32 {
        match self {
            Self::Blue => 0xFF0081FF,
            Self::Red => 0xFFF43F3B,
            Self::Orange => 0xFFFF9700,
            Self::Green => 0xFF39B54A,
            Self::Purple => 0xFF9000FF,
            Self::Pink => 0xFFEC008C,
        }
    }
}

/// Page-level ColorUI theme. `primary` is the default `bg-*` / control color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorUiTheme {
    pub mode: ThemeMode,
    pub primary: PaletteColor,
}

impl ColorUiTheme {
    pub const fn new(primary: PaletteColor, mode: ThemeMode) -> Self {
        Self { mode, primary }
    }

    pub const fn light(primary: PaletteColor) -> Self {
        Self::new(primary, ThemeMode::Light)
    }

    pub const fn dark(primary: PaletteColor) -> Self {
        Self::new(primary, ThemeMode::Dark)
    }

    pub fn tokens(self) -> Theme {
        let primary = swatch(self.primary);
        let dark = matches!(self.mode, ThemeMode::Dark);
        let background = if dark { 0xFF111111 } else { 0xFFF1F1F1 };
        let foreground = if dark { 0xFFF1F1F1 } else { 0xFF333333 };
        let card = if dark { 0xFF1F1F1F } else { 0xFFFFFFFF };
        let border = if dark { 0xFF333333 } else { 0xFFDDDDDD };
        let muted_fg = if dark { 0xFFAAAAAA } else { 0xFF888888 };
        let secondary = if dark { 0xFF2A2A2A } else { 0xFFF0F0F0 };
        Theme {
            mode: self.mode,
            colors: ColorTokens {
                background,
                foreground,
                card,
                card_foreground: foreground,
                popover: card,
                popover_foreground: foreground,
                primary: primary.fill,
                primary_foreground: primary.ink,
                primary_track: (primary.fill & 0x00FF_FFFF) | 0x3300_0000,
                secondary,
                secondary_foreground: foreground,
                muted: secondary,
                muted_foreground: muted_fg,
                accent: secondary,
                accent_foreground: foreground,
                destructive: 0xFFE54D42,
                destructive_foreground: 0xFFFFFFFF,
                border,
                input: border,
                ring: primary.fill,
                surface: background,
                chart_1: 0xFF0081FF,
                chart_2: 0xFF39B54A,
                chart_3: 0xFFF37B1D,
                chart_4: 0xFF6739B6,
                chart_5: 0xFFE54D42,
                sidebar: card,
                sidebar_foreground: foreground,
                sidebar_primary: primary.fill,
                sidebar_primary_foreground: primary.ink,
                sidebar_accent: secondary,
                sidebar_accent_foreground: foreground,
                sidebar_border: border,
                sidebar_ring: primary.fill,
            },
            radii: RadiusTokens {
                sm: 3.0,
                md: 6.0,
                lg: 10.0,
                xl: 16.0,
                xxl: 20.0,
                full: 999.0,
            },
        }
    }
}

impl Default for ColorUiTheme {
    fn default() -> Self {
        Self::light(PaletteColor::Green)
    }
}

pub fn use_colorui_provider(initial: ColorUiTheme) -> Signal<ColorUiTheme> {
    use_context_provider(move || Signal::new(initial))
}

pub fn use_colorui_theme() -> ColorUiTheme {
    dioxus_core::try_consume_context::<Signal<ColorUiTheme>>()
        .map(|sig| sig())
        .unwrap_or_default()
}

/// Token snapshot for page chrome. Same shape as shadcn `use_theme()`.
pub fn use_theme() -> Theme {
    use_colorui_theme().tokens()
}

/// Publish ColorUI tokens so headless descendants that call `use_theme()`
/// pick up ColorUI colors.
pub fn provide_colorui_tokens() {
    let tokens = use_colorui_theme().tokens();
    let _ = arkit_component::style::use_token_provider(tokens);
}

pub fn use_colorui(theme: ColorUiTheme) -> Signal<ColorUiTheme> {
    let mut provided = use_colorui_provider(theme);
    if *provided.peek() != theme {
        provided.set(theme);
    }
    let _ = use_style_provider(StyleKitHandle::new(ColorUiKit { theme: provided }));
    let _ = arkit_component::style::use_token_provider(provided().tokens());
    provided
}

#[component]
pub fn ColorUiProvider(theme: ColorUiTheme, children: Element) -> Element {
    let provided = use_colorui(theme);
    let tokens = provided().tokens();
    rsx! {
        column {
            width: "100%",
            height: "100%",
            background_color: tokens.colors.background,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_primary_is_colorui_green() {
        let theme = ColorUiTheme::default();
        assert_eq!(theme.primary, PaletteColor::Green);
        assert_eq!(theme.tokens().colors.primary, 0xFF39B54A);
    }
}
