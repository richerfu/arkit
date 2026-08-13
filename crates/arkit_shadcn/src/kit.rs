//! Shadcn appearances mounted onto `arkit_component` primitives.

use arkit_component::appearance::{
    AlertAppearance, AlertStyleInput, AlertVariant, AvatarAppearance, AvatarStyleInput,
    BadgeAppearance, BadgeStyleInput, BadgeVariant, ButtonAppearance, ButtonSize, ButtonStyleInput,
    ButtonVariant, CardAppearance, CardStyleInput, CheckboxAppearance, CheckboxStyleInput,
    InputAppearance, InputStyleInput, LabelAppearance, LabelStyleInput, ProgressAppearance,
    ProgressStyleInput, SeparatorAppearance, SeparatorStyleInput, SkeletonAppearance,
    SkeletonStyleInput, SwitchAppearance, SwitchStyleInput,
};
use arkit_component::style::{with_alpha, StyleKit, Theme as TokenTheme};
use arkit_prelude::Signal;

use crate::spec;
use crate::theme::Theme;

const TRANSPARENT: u32 = 0x0000_0000;

/// Style kit that paints primitives with the current shadcn theme signal.
#[derive(Clone, Copy)]
pub struct ShadcnKit {
    pub theme: Signal<Theme>,
}

impl StyleKit for ShadcnKit {
    fn theme(&self) -> TokenTheme {
        (self.theme)().tokens()
    }

    fn button(&self, input: &ButtonStyleInput) -> ButtonAppearance {
        shadcn_button(input, &(self.theme)())
    }

    fn badge(&self, input: &BadgeStyleInput) -> BadgeAppearance {
        shadcn_badge(input, &(self.theme)())
    }

    fn card(&self, input: &CardStyleInput) -> CardAppearance {
        shadcn_card(input, &(self.theme)())
    }

    fn input(&self, input: &InputStyleInput) -> InputAppearance {
        shadcn_input(input, &(self.theme)())
    }

    fn progress(&self, input: &ProgressStyleInput) -> ProgressAppearance {
        shadcn_progress(input, &(self.theme)())
    }

    fn avatar(&self, input: &AvatarStyleInput) -> AvatarAppearance {
        shadcn_avatar(input, &(self.theme)())
    }

    fn switch(&self, input: &SwitchStyleInput) -> SwitchAppearance {
        shadcn_switch(input, &(self.theme)())
    }

    fn checkbox(&self, input: &CheckboxStyleInput) -> CheckboxAppearance {
        shadcn_checkbox(input, &(self.theme)())
    }

    fn alert(&self, input: &AlertStyleInput) -> AlertAppearance {
        shadcn_alert(input, &(self.theme)())
    }

    fn separator(&self, input: &SeparatorStyleInput) -> SeparatorAppearance {
        shadcn_separator(input, &(self.theme)())
    }

    fn label(&self, input: &LabelStyleInput) -> LabelAppearance {
        shadcn_label(input, &(self.theme)())
    }

    fn skeleton(&self, input: &SkeletonStyleInput) -> SkeletonAppearance {
        shadcn_skeleton(input, &(self.theme)())
    }
}

pub fn button_appearance(input: &ButtonStyleInput, theme: &Theme) -> ButtonAppearance {
    shadcn_button(input, theme)
}

pub fn badge_appearance(input: &BadgeStyleInput, theme: &Theme) -> BadgeAppearance {
    shadcn_badge(input, theme)
}

pub fn alert_appearance(input: &AlertStyleInput, theme: &Theme) -> AlertAppearance {
    shadcn_alert(input, theme)
}

pub fn avatar_appearance(input: &AvatarStyleInput, theme: &Theme) -> AvatarAppearance {
    shadcn_avatar(input, theme)
}

pub fn card_appearance(input: &CardStyleInput, theme: &Theme) -> CardAppearance {
    shadcn_card(input, theme)
}

pub fn input_appearance(input: &InputStyleInput, theme: &Theme) -> InputAppearance {
    shadcn_input(input, theme)
}

pub fn progress_appearance(input: &ProgressStyleInput, theme: &Theme) -> ProgressAppearance {
    shadcn_progress(input, theme)
}

pub fn switch_appearance(input: &SwitchStyleInput, theme: &Theme) -> SwitchAppearance {
    shadcn_switch(input, theme)
}

pub fn checkbox_appearance(input: &CheckboxStyleInput, theme: &Theme) -> CheckboxAppearance {
    shadcn_checkbox(input, theme)
}

pub fn separator_appearance(input: &SeparatorStyleInput, theme: &Theme) -> SeparatorAppearance {
    shadcn_separator(input, theme)
}

pub fn label_appearance(input: &LabelStyleInput, theme: &Theme) -> LabelAppearance {
    shadcn_label(input, theme)
}

pub fn skeleton_appearance(input: &SkeletonStyleInput, theme: &Theme) -> SkeletonAppearance {
    shadcn_skeleton(input, theme)
}

fn shadcn_button(input: &ButtonStyleInput, theme: &Theme) -> ButtonAppearance {
    let (height, width, padding, text_size) = match input.size {
        ButtonSize::Default => (
            spec::BTN_HEIGHT,
            None,
            [12.0, spec::BTN_PX, 12.0, spec::BTN_PX],
            spec::TEXT_BASE,
        ),
        ButtonSize::Sm => (
            spec::BTN_HEIGHT_SM,
            None,
            [0.0, spec::BTN_PX_SM, 0.0, spec::BTN_PX_SM],
            spec::TEXT_BASE,
        ),
        ButtonSize::Lg => (
            spec::BTN_HEIGHT_LG,
            None,
            [0.0, spec::BTN_PX_LG, 0.0, spec::BTN_PX_LG],
            spec::TEXT_LG,
        ),
        ButtonSize::Icon => (
            spec::BTN_ICON,
            Some(spec::BTN_ICON),
            [0.0, 0.0, 0.0, 0.0],
            spec::TEXT_BASE,
        ),
    };
    let (background, foreground, border_width, border_color, shadow) = match input.variant {
        ButtonVariant::Default => (
            theme.colors.primary,
            theme.colors.primary_foreground,
            0.0,
            TRANSPARENT,
            true,
        ),
        ButtonVariant::Secondary => (
            theme.colors.secondary,
            theme.colors.secondary_foreground,
            0.0,
            TRANSPARENT,
            true,
        ),
        ButtonVariant::Outline => (
            theme.colors.background,
            theme.colors.foreground,
            1.0,
            theme.colors.border,
            true,
        ),
        ButtonVariant::Ghost => (
            TRANSPARENT,
            theme.colors.foreground,
            0.0,
            TRANSPARENT,
            false,
        ),
        ButtonVariant::Destructive => (
            theme.colors.destructive,
            theme.colors.destructive_foreground,
            0.0,
            TRANSPARENT,
            true,
        ),
        ButtonVariant::Link => (TRANSPARENT, theme.colors.primary, 0.0, TRANSPARENT, false),
    };
    ButtonAppearance {
        height: input.height.unwrap_or(height),
        width: input
            .width
            .clone()
            .or_else(|| width.map(|w| format!("{w}"))),
        padding,
        text_size,
        font_weight: spec::FONT_MEDIUM,
        background,
        foreground,
        border_width,
        border_color,
        border_radius: input.border_radius.unwrap_or(if input.round {
            theme.radii.full
        } else {
            spec::RADIUS_MD
        }),
        shadow: input.shadow.unwrap_or(shadow),
        opacity: if input.disabled {
            spec::DISABLED_OPACITY
        } else {
            1.0
        },
    }
}

fn shadcn_badge(input: &BadgeStyleInput, theme: &Theme) -> BadgeAppearance {
    let (background, foreground, border_color) = if let Some((bg, fg)) = input.icon_colors {
        (bg, fg, TRANSPARENT)
    } else {
        match input.variant {
            BadgeVariant::Default => (
                theme.colors.primary,
                theme.colors.primary_foreground,
                TRANSPARENT,
            ),
            BadgeVariant::Secondary => (
                theme.colors.secondary,
                theme.colors.secondary_foreground,
                TRANSPARENT,
            ),
            BadgeVariant::Destructive => (
                theme.colors.destructive,
                theme.colors.destructive_foreground,
                TRANSPARENT,
            ),
            BadgeVariant::Outline => (
                theme.colors.background,
                theme.colors.foreground,
                theme.colors.border,
            ),
        }
    };
    let hpad = if input.pill { 4.0 } else { 8.0 };
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
        min_height: spec::BADGE_H,
        font_size: spec::BADGE_FONT,
        font_weight: spec::FONT_MEDIUM,
        line_height: 16.0,
        icon_size: 12.0,
        icon_gap: 4.0,
    }
}

fn shadcn_avatar(input: &AvatarStyleInput, theme: &Theme) -> AvatarAppearance {
    AvatarAppearance {
        size: spec::AVATAR,
        radius: input.radius.unwrap_or(theme.radii.full),
        border_width: if input.ring { 2.0 } else { 0.0 },
        border_color: if input.ring {
            theme.colors.background
        } else {
            TRANSPARENT
        },
        fallback_background: theme.colors.muted,
        fallback_foreground: theme.colors.muted_foreground,
        fallback_font_size: spec::TEXT_SM,
    }
}

fn shadcn_card(input: &CardStyleInput, theme: &Theme) -> CardAppearance {
    CardAppearance {
        background: theme.colors.card,
        foreground: theme.colors.card_foreground,
        border_color: theme.colors.border,
        border_width: 1.0,
        radius: spec::RADIUS_XL,
        shadow: input.shadow,
        header_padding: [
            spec::DIALOG_PAD,
            spec::DIALOG_PAD,
            spec::DIALOG_PAD,
            spec::DIALOG_PAD,
        ],
        content_padding: [0.0, spec::DIALOG_PAD, spec::DIALOG_PAD, spec::DIALOG_PAD],
        footer_padding: [0.0, spec::DIALOG_PAD, spec::DIALOG_PAD, spec::DIALOG_PAD],
        title_size: spec::TEXT_XL,
        title_weight: spec::FONT_SEMIBOLD,
        title_line_height: 24.0,
        description_size: spec::TEXT_SM,
        description_color: theme.colors.muted_foreground,
    }
}

fn shadcn_input(input: &InputStyleInput, theme: &Theme) -> InputAppearance {
    InputAppearance {
        height: input.height.unwrap_or(spec::INPUT_H),
        font_size: spec::TEXT_BASE,
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
        border_radius: spec::RADIUS_MD,
        padding: [
            4.0,
            if input.password { spec::INPUT_H } else { 12.0 },
            4.0,
            12.0,
        ],
        password_trailing_padding: spec::INPUT_H,
    }
}

fn shadcn_progress(input: &ProgressStyleInput, theme: &Theme) -> ProgressAppearance {
    ProgressAppearance {
        height: input
            .height
            .filter(|h| h.is_finite() && *h > 0.0)
            .unwrap_or(spec::PROGRESS_H),
        radius: input
            .radius
            .filter(|r| r.is_finite() && *r >= 0.0)
            .unwrap_or(theme.radii.full),
        track_color: input.track_color.unwrap_or(theme.colors.primary_track),
        indicator_color: input.indicator_color.unwrap_or(theme.colors.primary),
    }
}

fn shadcn_switch(_input: &SwitchStyleInput, theme: &Theme) -> SwitchAppearance {
    SwitchAppearance {
        width: spec::SWITCH_W,
        height: spec::SWITCH_H,
        selected: theme.colors.primary,
        unselected: theme.colors.input,
        knob: theme.colors.background,
        border_width: 1.0,
        border_color: TRANSPARENT,
        radius: theme.radii.full,
    }
}

fn shadcn_checkbox(input: &CheckboxStyleInput, theme: &Theme) -> CheckboxAppearance {
    CheckboxAppearance {
        size: spec::CHECK,
        icon_size: 12.0,
        radius: 4.0,
        border_width: if input.checked { 0.0 } else { 1.0 },
        border_color: if input.checked {
            input.checked_color.unwrap_or(theme.colors.primary)
        } else {
            theme.colors.input
        },
        background: if input.checked {
            input.checked_color.unwrap_or(theme.colors.primary)
        } else {
            theme.colors.background
        },
        check_color: theme.colors.primary_foreground,
        label_size: spec::TEXT_SM,
        label_color: theme.colors.foreground,
        label_gap: 8.0,
    }
}

fn shadcn_separator(_input: &SeparatorStyleInput, theme: &Theme) -> SeparatorAppearance {
    SeparatorAppearance {
        color: theme.colors.border,
        thickness: 1.0,
    }
}

fn shadcn_label(_input: &LabelStyleInput, theme: &Theme) -> LabelAppearance {
    LabelAppearance {
        font_size: spec::TEXT_SM,
        font_weight: spec::FONT_MEDIUM,
        color: theme.colors.foreground,
        line_height: 20.0,
    }
}

fn shadcn_skeleton(_input: &SkeletonStyleInput, theme: &Theme) -> SkeletonAppearance {
    SkeletonAppearance {
        fill: theme.colors.muted,
        radius: spec::RADIUS_MD,
    }
}

fn shadcn_alert(input: &AlertStyleInput, theme: &Theme) -> AlertAppearance {
    match input.variant {
        AlertVariant::Default => AlertAppearance {
            background: theme.colors.card,
            border_color: theme.colors.border,
            border_width: 1.0,
            radius: spec::RADIUS_LG,
            title_color: theme.colors.foreground,
            description_color: theme.colors.muted_foreground,
            icon_color: theme.colors.foreground,
        },
        AlertVariant::Destructive => AlertAppearance {
            background: theme.colors.card,
            border_color: with_alpha(theme.colors.destructive, 0x80),
            border_width: 1.0,
            radius: spec::RADIUS_LG,
            title_color: theme.colors.destructive,
            description_color: with_alpha(theme.colors.destructive, 0xE6),
            icon_color: theme.colors.destructive,
        },
    }
}
