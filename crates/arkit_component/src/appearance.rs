//! Per-control visual recipes resolved by a [`crate::style::StyleKit`].
//!
//! Headless components pass semantic inputs; kits return concrete paint,
//! type, and geometry. [`unstyled_*`] helpers are the fallback used when no
//! style library is mounted.

use crate::style::{spacing, typography, with_alpha, PaletteColor, Theme};

const TRANSPARENT: u32 = 0x0000_0000;

/// Button visual variant. Kits map this onto their own language
/// (shadcn `default` / ColorUI `bg-*` / …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
}

/// Button size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Icon,
}

/// Badge visual variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
}

/// Alert visual variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
}

/// Inputs the button primitive sends to a style kit.
#[derive(Debug, Clone)]
pub struct ButtonStyleInput {
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub disabled: bool,
    pub color: Option<PaletteColor>,
    pub round: bool,
    pub block: bool,
    pub height: Option<f32>,
    pub border_radius: Option<f32>,
    pub width: Option<String>,
    pub shadow: Option<bool>,
}

/// Resolved button paint and geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonAppearance {
    pub height: f32,
    pub width: Option<String>,
    pub padding: [f32; 4],
    pub text_size: f32,
    pub font_weight: u32,
    pub background: u32,
    pub foreground: u32,
    pub border_width: f32,
    pub border_color: u32,
    pub border_radius: f32,
    pub shadow: bool,
    pub opacity: f32,
}

#[derive(Debug, Clone)]
pub struct BadgeStyleInput {
    pub variant: BadgeVariant,
    pub pill: bool,
    pub color: Option<PaletteColor>,
    pub icon_colors: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BadgeAppearance {
    pub background: u32,
    pub foreground: u32,
    pub border_width: f32,
    pub border_color: u32,
    pub radius: f32,
    pub padding: [f32; 4],
    pub min_height: f32,
    pub font_size: f32,
    pub font_weight: u32,
    pub line_height: f32,
    pub icon_size: f32,
    pub icon_gap: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CardStyleInput {
    pub shadow: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardAppearance {
    pub background: u32,
    pub foreground: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub radius: f32,
    pub shadow: bool,
    pub header_padding: [f32; 4],
    pub content_padding: [f32; 4],
    pub footer_padding: [f32; 4],
    pub title_size: f32,
    pub title_weight: u32,
    pub title_line_height: f32,
    pub description_size: f32,
    pub description_color: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct InputStyleInput {
    pub invalid: bool,
    pub disabled: bool,
    pub read_only: bool,
    pub password: bool,
    pub height: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputAppearance {
    pub height: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub foreground: u32,
    pub placeholder: u32,
    pub caret: u32,
    pub background: u32,
    pub border_width: f32,
    pub border_color: u32,
    pub border_radius: f32,
    pub padding: [f32; 4],
    pub password_trailing_padding: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ProgressStyleInput {
    pub height: Option<f32>,
    pub track_color: Option<u32>,
    pub indicator_color: Option<u32>,
    pub radius: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressAppearance {
    pub height: f32,
    pub radius: f32,
    pub track_color: u32,
    pub indicator_color: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct AvatarStyleInput {
    pub ring: bool,
    pub radius: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvatarAppearance {
    pub size: f32,
    pub radius: f32,
    pub border_width: f32,
    pub border_color: u32,
    pub fallback_background: u32,
    pub fallback_foreground: u32,
    pub fallback_font_size: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct SwitchStyleInput {
    pub checked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchAppearance {
    pub width: f32,
    pub height: f32,
    pub selected: u32,
    pub unselected: u32,
    pub knob: u32,
    pub border_width: f32,
    pub border_color: u32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CheckboxStyleInput {
    pub checked: bool,
    pub checked_color: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxAppearance {
    pub size: f32,
    pub icon_size: f32,
    pub radius: f32,
    pub border_width: f32,
    pub border_color: u32,
    pub background: u32,
    pub check_color: u32,
    pub label_size: f32,
    pub label_color: u32,
    pub label_gap: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct AlertStyleInput {
    pub variant: AlertVariant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlertAppearance {
    pub background: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub radius: f32,
    pub title_color: u32,
    pub description_color: u32,
    pub icon_color: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SeparatorStyleInput {
    pub vertical: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeparatorAppearance {
    pub color: u32,
    pub thickness: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct LabelStyleInput;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelAppearance {
    pub font_size: f32,
    pub font_weight: u32,
    pub color: u32,
    pub line_height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct SkeletonStyleInput {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkeletonAppearance {
    pub fill: u32,
    pub radius: f32,
}

fn size_geometry(size: ButtonSize) -> (f32, Option<f32>, [f32; 4], f32) {
    match size {
        ButtonSize::Default => (48.0, None, [12.0, 20.0, 12.0, 20.0], typography::MD),
        ButtonSize::Sm => (36.0, None, [0.0, 12.0, 0.0, 12.0], typography::MD),
        ButtonSize::Lg => (56.0, None, [0.0, 32.0, 0.0, 32.0], typography::LG),
        ButtonSize::Icon => (40.0, Some(40.0), [0.0, 0.0, 0.0, 0.0], typography::MD),
    }
}

/// Structural button fallback: no elevation, token colors only.
pub fn unstyled_button(input: &ButtonStyleInput, theme: &Theme) -> ButtonAppearance {
    let (height, width, padding, text_size) = size_geometry(input.size);
    let (background, foreground, border_width, border_color) = match input.variant {
        ButtonVariant::Default | ButtonVariant::Secondary => (
            theme.colors.secondary,
            theme.colors.foreground,
            0.0,
            TRANSPARENT,
        ),
        ButtonVariant::Outline => (
            TRANSPARENT,
            theme.colors.foreground,
            1.0,
            theme.colors.border,
        ),
        ButtonVariant::Ghost | ButtonVariant::Link => {
            (TRANSPARENT, theme.colors.foreground, 0.0, TRANSPARENT)
        }
        ButtonVariant::Destructive => (
            TRANSPARENT,
            theme.colors.destructive,
            1.0,
            theme.colors.destructive,
        ),
    };
    ButtonAppearance {
        height: input.height.unwrap_or(height),
        width: input
            .width
            .clone()
            .or_else(|| width.map(|w| format!("{w}"))),
        padding,
        text_size,
        font_weight: 500,
        background,
        foreground,
        border_width,
        border_color,
        border_radius: input.border_radius.unwrap_or(if input.round {
            theme.radii.full
        } else {
            theme.radii.md
        }),
        shadow: input.shadow.unwrap_or(false),
        opacity: if input.disabled { 0.5 } else { 1.0 },
    }
}

pub fn unstyled_badge(input: &BadgeStyleInput, theme: &Theme) -> BadgeAppearance {
    let (background, foreground, border_color) = if let Some((bg, fg)) = input.icon_colors {
        (bg, fg, TRANSPARENT)
    } else {
        match input.variant {
            BadgeVariant::Default | BadgeVariant::Secondary => {
                (theme.colors.secondary, theme.colors.foreground, TRANSPARENT)
            }
            BadgeVariant::Destructive => (
                TRANSPARENT,
                theme.colors.destructive,
                theme.colors.destructive,
            ),
            BadgeVariant::Outline => (TRANSPARENT, theme.colors.foreground, theme.colors.border),
        }
    };
    let hpad = if input.pill {
        spacing::XXS
    } else {
        spacing::SM
    };
    BadgeAppearance {
        background,
        foreground,
        border_width: 1.0,
        border_color,
        radius: if input.pill {
            theme.radii.full
        } else {
            theme.radii.md
        },
        padding: [2.0, hpad, 2.0, hpad],
        min_height: 22.0,
        font_size: typography::XS,
        font_weight: 500,
        line_height: 16.0,
        icon_size: 12.0,
        icon_gap: 4.0,
    }
}

pub fn unstyled_card(input: &CardStyleInput, theme: &Theme) -> CardAppearance {
    CardAppearance {
        background: theme.colors.card,
        foreground: theme.colors.card_foreground,
        border_color: theme.colors.border,
        border_width: 1.0,
        radius: theme.radii.lg,
        shadow: input.shadow,
        header_padding: [spacing::XXL, spacing::XXL, spacing::XXL, spacing::XXL],
        content_padding: [0.0, spacing::XXL, spacing::XXL, spacing::XXL],
        footer_padding: [0.0, spacing::XXL, spacing::XXL, spacing::XXL],
        title_size: typography::XXL,
        title_weight: 600,
        title_line_height: 24.0,
        description_size: typography::SM,
        description_color: theme.colors.muted_foreground,
    }
}

pub fn unstyled_input(input: &InputStyleInput, theme: &Theme) -> InputAppearance {
    InputAppearance {
        height: input.height.unwrap_or(48.0),
        font_size: typography::LG,
        line_height: 22.5,
        foreground: theme.colors.foreground,
        placeholder: with_alpha(theme.colors.muted_foreground, 0x80),
        caret: if input.read_only {
            TRANSPARENT
        } else {
            theme.colors.primary
        },
        background: theme.colors.background,
        border_width: 1.0,
        border_color: if input.invalid {
            theme.colors.destructive
        } else {
            theme.colors.input
        },
        border_radius: theme.radii.md,
        padding: [
            spacing::XXS,
            if input.password { 48.0 } else { spacing::MD },
            spacing::XXS,
            spacing::MD,
        ],
        password_trailing_padding: 48.0,
    }
}

pub fn unstyled_progress(input: &ProgressStyleInput, theme: &Theme) -> ProgressAppearance {
    ProgressAppearance {
        height: input
            .height
            .filter(|h| h.is_finite() && *h > 0.0)
            .unwrap_or(8.0),
        radius: input
            .radius
            .filter(|r| r.is_finite() && *r >= 0.0)
            .unwrap_or(theme.radii.full),
        track_color: input.track_color.unwrap_or(theme.colors.primary_track),
        indicator_color: input.indicator_color.unwrap_or(theme.colors.primary),
    }
}

pub fn unstyled_avatar(input: &AvatarStyleInput, theme: &Theme) -> AvatarAppearance {
    AvatarAppearance {
        size: 32.0,
        radius: input.radius.unwrap_or(theme.radii.full),
        border_width: if input.ring { 2.0 } else { 0.0 },
        border_color: if input.ring {
            theme.colors.background
        } else {
            TRANSPARENT
        },
        fallback_background: theme.colors.muted,
        fallback_foreground: theme.colors.muted_foreground,
        fallback_font_size: typography::SM,
    }
}

pub fn unstyled_switch(_input: &SwitchStyleInput, theme: &Theme) -> SwitchAppearance {
    SwitchAppearance {
        width: 32.0,
        height: 18.4,
        selected: theme.colors.primary,
        unselected: theme.colors.input,
        knob: theme.colors.background,
        border_width: 1.0,
        border_color: TRANSPARENT,
        radius: theme.radii.full,
    }
}

pub fn unstyled_checkbox(input: &CheckboxStyleInput, theme: &Theme) -> CheckboxAppearance {
    let checked_color = input.checked_color.unwrap_or(theme.colors.primary);
    CheckboxAppearance {
        size: 16.0,
        icon_size: 16.0,
        radius: theme.radii.sm,
        border_width: 1.0,
        border_color: checked_color,
        background: if input.checked {
            checked_color
        } else {
            theme.colors.background
        },
        check_color: theme.colors.primary_foreground,
        label_size: typography::SM,
        label_color: theme.colors.foreground,
        label_gap: spacing::SM,
    }
}

pub fn unstyled_alert(input: &AlertStyleInput, theme: &Theme) -> AlertAppearance {
    match input.variant {
        AlertVariant::Default => AlertAppearance {
            background: theme.colors.card,
            border_color: theme.colors.border,
            border_width: 1.0,
            radius: theme.radii.lg,
            title_color: theme.colors.foreground,
            description_color: theme.colors.muted_foreground,
            icon_color: theme.colors.foreground,
        },
        AlertVariant::Destructive => AlertAppearance {
            background: theme.colors.card,
            border_color: with_alpha(theme.colors.destructive, 0x80),
            border_width: 1.0,
            radius: theme.radii.lg,
            title_color: theme.colors.destructive,
            description_color: with_alpha(theme.colors.destructive, 0xE6),
            icon_color: theme.colors.destructive,
        },
    }
}

pub fn unstyled_separator(_input: &SeparatorStyleInput, theme: &Theme) -> SeparatorAppearance {
    SeparatorAppearance {
        color: theme.colors.border,
        thickness: 1.0,
    }
}

pub fn unstyled_label(_input: &LabelStyleInput, theme: &Theme) -> LabelAppearance {
    LabelAppearance {
        font_size: typography::SM,
        font_weight: 500,
        color: theme.colors.foreground,
        line_height: 14.0,
    }
}

pub fn unstyled_skeleton(input: &SkeletonStyleInput, theme: &Theme) -> SkeletonAppearance {
    let circular = (input.width - input.height).abs() < f32::EPSILON && input.width >= 40.0;
    SkeletonAppearance {
        fill: with_alpha(theme.colors.primary, 0x1A),
        radius: if circular {
            theme.radii.full
        } else {
            theme.radii.md
        },
    }
}
