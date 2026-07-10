//! Shared pure style values and small text fragments for Dioxus components.
//! Each helper takes a `&Theme` resolved by [`crate::theme::use_theme`].

use crate::theme::{spacing, typography, Theme};
use arkit_prelude::*;

/// 4-float padding tuple `[top, right, bottom, left]` (ArkUI order).
pub type Padding4 = [f32; 4];

/// Resolved surface style (background, foreground, border, radius, shadow,
/// padding).
///
/// Mirrors the composition of the legacy `card_surface` / `input_surface` /
/// `panel_surface` helpers: `shadow_sm` + `rounded(r)` + `border` +
/// background/foreground fill + padding.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceStyle {
    pub background: u32,
    pub foreground: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub radius: f32,
    pub shadow: i32,
    pub padding: Padding4,
}

/// `shadow-sm` — the legacy `ShadowStyle::OuterDefaultSm`. In the rsx renderer
/// `shadow: 1` maps to `OuterDefaultSm`; `shadow: 0` is none.
pub fn shadow_sm() -> i32 {
    1
}

/// `card_surface` — `shadow_sm` + `rounded(lg)` + 1px `border` + `bg=card` +
/// `fg=card_foreground`. Padding belongs to CardHeader/CardContent/CardFooter.
pub fn card_surface(theme: &Theme) -> SurfaceStyle {
    SurfaceStyle {
        background: theme.colors.card,
        foreground: theme.colors.card_foreground,
        border_color: theme.colors.border,
        border_width: 1.0,
        radius: theme.radii.lg,
        shadow: shadow_sm(),
        padding: [0.0, 0.0, 0.0, 0.0],
    }
}

/// `input_surface` — `input_shadow_sm` + `rounded(md)` + 1px `border` +
/// `bg=background` + `fg=foreground` + padding `[XXS, MD, XXS, MD]`
/// (legacy `padding_xy(MD, XXS)` → `[top=XXS, right=MD, bottom=XXS, left=MD]`).
pub fn input_surface(theme: &Theme) -> SurfaceStyle {
    SurfaceStyle {
        background: theme.colors.background,
        foreground: theme.colors.foreground,
        border_color: theme.colors.border,
        border_width: 1.0,
        radius: theme.radii.md,
        shadow: shadow_sm(),
        padding: [spacing::XXS, spacing::MD, spacing::XXS, spacing::MD],
    }
}

/// `panel_surface` — `shadow_sm` + `rounded(md)` + 1px `border` +
/// `bg=popover` + `fg=popover_foreground`.
pub fn panel_surface(theme: &Theme) -> SurfaceStyle {
    SurfaceStyle {
        background: theme.colors.popover,
        foreground: theme.colors.popover_foreground,
        border_color: theme.colors.border,
        border_width: 1.0,
        radius: theme.radii.md,
        shadow: shadow_sm(),
        padding: [0.0, 0.0, 0.0, 0.0],
    }
}

/// `title_text` — `text-lg` (`LG`), `W600`, foreground, 20px leading, start.
pub fn title_text(content: impl Into<String>, theme: &Theme) -> Element {
    let content = content.into();
    rsx! {
        text {
            content: content,
            font_size: typography::LG,
            font_weight: 600,
            font_color: theme.colors.foreground,
            line_height: 20.0,
            text_align: 0,
        }
    }
}

/// `body_text` — `text-sm` (`SM`), `W500`, foreground, 20px leading, start.
pub fn body_text(content: impl Into<String>, theme: &Theme) -> Element {
    let content = content.into();
    rsx! {
        text {
            content: content,
            font_size: typography::SM,
            font_weight: 500,
            font_color: theme.colors.foreground,
            line_height: 20.0,
            text_align: 0,
        }
    }
}

/// `body_text_regular` — `text-md` (`MD`), normal weight, foreground, 20px
/// leading, start.
pub fn body_text_regular(content: impl Into<String>, theme: &Theme) -> Element {
    let content = content.into();
    rsx! {
        text {
            content: content,
            font_size: typography::MD,
            font_color: theme.colors.foreground,
            line_height: 20.0,
            text_align: 0,
        }
    }
}

/// `muted_text` — `text-sm` (`SM`), normal weight, muted foreground, 20px
/// leading, start.
pub fn muted_text(content: impl Into<String>, theme: &Theme) -> Element {
    let content = content.into();
    rsx! {
        text {
            content: content,
            font_size: typography::SM,
            font_color: theme.colors.muted_foreground,
            line_height: 20.0,
            text_align: 0,
        }
    }
}

/// `card_padding` — `[0, XXL, 0, XXL]` horizontal padding used by card
/// surfaces.
pub fn card_padding() -> Padding4 {
    [0.0, spacing::XXL, 0.0, spacing::XXL]
}

/// `with_alpha`-style helper kept for ergonomic alpha colors.
pub fn alpha(color: u32, a: u8) -> u32 {
    crate::theme::with_alpha(color, a)
}

/// Whether the disabled opacity should apply.
pub fn disabled_opacity(disabled: bool) -> f32 {
    if disabled {
        0.5
    } else {
        1.0
    }
}
