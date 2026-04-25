use std::cell::RefCell;

thread_local! {
    static THEME_STACK: RefCell<Vec<Theme>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorTokens {
    pub background: u32,
    pub foreground: u32,
    pub card: u32,
    pub card_foreground: u32,
    pub popover: u32,
    pub popover_foreground: u32,
    pub primary: u32,
    pub primary_foreground: u32,
    pub primary_track: u32,
    pub secondary: u32,
    pub secondary_foreground: u32,
    pub muted: u32,
    pub muted_foreground: u32,
    pub accent: u32,
    pub accent_foreground: u32,
    pub destructive: u32,
    pub destructive_foreground: u32,
    pub border: u32,
    pub input: u32,
    pub ring: u32,
    pub surface: u32,
    pub chart_1: u32,
    pub chart_2: u32,
    pub chart_3: u32,
    pub chart_4: u32,
    pub chart_5: u32,
    pub sidebar: u32,
    pub sidebar_foreground: u32,
    pub sidebar_primary: u32,
    pub sidebar_primary_foreground: u32,
    pub sidebar_accent: u32,
    pub sidebar_accent_foreground: u32,
    pub sidebar_border: u32,
    pub sidebar_ring: u32,
}

impl ColorTokens {
    pub const fn with_primary_track(mut self, value: u32) -> Self {
        self.primary_track = value;
        self
    }

    pub const fn with_surface(mut self, value: u32) -> Self {
        self.surface = value;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadiusTokens {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub full: f32,
}

impl RadiusTokens {
    pub const fn from_base(base: f32) -> Self {
        Self {
            sm: base * 0.5,
            md: base * 0.75,
            lg: base,
            xl: base * 1.5,
            xxl: base * 2.0,
            full: 999.0,
        }
    }
}

impl Default for RadiusTokens {
    fn default() -> Self {
        Self {
            sm: radius::SM,
            md: radius::MD,
            lg: radius::LG,
            xl: radius::XL,
            xxl: radius::XXL,
            full: radius::FULL,
        }
    }
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
}

impl Default for Theme {
    fn default() -> Self {
        Self::light(ThemePreset::Zinc)
    }
}

pub fn with_theme<R>(theme: Theme, render: impl FnOnce() -> R) -> R {
    struct ThemeGuard;

    impl Drop for ThemeGuard {
        fn drop(&mut self) {
            THEME_STACK.with(|stack| {
                stack.borrow_mut().pop();
            });
        }
    }

    THEME_STACK.with(|stack| {
        stack.borrow_mut().push(theme);
    });
    let _guard = ThemeGuard;

    render()
}

pub fn current() -> Theme {
    THEME_STACK
        .with(|stack| stack.borrow().last().copied())
        .unwrap_or_default()
}

pub fn colors() -> ColorTokens {
    current().colors
}

pub fn radii() -> RadiusTokens {
    current().radii
}

pub const fn with_alpha(color: u32, alpha: u8) -> u32 {
    (color & 0x00FF_FFFF) | ((alpha as u32) << 24)
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
            0xFFFFFFFF, 0xFF09090B, 0xFFFFFFFF, 0xFF09090B, 0xFF09090B, 0xFFFAFAFA, 0xFFF4F4F5,
            0xFF09090B, 0xFF71717A, 0xFFE4E4E7, 0xFF71717A,
        ),
        ThemePreset::Neutral => base_tokens(
            0xFFFFFFFF, 0xFF0A0A0A, 0xFFFFFFFF, 0xFF0A0A0A, 0xFF171717, 0xFFFAFAFA, 0xFFF5F5F5,
            0xFF171717, 0xFF737373, 0xFFE5E5E5, 0xFF737373,
        ),
        ThemePreset::Stone => base_tokens(
            0xFFFFFFFF, 0xFF0C0A09, 0xFFFFFFFF, 0xFF0C0A09, 0xFF1C1917, 0xFFFAFAF9, 0xFFF5F5F4,
            0xFF1C1917, 0xFF78716C, 0xFFE7E5E4, 0xFF78716C,
        ),
        ThemePreset::Mauve => base_tokens(
            0xFFFFFFFF, 0xFF1F1A24, 0xFFFFFFFF, 0xFF1F1A24, 0xFF2E2633, 0xFFFBF8FC, 0xFFF4EEF7,
            0xFF2E2633, 0xFF7A6F80, 0xFFE8DFED, 0xFF7A6F80,
        ),
        ThemePreset::Olive => base_tokens(
            0xFFFFFFFF, 0xFF1C1F1A, 0xFFFFFFFF, 0xFF1C1F1A, 0xFF283025, 0xFFFAFCF8, 0xFFF1F5EE,
            0xFF283025, 0xFF6F7869, 0xFFE2E8DD, 0xFF6F7869,
        ),
        ThemePreset::Mist => base_tokens(
            0xFFFFFFFF, 0xFF172123, 0xFFFFFFFF, 0xFF172123, 0xFF203033, 0xFFF7FCFC, 0xFFEDF5F5,
            0xFF203033, 0xFF667779, 0xFFDCE8E8, 0xFF667779,
        ),
        ThemePreset::Taupe => base_tokens(
            0xFFFFFFFF, 0xFF211D1B, 0xFFFFFFFF, 0xFF211D1B, 0xFF302A27, 0xFFFCFAF8, 0xFFF5F1EE,
            0xFF302A27, 0xFF7B716B, 0xFFE8E1DD, 0xFF7B716B,
        ),
    }
}

const fn dark_tokens(preset: ThemePreset) -> ColorTokens {
    match preset {
        ThemePreset::Zinc => dark_base_tokens(
            0xFF09090B, 0xFFFAFAFA, 0xFF18181B, 0xFFFAFAFA, 0xFFFAFAFA, 0xFF18181B, 0xFF27272A,
            0xFFFAFAFA, 0xFFA1A1AA, 0xFF27272A, 0xFFD4D4D8,
        ),
        ThemePreset::Neutral => dark_base_tokens(
            0xFF0A0A0A, 0xFFFAFAFA, 0xFF171717, 0xFFFAFAFA, 0xFFFAFAFA, 0xFF171717, 0xFF262626,
            0xFFFAFAFA, 0xFFA3A3A3, 0xFF262626, 0xFFD4D4D4,
        ),
        ThemePreset::Stone => dark_base_tokens(
            0xFF0C0A09, 0xFFFAFAF9, 0xFF1C1917, 0xFFFAFAF9, 0xFFFAFAF9, 0xFF1C1917, 0xFF292524,
            0xFFFAFAF9, 0xFFA8A29E, 0xFF292524, 0xFFD6D3D1,
        ),
        ThemePreset::Mauve => dark_base_tokens(
            0xFF121016, 0xFFFBF8FC, 0xFF211C27, 0xFFFBF8FC, 0xFFFBF8FC, 0xFF2E2633, 0xFF352C3A,
            0xFFFBF8FC, 0xFFB8ADBF, 0xFF352C3A, 0xFFD8CDDD,
        ),
        ThemePreset::Olive => dark_base_tokens(
            0xFF11140F, 0xFFFAFCF8, 0xFF1D241A, 0xFFFAFCF8, 0xFFFAFCF8, 0xFF283025, 0xFF30382B,
            0xFFFAFCF8, 0xFFAFB8A9, 0xFF30382B, 0xFFD0D8CA,
        ),
        ThemePreset::Mist => dark_base_tokens(
            0xFF0D1416, 0xFFF7FCFC, 0xFF182528, 0xFFF7FCFC, 0xFFF7FCFC, 0xFF203033, 0xFF283A3D,
            0xFFF7FCFC, 0xFFA7B8BA, 0xFF283A3D, 0xFFCADADB,
        ),
        ThemePreset::Taupe => dark_base_tokens(
            0xFF14110F, 0xFFFCFAF8, 0xFF241F1C, 0xFFFCFAF8, 0xFFFCFAF8, 0xFF302A27, 0xFF39312D,
            0xFFFCFAF8, 0xFFB8ADA7, 0xFF39312D, 0xFFD8CEC8,
        ),
    }
}

const fn base_tokens(
    background: u32,
    foreground: u32,
    card: u32,
    card_foreground: u32,
    primary: u32,
    primary_foreground: u32,
    secondary: u32,
    secondary_foreground: u32,
    muted_foreground: u32,
    border: u32,
    ring: u32,
) -> ColorTokens {
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
        destructive: color::DESTRUCTIVE,
        destructive_foreground: color::DESTRUCTIVE_FOREGROUND,
        border,
        input: border,
        ring,
        surface: background,
        chart_1: color::CHART_1,
        chart_2: color::CHART_2,
        chart_3: color::CHART_3,
        chart_4: color::CHART_4,
        chart_5: color::CHART_5,
        sidebar: secondary,
        sidebar_foreground: foreground,
        sidebar_primary: primary,
        sidebar_primary_foreground: primary_foreground,
        sidebar_accent: secondary,
        sidebar_accent_foreground: secondary_foreground,
        sidebar_border: border,
        sidebar_ring: ring,
    }
}

const fn dark_base_tokens(
    background: u32,
    foreground: u32,
    card: u32,
    card_foreground: u32,
    primary: u32,
    primary_foreground: u32,
    secondary: u32,
    secondary_foreground: u32,
    muted_foreground: u32,
    border: u32,
    ring: u32,
) -> ColorTokens {
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
        destructive: 0xFF7F1D1D,
        destructive_foreground: color::DESTRUCTIVE_FOREGROUND,
        border,
        input: border,
        ring,
        surface: background,
        chart_1: 0xFF3B82F6,
        chart_2: 0xFF10B981,
        chart_3: 0xFFF59E0B,
        chart_4: 0xFFA855F7,
        chart_5: 0xFFEF4444,
        sidebar: card,
        sidebar_foreground: foreground,
        sidebar_primary: primary,
        sidebar_primary_foreground: primary_foreground,
        sidebar_accent: secondary,
        sidebar_accent_foreground: secondary_foreground,
        sidebar_border: border,
        sidebar_ring: ring,
    }
}

pub mod color {
    pub const BACKGROUND: u32 = 0xFFFFFFFF;
    pub const FOREGROUND: u32 = 0xFF09090B;
    pub const CARD: u32 = 0xFFFFFFFF;
    pub const CARD_FOREGROUND: u32 = 0xFF09090B;
    pub const POPOVER: u32 = 0xFFFFFFFF;
    pub const POPOVER_FOREGROUND: u32 = 0xFF09090B;
    pub const PRIMARY: u32 = 0xFF09090B;
    pub const PRIMARY_FOREGROUND: u32 = 0xFFFAFAFA;
    pub const PRIMARY_TRACK: u32 = 0x3309090B;
    pub const SECONDARY: u32 = 0xFFF4F4F5;
    pub const SECONDARY_FOREGROUND: u32 = 0xFF09090B;
    pub const MUTED: u32 = 0xFFF4F4F5;
    pub const MUTED_FOREGROUND: u32 = 0xFF71717A;
    pub const ACCENT: u32 = 0xFFF4F4F5;
    pub const ACCENT_FOREGROUND: u32 = 0xFF09090B;
    pub const DESTRUCTIVE: u32 = 0xFFEF4444;
    pub const DESTRUCTIVE_FOREGROUND: u32 = 0xFFFAFAFA;
    pub const BORDER: u32 = 0xFFE4E4E7;
    pub const INPUT: u32 = 0xFFE4E4E7;
    pub const RING: u32 = 0xFF71717A;
    pub const SURFACE: u32 = 0xFFFFFFFF;
    pub const CHART_1: u32 = 0xFFE76E50;
    pub const CHART_2: u32 = 0xFF2A9D90;
    pub const CHART_3: u32 = 0xFF274754;
    pub const CHART_4: u32 = 0xFFE8C468;
    pub const CHART_5: u32 = 0xFFF4A462;
}

pub mod radius {
    // Keep these close to the Tailwind radii used by react-native-reusables:
    // `rounded`/small surfaces ~= 4, `rounded-md` = 6, `rounded-lg` = 8.
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 6.0;
    pub const LG: f32 = 8.0;
    pub const XL: f32 = 12.0;
    pub const XXL: f32 = 16.0;
    pub const FULL: f32 = 999.0;
}

pub mod spacing {
    pub const XXS: f32 = 4.0;
    pub const XS: f32 = 6.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 20.0;
    pub const XXL: f32 = 24.0;
}

pub mod typography {
    pub const XS: f32 = 12.0;
    pub const SM: f32 = 14.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 18.0;
    pub const XL: f32 = 20.0;
    pub const XXL: f32 = 24.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_matches_legacy_zinc_light_tokens() {
        let theme = Theme::default();

        assert_eq!(theme.mode, ThemeMode::Light);
        assert_eq!(theme.preset, Some(ThemePreset::Zinc));
        assert_eq!(theme.colors.background, color::BACKGROUND);
        assert_eq!(theme.colors.foreground, color::FOREGROUND);
        assert_eq!(theme.colors.primary_track, color::PRIMARY_TRACK);
        assert_eq!(theme.radii.md, radius::MD);
    }

    #[test]
    fn light_and_dark_presets_resolve_different_tokens() {
        let light = Theme::light(ThemePreset::Zinc);
        let dark = Theme::dark(ThemePreset::Zinc);

        assert_ne!(light.colors.background, dark.colors.background);
        assert_ne!(light.colors.foreground, dark.colors.foreground);
        assert_ne!(
            light.colors.primary_foreground,
            dark.colors.primary_foreground
        );
    }

    #[test]
    fn built_in_presets_expose_distinct_primary_palettes() {
        let zinc = Theme::light(ThemePreset::Zinc).colors;

        for preset in [
            ThemePreset::Neutral,
            ThemePreset::Stone,
            ThemePreset::Mauve,
            ThemePreset::Olive,
            ThemePreset::Mist,
            ThemePreset::Taupe,
        ] {
            let colors = Theme::light(preset).colors;

            assert_ne!(colors.primary, zinc.primary);
        }
    }

    #[test]
    fn scoped_theme_uses_nearest_value() {
        let outer = Theme::dark(ThemePreset::Stone);
        let inner = Theme::light(ThemePreset::Olive);

        assert_eq!(current(), Theme::default());
        with_theme(outer, || {
            assert_eq!(current(), outer);
            with_theme(inner, || {
                assert_eq!(current(), inner);
            });
            assert_eq!(current(), outer);
        });
        assert_eq!(current(), Theme::default());
    }

    #[test]
    fn custom_theme_exposes_custom_colors_and_radius() {
        let colors = Theme::light(ThemePreset::Neutral)
            .colors
            .with_surface(0xFF010203);
        let radii = RadiusTokens::from_base(10.0);
        let theme = Theme::custom(colors).with_radius(radii);

        with_theme(theme, || {
            assert_eq!(super::colors().surface, 0xFF010203);
            assert_eq!(super::radii().lg, 10.0);
            assert_eq!(super::radii().xl, 15.0);
        });
    }

    #[test]
    fn token_helpers_restore_default_after_rendering() {
        let theme = Theme::dark(ThemePreset::Stone);

        with_theme(theme, || {
            assert_eq!(super::colors().background, theme.colors.background);
        });

        assert_eq!(current(), Theme::default());
        assert_eq!(
            super::colors().background,
            Theme::default().colors.background
        );
    }

    #[test]
    fn alpha_helper_replaces_alpha_channel() {
        assert_eq!(with_alpha(0xFF112233, 0x80), 0x80112233);
    }
}
