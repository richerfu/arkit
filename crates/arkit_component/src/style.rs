//! Style-kit mount point and token contract.
//!
//! Components never own a brand palette. They read a [`Theme`] snapshot from
//! the nearest [`StyleKit`] (via [`use_theme`]) and resolve per-control
//! recipes through [`StyleKitHandle`]. Style crates mount a kit with
//! [`StyleProvider`] or [`use_style_provider`].

use std::rc::Rc;

use arkit_prelude::*;
use dioxus_core_macro::{component, Props};

use crate::appearance::{
    unstyled_alert, unstyled_avatar, unstyled_badge, unstyled_button, unstyled_card,
    unstyled_checkbox, unstyled_input, unstyled_label, unstyled_progress, unstyled_separator,
    unstyled_skeleton, unstyled_switch, AlertAppearance, AlertStyleInput, AvatarAppearance,
    AvatarStyleInput, BadgeAppearance, BadgeStyleInput, ButtonAppearance, ButtonStyleInput,
    CardAppearance, CardStyleInput, CheckboxAppearance, CheckboxStyleInput, InputAppearance,
    InputStyleInput, LabelAppearance, LabelStyleInput, ProgressAppearance, ProgressStyleInput,
    SeparatorAppearance, SeparatorStyleInput, SkeletonAppearance, SkeletonStyleInput,
    SwitchAppearance, SwitchStyleInput,
};

/// Light / dark token set. Style kits decide what each mode means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

/// Named palette hint consumed by style kits that expose a color system
/// (ColorUI `bg-blue`, `line-red`, …). Headless components treat this as an
/// opaque input; unstyled kits ignore it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaletteColor {
    #[default]
    Default,
    Red,
    Orange,
    Yellow,
    Olive,
    Green,
    Cyan,
    Blue,
    Purple,
    Mauve,
    Pink,
    Brown,
    Grey,
    Gray,
    Black,
    White,
}

/// Semantic color roles. Values are supplied by the mounted style kit.
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

/// Corner-radius scale. Style kits fill this; components only consume it.
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

/// Token snapshot read by components. Brand palettes live in style crates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub mode: ThemeMode,
    pub colors: ColorTokens,
    pub radii: RadiusTokens,
}

impl Theme {
    pub const fn new(mode: ThemeMode, colors: ColorTokens, radii: RadiusTokens) -> Self {
        Self {
            mode,
            colors,
            radii,
        }
    }

    pub const fn custom(colors: ColorTokens) -> Self {
        Self {
            mode: ThemeMode::Light,
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

    pub const fn unstyled() -> Self {
        Self {
            mode: ThemeMode::Light,
            colors: ColorTokens {
                background: color::BACKGROUND,
                foreground: color::FOREGROUND,
                card: color::CARD,
                card_foreground: color::CARD_FOREGROUND,
                popover: color::POPOVER,
                popover_foreground: color::POPOVER_FOREGROUND,
                primary: color::PRIMARY,
                primary_foreground: color::PRIMARY_FOREGROUND,
                primary_track: color::PRIMARY_TRACK,
                secondary: color::SECONDARY,
                secondary_foreground: color::SECONDARY_FOREGROUND,
                muted: color::MUTED,
                muted_foreground: color::MUTED_FOREGROUND,
                accent: color::ACCENT,
                accent_foreground: color::ACCENT_FOREGROUND,
                destructive: color::DESTRUCTIVE,
                destructive_foreground: color::DESTRUCTIVE_FOREGROUND,
                border: color::BORDER,
                input: color::INPUT,
                ring: color::RING,
                surface: color::SURFACE,
                chart_1: color::CHART_1,
                chart_2: color::CHART_2,
                chart_3: color::CHART_3,
                chart_4: color::CHART_4,
                chart_5: color::CHART_5,
                sidebar: color::SECONDARY,
                sidebar_foreground: color::FOREGROUND,
                sidebar_primary: color::PRIMARY,
                sidebar_primary_foreground: color::PRIMARY_FOREGROUND,
                sidebar_accent: color::SECONDARY,
                sidebar_accent_foreground: color::SECONDARY_FOREGROUND,
                sidebar_border: color::BORDER,
                sidebar_ring: color::RING,
            },
            radii: RadiusTokens {
                sm: 0.0,
                md: 2.0,
                lg: 2.0,
                xl: 4.0,
                xxl: 4.0,
                full: 999.0,
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::unstyled()
    }
}

pub const fn with_alpha(color: u32, alpha: u8) -> u32 {
    (color & 0x00FF_FFFF) | ((alpha as u32) << 24)
}

/// Neutral fallback roles. Not a brand palette — style kits replace these.
pub mod color {
    pub const BACKGROUND: u32 = 0xFFFFFFFF;
    pub const FOREGROUND: u32 = 0xFF111111;
    pub const CARD: u32 = 0xFFFFFFFF;
    pub const CARD_FOREGROUND: u32 = 0xFF111111;
    pub const POPOVER: u32 = 0xFFFFFFFF;
    pub const POPOVER_FOREGROUND: u32 = 0xFF111111;
    pub const PRIMARY: u32 = 0xFF333333;
    pub const PRIMARY_FOREGROUND: u32 = 0xFFFAFAFA;
    pub const PRIMARY_TRACK: u32 = 0x33333333;
    pub const SECONDARY: u32 = 0xFFF4F4F5;
    pub const SECONDARY_FOREGROUND: u32 = 0xFF111111;
    pub const MUTED: u32 = 0xFFF4F4F5;
    pub const MUTED_FOREGROUND: u32 = 0xFF71717A;
    pub const ACCENT: u32 = 0xFFF4F4F5;
    pub const ACCENT_FOREGROUND: u32 = 0xFF111111;
    pub const DESTRUCTIVE: u32 = 0xFFDC2626;
    pub const DESTRUCTIVE_FOREGROUND: u32 = 0xFFFAFAFA;
    pub const BORDER: u32 = 0xFFE4E4E7;
    pub const INPUT: u32 = 0xFFE4E4E7;
    pub const RING: u32 = 0xFF71717A;
    pub const SURFACE: u32 = 0xFFFFFFFF;
    pub const CHART_1: u32 = 0xFF3B82F6;
    pub const CHART_2: u32 = 0xFF10B981;
    pub const CHART_3: u32 = 0xFFF59E0B;
    pub const CHART_4: u32 = 0xFFA855F7;
    pub const CHART_5: u32 = 0xFFEF4444;
}

pub mod radius {
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

/// Recipe surface mounted by a style library.
pub trait StyleKit: 'static {
    fn theme(&self) -> Theme;

    fn button(&self, input: &ButtonStyleInput) -> ButtonAppearance {
        unstyled_button(input, &self.theme())
    }

    fn badge(&self, input: &BadgeStyleInput) -> BadgeAppearance {
        unstyled_badge(input, &self.theme())
    }

    fn card(&self, input: &CardStyleInput) -> CardAppearance {
        unstyled_card(input, &self.theme())
    }

    fn input(&self, input: &InputStyleInput) -> InputAppearance {
        unstyled_input(input, &self.theme())
    }

    fn progress(&self, input: &ProgressStyleInput) -> ProgressAppearance {
        unstyled_progress(input, &self.theme())
    }

    fn avatar(&self, input: &AvatarStyleInput) -> AvatarAppearance {
        unstyled_avatar(input, &self.theme())
    }

    fn switch(&self, input: &SwitchStyleInput) -> SwitchAppearance {
        unstyled_switch(input, &self.theme())
    }

    fn checkbox(&self, input: &CheckboxStyleInput) -> CheckboxAppearance {
        unstyled_checkbox(input, &self.theme())
    }

    fn alert(&self, input: &AlertStyleInput) -> AlertAppearance {
        unstyled_alert(input, &self.theme())
    }

    fn separator(&self, input: &SeparatorStyleInput) -> SeparatorAppearance {
        unstyled_separator(input, &self.theme())
    }

    fn label(&self, input: &LabelStyleInput) -> LabelAppearance {
        unstyled_label(input, &self.theme())
    }

    fn skeleton(&self, input: &SkeletonStyleInput) -> SkeletonAppearance {
        unstyled_skeleton(input, &self.theme())
    }
}

/// Neutral kit used when nothing is mounted.
#[derive(Clone, Copy, Default)]
pub struct UnstyledKit;

impl StyleKit for UnstyledKit {
    fn theme(&self) -> Theme {
        Theme::unstyled()
    }
}

/// Kit that only overrides tokens. Used for local surfaces such as toast.
#[derive(Clone, Copy)]
pub struct StaticKit(pub Theme);

impl StyleKit for StaticKit {
    fn theme(&self) -> Theme {
        self.0
    }
}

/// Cloneable handle stored in Dioxus context.
#[derive(Clone)]
pub struct StyleKitHandle {
    inner: Rc<dyn StyleKit>,
}

impl PartialEq for StyleKitHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl StyleKitHandle {
    pub fn new(kit: impl StyleKit + 'static) -> Self {
        Self {
            inner: Rc::new(kit),
        }
    }

    pub fn unstyled() -> Self {
        Self::new(UnstyledKit)
    }

    pub fn theme(&self) -> Theme {
        self.inner.theme()
    }

    pub fn button(&self, input: &ButtonStyleInput) -> ButtonAppearance {
        self.inner.button(input)
    }

    pub fn badge(&self, input: &BadgeStyleInput) -> BadgeAppearance {
        self.inner.badge(input)
    }

    pub fn card(&self, input: &CardStyleInput) -> CardAppearance {
        self.inner.card(input)
    }

    pub fn input(&self, input: &InputStyleInput) -> InputAppearance {
        self.inner.input(input)
    }

    pub fn progress(&self, input: &ProgressStyleInput) -> ProgressAppearance {
        self.inner.progress(input)
    }

    pub fn avatar(&self, input: &AvatarStyleInput) -> AvatarAppearance {
        self.inner.avatar(input)
    }

    pub fn switch(&self, input: &SwitchStyleInput) -> SwitchAppearance {
        self.inner.switch(input)
    }

    pub fn checkbox(&self, input: &CheckboxStyleInput) -> CheckboxAppearance {
        self.inner.checkbox(input)
    }

    pub fn alert(&self, input: &AlertStyleInput) -> AlertAppearance {
        self.inner.alert(input)
    }

    pub fn separator(&self, input: &SeparatorStyleInput) -> SeparatorAppearance {
        self.inner.separator(input)
    }

    pub fn label(&self, input: &LabelStyleInput) -> LabelAppearance {
        self.inner.label(input)
    }

    pub fn skeleton(&self, input: &SkeletonStyleInput) -> SkeletonAppearance {
        self.inner.skeleton(input)
    }
}

/// Install a style kit for the current Dioxus subtree. Prefer [`StyleProvider`]
/// at the app root.
///
/// The handle is stored in a `Signal` so descendants re-resolve on kit swap.
/// Call this from the same component that renders the styled children (or from
/// a parent of those children). `try_use_context` is the wrong lookup here: it
/// caches the first miss as [`UnstyledKit`] for the rest of the scope lifetime.
pub fn use_style_provider(kit: StyleKitHandle) -> Signal<StyleKitHandle> {
    use_context_provider(move || Signal::new(kit))
}

/// Read the mounted style kit. Falls back to [`UnstyledKit`].
///
/// Looks up context on every render (same pattern as shadcn `use_theme`) so a
/// provider that mounts after the first child render still takes effect.
pub fn use_style_kit() -> StyleKitHandle {
    dioxus_core::try_consume_context::<Signal<StyleKitHandle>>()
        .map(|sig| sig())
        .unwrap_or_else(StyleKitHandle::unstyled)
}

/// Read the active token snapshot.
///
/// Order: nearest `Signal<Theme>` (style-crate wrappers / ThemeProvider
/// tokens), then the mounted [`StyleKit`], then [`Theme::unstyled`].
pub fn use_theme() -> Theme {
    dioxus_core::try_consume_context::<Signal<Theme>>()
        .map(|sig| sig())
        .unwrap_or_else(|| use_style_kit().theme())
}

/// Publish a token snapshot for the current subtree. Style wrappers use this
/// so headless descendants pick up shadcn / ColorUI colors without a kit.
pub fn use_token_provider(tokens: Theme) -> Signal<Theme> {
    let mut provided = use_context_provider(move || Signal::new(tokens));
    if *provided.peek() != tokens {
        provided.set(tokens);
    }
    provided
}

/// Mount a style kit for `children`.
#[component]
pub fn StyleProvider(kit: StyleKitHandle, children: Element) -> Element {
    let _ = use_style_provider(kit);
    rsx! { {children} }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unstyled_theme_has_no_brand_radius() {
        let theme = Theme::unstyled();
        assert_eq!(theme.radii.md, 2.0);
        assert_eq!(theme.colors.primary, color::PRIMARY);
    }

    #[test]
    fn alpha_helper_replaces_alpha_channel() {
        assert_eq!(with_alpha(0xFF112233, 0x80), 0x80112233);
    }
}
