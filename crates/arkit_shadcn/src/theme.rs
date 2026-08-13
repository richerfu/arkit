//! Shadcn theme presets and the provider that mounts them onto components.
//!
//! Token types live in [`arkit_component::style`]. This module owns the
//! Zinc/Neutral/… palettes and installs a [`crate::kit::ShadcnKit`] so
//! headless primitives pick up shadcn appearances.

use arkit_component::style::{use_style_provider, StyleKitHandle};
use arkit_prelude::*;
use dioxus_core_macro::{component, Props};

pub use arkit_component::style::{
    color, radius, spacing, typography, with_alpha, ColorTokens, RadiusTokens, ThemeMode,
};

use crate::kit::ShadcnKit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreset {
    #[default]
    Zinc,
    Neutral,
    Stone,
    Mauve,
    Olive,
    Mist,
    Taupe,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub mode: ThemeMode,
    pub preset: Option<ThemePreset>,
    pub colors: ColorTokens,
    pub radii: RadiusTokens,
}

impl Theme {
    pub const fn preset(preset: ThemePreset, mode: ThemeMode) -> Self {
        Self {
            mode,
            preset: Some(preset),
            colors: preset_tokens(preset, mode),
            radii: RadiusTokens {
                sm: radius::SM,
                md: radius::MD,
                lg: radius::LG,
                xl: radius::XL,
                xxl: radius::XXL,
                full: radius::FULL,
            },
        }
    }

    pub const fn light(preset: ThemePreset) -> Self {
        Self::preset(preset, ThemeMode::Light)
    }

    pub const fn dark(preset: ThemePreset) -> Self {
        Self::preset(preset, ThemeMode::Dark)
    }

    pub const fn custom(colors: ColorTokens) -> Self {
        Self {
            mode: ThemeMode::Light,
            preset: None,
            colors,
            radii: RadiusTokens {
                sm: radius::SM,
                md: radius::MD,
                lg: radius::LG,
                xl: radius::XL,
                xxl: radius::XXL,
                full: radius::FULL,
            },
        }
    }

    pub const fn with_mode(mut self, mode: ThemeMode) -> Self {
        self.mode = mode;
        self
    }

    pub const fn with_colors(mut self, colors: ColorTokens) -> Self {
        self.colors = colors;
        self
    }

    pub const fn with_radius(mut self, radii: RadiusTokens) -> Self {
        self.radii = radii;
        self
    }

    /// Token snapshot consumed by `arkit_component`.
    pub const fn tokens(self) -> arkit_component::style::Theme {
        arkit_component::style::Theme {
            mode: self.mode,
            colors: self.colors,
            radii: self.radii,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light(ThemePreset::Zinc)
    }
}

/// Provide a mutable theme signal to the current Dioxus subtree.
///
/// This is a custom hook and must be called unconditionally from a component.
/// Prefer [`ThemeProvider`] when the theme is controlled by props; use this
/// hook when the component itself needs to mutate the active theme.
pub fn use_theme_provider(initial: Theme) -> Signal<Theme> {
    use_context_provider(move || Signal::new(initial))
}

/// Read the active theme from context (reactively). Falls back to
/// [`Theme::default`] when no provider is mounted so leaf components still
/// render in tests/snippets.
pub fn use_theme() -> Theme {
    dioxus_core::try_consume_context::<dioxus_signals::Signal<Theme>>()
        .map(|sig| sig())
        .unwrap_or_default()
}

const fn preset_tokens(preset: ThemePreset, mode: ThemeMode) -> ColorTokens {
    match mode {
        ThemeMode::Light => light_tokens(preset),
        ThemeMode::Dark => dark_tokens(preset),
    }
}

const fn light_tokens(preset: ThemePreset) -> ColorTokens {
    match preset {
        ThemePreset::Zinc => base_tokens(
            TokenSeed {
                canvas: [0xFFFFFFFF, 0xFF09090B],
                card: [0xFFFFFFFF, 0xFF09090B],
                primary: [0xFF09090B, 0xFFFAFAFA],
                secondary: [0xFFF4F4F5, 0xFF09090B],
                muted_foreground: 0xFF71717A,
                border: 0xFFE4E4E7,
                ring: 0xFF71717A,
            },
            ThemeMode::Light,
        ),
        ThemePreset::Neutral => base_tokens(
            TokenSeed {
                canvas: [0xFFFFFFFF, 0xFF0A0A0A],
                card: [0xFFFFFFFF, 0xFF0A0A0A],
                primary: [0xFF171717, 0xFFFAFAFA],
                secondary: [0xFFF5F5F5, 0xFF171717],
                muted_foreground: 0xFF737373,
                border: 0xFFE5E5E5,
                ring: 0xFF737373,
            },
            ThemeMode::Light,
        ),
        ThemePreset::Stone => base_tokens(
            TokenSeed {
                canvas: [0xFFFFFFFF, 0xFF0C0A09],
                card: [0xFFFFFFFF, 0xFF0C0A09],
                primary: [0xFF1C1917, 0xFFFAFAF9],
                secondary: [0xFFF5F5F4, 0xFF1C1917],
                muted_foreground: 0xFF78716C,
                border: 0xFFE7E5E4,
                ring: 0xFF78716C,
            },
            ThemeMode::Light,
        ),
        ThemePreset::Mauve => base_tokens(
            TokenSeed {
                canvas: [0xFFFFFFFF, 0xFF1F1A24],
                card: [0xFFFFFFFF, 0xFF1F1A24],
                primary: [0xFF2E2633, 0xFFFBF8FC],
                secondary: [0xFFF4EEF7, 0xFF2E2633],
                muted_foreground: 0xFF7A6F80,
                border: 0xFFE8DFED,
                ring: 0xFF7A6F80,
            },
            ThemeMode::Light,
        ),
        ThemePreset::Olive => base_tokens(
            TokenSeed {
                canvas: [0xFFFFFFFF, 0xFF1C1F1A],
                card: [0xFFFFFFFF, 0xFF1C1F1A],
                primary: [0xFF283025, 0xFFFAFCF8],
                secondary: [0xFFF1F5EE, 0xFF283025],
                muted_foreground: 0xFF6F7869,
                border: 0xFFE2E8DD,
                ring: 0xFF6F7869,
            },
            ThemeMode::Light,
        ),
        ThemePreset::Mist => base_tokens(
            TokenSeed {
                canvas: [0xFFFFFFFF, 0xFF172123],
                card: [0xFFFFFFFF, 0xFF172123],
                primary: [0xFF203033, 0xFFF7FCFC],
                secondary: [0xFFEDF5F5, 0xFF203033],
                muted_foreground: 0xFF667779,
                border: 0xFFDCE8E8,
                ring: 0xFF667779,
            },
            ThemeMode::Light,
        ),
        ThemePreset::Taupe => base_tokens(
            TokenSeed {
                canvas: [0xFFFFFFFF, 0xFF211D1B],
                card: [0xFFFFFFFF, 0xFF211D1B],
                primary: [0xFF302A27, 0xFFFCFAF8],
                secondary: [0xFFF5F1EE, 0xFF302A27],
                muted_foreground: 0xFF7B716B,
                border: 0xFFE8E1DD,
                ring: 0xFF7B716B,
            },
            ThemeMode::Light,
        ),
    }
}

const fn dark_tokens(preset: ThemePreset) -> ColorTokens {
    match preset {
        ThemePreset::Zinc => base_tokens(
            TokenSeed {
                canvas: [0xFF09090B, 0xFFFAFAFA],
                card: [0xFF18181B, 0xFFFAFAFA],
                primary: [0xFFFAFAFA, 0xFF18181B],
                secondary: [0xFF27272A, 0xFFFAFAFA],
                muted_foreground: 0xFFA1A1AA,
                border: 0xFF27272A,
                ring: 0xFFD4D4D8,
            },
            ThemeMode::Dark,
        ),
        ThemePreset::Neutral => base_tokens(
            TokenSeed {
                canvas: [0xFF0A0A0A, 0xFFFAFAFA],
                card: [0xFF171717, 0xFFFAFAFA],
                primary: [0xFFFAFAFA, 0xFF171717],
                secondary: [0xFF262626, 0xFFFAFAFA],
                muted_foreground: 0xFFA3A3A3,
                border: 0xFF262626,
                ring: 0xFFD4D4D4,
            },
            ThemeMode::Dark,
        ),
        ThemePreset::Stone => base_tokens(
            TokenSeed {
                canvas: [0xFF0C0A09, 0xFFFAFAF9],
                card: [0xFF1C1917, 0xFFFAFAF9],
                primary: [0xFFFAFAF9, 0xFF1C1917],
                secondary: [0xFF292524, 0xFFFAFAF9],
                muted_foreground: 0xFFA8A29E,
                border: 0xFF292524,
                ring: 0xFFD6D3D1,
            },
            ThemeMode::Dark,
        ),
        ThemePreset::Mauve => base_tokens(
            TokenSeed {
                canvas: [0xFF121016, 0xFFFBF8FC],
                card: [0xFF211C27, 0xFFFBF8FC],
                primary: [0xFFFBF8FC, 0xFF2E2633],
                secondary: [0xFF352C3A, 0xFFFBF8FC],
                muted_foreground: 0xFFB8ADBF,
                border: 0xFF352C3A,
                ring: 0xFFD8CDDD,
            },
            ThemeMode::Dark,
        ),
        ThemePreset::Olive => base_tokens(
            TokenSeed {
                canvas: [0xFF11140F, 0xFFFAFCF8],
                card: [0xFF1D241A, 0xFFFAFCF8],
                primary: [0xFFFAFCF8, 0xFF283025],
                secondary: [0xFF30382B, 0xFFFAFCF8],
                muted_foreground: 0xFFAFB8A9,
                border: 0xFF30382B,
                ring: 0xFFD0D8CA,
            },
            ThemeMode::Dark,
        ),
        ThemePreset::Mist => base_tokens(
            TokenSeed {
                canvas: [0xFF0D1416, 0xFFF7FCFC],
                card: [0xFF182528, 0xFFF7FCFC],
                primary: [0xFFF7FCFC, 0xFF203033],
                secondary: [0xFF283A3D, 0xFFF7FCFC],
                muted_foreground: 0xFFA7B8BA,
                border: 0xFF283A3D,
                ring: 0xFFCADADB,
            },
            ThemeMode::Dark,
        ),
        ThemePreset::Taupe => base_tokens(
            TokenSeed {
                canvas: [0xFF14110F, 0xFFFCFAF8],
                card: [0xFF241F1C, 0xFFFCFAF8],
                primary: [0xFFFCFAF8, 0xFF302A27],
                secondary: [0xFF39312D, 0xFFFCFAF8],
                muted_foreground: 0xFFB8ADA7,
                border: 0xFF39312D,
                ring: 0xFFD8CEC8,
            },
            ThemeMode::Dark,
        ),
    }
}

#[derive(Clone, Copy)]
struct TokenSeed {
    canvas: [u32; 2],
    card: [u32; 2],
    primary: [u32; 2],
    secondary: [u32; 2],
    muted_foreground: u32,
    border: u32,
    ring: u32,
}

const fn base_tokens(seed: TokenSeed, mode: ThemeMode) -> ColorTokens {
    let background = seed.canvas[0];
    let foreground = seed.canvas[1];
    let card = seed.card[0];
    let card_foreground = seed.card[1];
    let primary = seed.primary[0];
    let primary_foreground = seed.primary[1];
    let secondary = seed.secondary[0];
    let secondary_foreground = seed.secondary[1];
    let muted_foreground = seed.muted_foreground;
    let border = seed.border;
    let ring = seed.ring;
    let dark = matches!(mode, ThemeMode::Dark);
    ColorTokens {
        background,
        foreground,
        card,
        card_foreground,
        popover: card,
        popover_foreground: card_foreground,
        primary,
        primary_foreground,
        primary_track: with_alpha(primary, 0x33),
        secondary,
        secondary_foreground,
        muted: secondary,
        muted_foreground,
        accent: secondary,
        accent_foreground: secondary_foreground,
        destructive: if dark { 0xFF7F1D1D } else { zinc::DESTRUCTIVE },
        destructive_foreground: zinc::DESTRUCTIVE_FOREGROUND,
        border,
        input: border,
        ring,
        surface: background,
        chart_1: if dark { 0xFF3B82F6 } else { zinc::CHART_1 },
        chart_2: if dark { 0xFF10B981 } else { zinc::CHART_2 },
        chart_3: if dark { 0xFFF59E0B } else { zinc::CHART_3 },
        chart_4: if dark { 0xFFA855F7 } else { zinc::CHART_4 },
        chart_5: if dark { 0xFFEF4444 } else { zinc::CHART_5 },
        sidebar: if dark { card } else { secondary },
        sidebar_foreground: foreground,
        sidebar_primary: primary,
        sidebar_primary_foreground: primary_foreground,
        sidebar_accent: secondary,
        sidebar_accent_foreground: secondary_foreground,
        sidebar_border: border,
        sidebar_ring: ring,
    }
}

mod zinc {
    pub const DESTRUCTIVE: u32 = 0xFFEF4444;
    pub const DESTRUCTIVE_FOREGROUND: u32 = 0xFFFAFAFA;
    pub const CHART_1: u32 = 0xFFE76E50;
    pub const CHART_2: u32 = 0xFF2A9D90;
    pub const CHART_3: u32 = 0xFF274754;
    pub const CHART_4: u32 = 0xFFE8C468;
    pub const CHART_5: u32 = 0xFFF4A462;
}

/// Theme provider component. Mount near the app root to seed the dioxus
/// context consumed by [`use_theme`]. The theme is held in a `Signal<Theme>`
/// so descendants re-skin reactively when the provider's theme signal is
/// updated.
#[component]
pub fn ThemeProvider(theme: Theme, children: Element) -> Element {
    let mut provided = use_theme_provider(theme);
    use_effect(use_reactive((&theme,), move |(theme,)| {
        if *provided.peek() != theme {
            provided.set(theme);
        }
    }));
    let _ = use_style_provider(StyleKitHandle::new(ShadcnKit { theme: provided }));
    let tokens = provided().tokens();
    let _ = arkit_component::style::use_token_provider(tokens);
    rsx! { {children} }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_matches_legacy_zinc_light_tokens() {
        let theme = Theme::default();
        assert_eq!(theme.mode, ThemeMode::Light);
        assert_eq!(theme.preset, Some(ThemePreset::Zinc));
        assert_eq!(theme.colors.background, 0xFFFFFFFF);
        assert_eq!(theme.colors.foreground, 0xFF09090B);
        assert_eq!(theme.colors.primary_track, 0x3309090B);
        assert_eq!(theme.radii.md, radius::MD);
    }

    #[test]
    fn light_and_dark_presets_resolve_different_tokens() {
        let light = Theme::light(ThemePreset::Zinc);
        let dark = Theme::dark(ThemePreset::Zinc);
        assert_ne!(light.colors.background, dark.colors.background);
        assert_ne!(light.colors.foreground, dark.colors.foreground);
    }

    #[test]
    fn alpha_helper_replaces_alpha_channel() {
        assert_eq!(with_alpha(0xFF112233, 0x80), 0x80112233);
    }
}
