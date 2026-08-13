//! ColorUI paint recipes and the kit mounted for nested headless primitives.

use arkit_component::appearance::{
    AlertAppearance, AlertStyleInput, AlertVariant, AvatarAppearance, AvatarStyleInput,
    BadgeAppearance, BadgeStyleInput, BadgeVariant, ButtonAppearance, ButtonSize, ButtonStyleInput,
    ButtonVariant, CardAppearance, CardStyleInput, CheckboxAppearance, CheckboxStyleInput,
    InputAppearance, InputStyleInput, LabelAppearance, LabelStyleInput, ProgressAppearance,
    ProgressStyleInput, SeparatorAppearance, SeparatorStyleInput, SkeletonAppearance,
    SkeletonStyleInput, SwitchAppearance, SwitchStyleInput,
};
use arkit_component::style::{StyleKit, Theme as TokenTheme};
use arkit_prelude::Signal;

use crate::spec;
use crate::theme::{swatch, ColorUiTheme};
use crate::PaletteColor;

const TRANSPARENT: u32 = 0x0000_0000;

pub(crate) fn resolve_color(input: Option<PaletteColor>, theme: &ColorUiTheme) -> PaletteColor {
    match input {
        Some(PaletteColor::Default) | None => theme.primary,
        Some(color) => color,
    }
}

pub(crate) fn colorui_button(
    theme: &ColorUiTheme,
    color: Option<PaletteColor>,
    size: ButtonSize,
    variant: ButtonVariant,
    line: bool,
    round: bool,
    block: bool,
    disabled: bool,
    width: Option<String>,
    height: Option<f32>,
    shadow: Option<bool>,
) -> ButtonAppearance {
    let icon = matches!(size, ButtonSize::Icon);
    let (base_height, pad_h, text_size) = match size {
        ButtonSize::Sm => (spec::BTN_HEIGHT_SM, spec::BTN_PAD_SM, spec::BTN_FONT_SM),
        ButtonSize::Default => (spec::BTN_HEIGHT, spec::BTN_PAD, spec::BTN_FONT),
        ButtonSize::Lg => (spec::BTN_HEIGHT_LG, spec::BTN_PAD_LG, spec::BTN_FONT_LG),
        ButtonSize::Icon => (spec::BTN_HEIGHT, 0.0, spec::BTN_FONT),
    };
    let tone = swatch(resolve_color(color, theme));
    let outline = line || matches!(variant, ButtonVariant::Outline);
    let (background, foreground, border_width, border_color, default_shadow) = match variant {
        ButtonVariant::Destructive => {
            let red = swatch(PaletteColor::Red);
            (red.fill, red.ink, 0.0, TRANSPARENT, true)
        }
        ButtonVariant::Ghost => (
            TRANSPARENT,
            theme.tokens().colors.foreground,
            0.0,
            TRANSPARENT,
            false,
        ),
        ButtonVariant::Link => (TRANSPARENT, tone.fill, 0.0, TRANSPARENT, false),
        ButtonVariant::Secondary => (spec::BG_GRAY, spec::INK_ON_GRAY, 0.0, TRANSPARENT, false),
        ButtonVariant::Outline | ButtonVariant::Default => {
            if outline {
                (TRANSPARENT, tone.fill, 1.0, tone.fill, false)
            } else if matches!(color, Some(PaletteColor::Gray) | Some(PaletteColor::Grey)) {
                (spec::BG_GRAY, spec::INK_ON_GRAY, 0.0, TRANSPARENT, false)
            } else {
                (tone.fill, tone.ink, 0.0, TRANSPARENT, true)
            }
        }
    };
    ButtonAppearance {
        height: height.unwrap_or(base_height),
        width: width.or_else(|| {
            if block {
                Some("100%".into())
            } else if icon {
                Some(format!("{base_height}"))
            } else {
                None
            }
        }),
        padding: if icon {
            [0.0, 0.0, 0.0, 0.0]
        } else {
            [0.0, pad_h, 0.0, pad_h]
        },
        text_size,
        font_weight: 400,
        background,
        foreground,
        border_width,
        border_color,
        border_radius: if round || icon { 999.0 } else { spec::RADIUS },
        shadow: shadow.unwrap_or(default_shadow),
        opacity: if disabled { 0.6 } else { 1.0 },
    }
}

pub(crate) fn colorui_card(theme: &ColorUiTheme, shadow: bool) -> CardAppearance {
    let tokens = theme.tokens();
    CardAppearance {
        background: tokens.colors.card,
        foreground: tokens.colors.card_foreground,
        border_color: tokens.colors.border,
        border_width: 0.0,
        radius: spec::RADIUS_CARD,
        shadow,
        header_padding: [spec::PADDING, spec::PADDING, spec::PADDING, spec::PADDING],
        content_padding: [0.0, spec::PADDING, spec::PADDING, spec::PADDING],
        footer_padding: [0.0, spec::PADDING, spec::PADDING, spec::PADDING],
        title_size: spec::TEXT_LG,
        title_weight: 700,
        title_line_height: 22.0,
        description_size: 14.0,
        description_color: tokens.colors.muted_foreground,
    }
}

pub(crate) fn colorui_switch(theme: &ColorUiTheme) -> SwitchAppearance {
    let tokens = theme.tokens();
    SwitchAppearance {
        width: spec::SWITCH_W,
        height: spec::SWITCH_H,
        selected: tokens.colors.primary,
        unselected: spec::SWITCH_OFF,
        knob: 0xFFFFFFFF,
        border_width: 0.0,
        border_color: TRANSPARENT,
        radius: tokens.radii.full,
    }
}

pub(crate) fn colorui_progress(
    theme: &ColorUiTheme,
    color: Option<PaletteColor>,
    height: Option<f32>,
) -> ProgressAppearance {
    let fill = swatch(resolve_color(color, theme)).fill;
    ProgressAppearance {
        height: height.unwrap_or(spec::PROGRESS_HEIGHT),
        radius: 999.0,
        track_color: spec::PROGRESS_TRACK,
        indicator_color: fill,
    }
}

pub(crate) fn colorui_badge(
    theme: &ColorUiTheme,
    color: Option<PaletteColor>,
    variant: BadgeVariant,
    line: bool,
    pill: bool,
) -> BadgeAppearance {
    let tone = swatch(resolve_color(color, theme));
    let (background, foreground, border_width, border_color) =
        if line || matches!(variant, BadgeVariant::Outline) {
            (TRANSPARENT, tone.fill, 1.0, tone.fill)
        } else if matches!(variant, BadgeVariant::Destructive) {
            let red = swatch(PaletteColor::Red);
            (red.fill, red.ink, 0.0, TRANSPARENT)
        } else if matches!(variant, BadgeVariant::Secondary) {
            (0xFFF1F1F1, 0xFF333333, 0.0, TRANSPARENT)
        } else {
            (tone.fill, tone.ink, 0.0, TRANSPARENT)
        };
    BadgeAppearance {
        background,
        foreground,
        border_width,
        border_color,
        radius: if pill { 999.0 } else { 0.0 },
        padding: [0.0, 8.0, 0.0, 8.0],
        min_height: spec::TAG_HEIGHT,
        font_size: spec::TAG_FONT,
        font_weight: 400,
        line_height: 16.0,
        icon_size: 12.0,
        icon_gap: 4.0,
    }
}

pub(crate) fn colorui_input(
    theme: &ColorUiTheme,
    invalid: bool,
    height: Option<f32>,
) -> InputAppearance {
    let tokens = theme.tokens();
    InputAppearance {
        height: height.unwrap_or(36.0),
        font_size: spec::TEXT_DF,
        line_height: 20.0,
        foreground: 0xFF555555,
        placeholder: spec::TEXT_MUTED,
        caret: tokens.colors.primary,
        background: tokens.colors.card,
        border_width: 1.0,
        border_color: if invalid {
            tokens.colors.destructive
        } else {
            0xFFEEEEEE
        },
        border_radius: 6.0,
        padding: [0.0, 12.0, 0.0, 12.0],
        password_trailing_padding: 40.0,
    }
}

pub(crate) fn colorui_checkbox(theme: &ColorUiTheme, checked: bool) -> CheckboxAppearance {
    let tokens = theme.tokens();
    CheckboxAppearance {
        size: spec::CHECK_RADIO,
        icon_size: 16.0,
        radius: spec::RADIUS,
        border_width: if checked { 0.0 } else { 1.0 },
        border_color: if checked {
            tokens.colors.primary
        } else {
            0xFFCCCCCC
        },
        background: if checked {
            tokens.colors.primary
        } else {
            0xFFFFFFFF
        },
        check_color: 0xFFFFFFFF,
        label_size: 14.0,
        label_color: tokens.colors.foreground,
        label_gap: 8.0,
    }
}

pub(crate) fn colorui_alert(theme: &ColorUiTheme, destructive: bool) -> AlertAppearance {
    if destructive {
        let red = swatch(PaletteColor::Red);
        AlertAppearance {
            background: red.light_fill,
            border_color: red.light_fill,
            border_width: 0.0,
            radius: spec::RADIUS,
            title_color: red.fill,
            description_color: red.fill,
            icon_color: red.fill,
        }
    } else {
        let tone = swatch(theme.primary);
        AlertAppearance {
            background: tone.light_fill,
            border_color: tone.light_fill,
            border_width: 0.0,
            radius: spec::RADIUS,
            title_color: tone.fill,
            description_color: theme.tokens().colors.foreground,
            icon_color: tone.fill,
        }
    }
}

pub(crate) fn colorui_separator(theme: &ColorUiTheme) -> SeparatorAppearance {
    SeparatorAppearance {
        color: theme.tokens().colors.border,
        thickness: 1.0,
    }
}

pub(crate) fn colorui_label(theme: &ColorUiTheme) -> LabelAppearance {
    LabelAppearance {
        font_size: 15.0,
        font_weight: 400,
        color: theme.tokens().colors.foreground,
        line_height: 20.0,
    }
}

pub(crate) fn colorui_skeleton(theme: &ColorUiTheme) -> SkeletonAppearance {
    SkeletonAppearance {
        fill: theme.tokens().colors.muted,
        radius: 4.0,
    }
}

pub(crate) fn colorui_avatar(
    theme: &ColorUiTheme,
    ring: bool,
    radius: Option<f32>,
) -> AvatarAppearance {
    AvatarAppearance {
        size: spec::AVATAR,
        radius: radius.unwrap_or(theme.tokens().radii.full),
        border_width: if ring { 2.0 } else { 0.0 },
        border_color: if ring { spec::PAGE_BG } else { TRANSPARENT },
        fallback_background: spec::AVATAR_FALLBACK,
        fallback_foreground: 0xFFFFFFFF,
        fallback_font_size: 16.0,
    }
}

pub(crate) fn colorui_button_from_input(
    theme: &ColorUiTheme,
    input: &ButtonStyleInput,
) -> ButtonAppearance {
    let mut appearance = colorui_button(
        theme,
        input.color,
        input.size,
        input.variant,
        matches!(input.variant, ButtonVariant::Outline),
        input.round,
        input.block,
        input.disabled,
        input.width.clone(),
        input.height,
        input.shadow,
    );
    if let Some(radius) = input.border_radius {
        appearance.border_radius = radius;
    }
    appearance
}

/// Kit mounted by [`crate::theme::use_colorui`] so nested headless primitives
/// (AlertDialogAction, Form submit, …) pick up ColorUI paint.
#[derive(Clone, Copy)]
pub struct ColorUiKit {
    pub theme: Signal<ColorUiTheme>,
}

impl StyleKit for ColorUiKit {
    fn theme(&self) -> TokenTheme {
        (self.theme)().tokens()
    }

    fn button(&self, input: &ButtonStyleInput) -> ButtonAppearance {
        colorui_button_from_input(&(self.theme)(), input)
    }

    fn badge(&self, input: &BadgeStyleInput) -> BadgeAppearance {
        colorui_badge(
            &(self.theme)(),
            input.color,
            input.variant,
            matches!(input.variant, BadgeVariant::Outline),
            input.pill,
        )
    }

    fn card(&self, input: &CardStyleInput) -> CardAppearance {
        colorui_card(&(self.theme)(), input.shadow)
    }

    fn input(&self, input: &InputStyleInput) -> InputAppearance {
        colorui_input(&(self.theme)(), input.invalid, input.height)
    }

    fn progress(&self, input: &ProgressStyleInput) -> ProgressAppearance {
        colorui_progress(&(self.theme)(), None, input.height)
    }

    fn avatar(&self, input: &AvatarStyleInput) -> AvatarAppearance {
        colorui_avatar(&(self.theme)(), input.ring, input.radius)
    }

    fn switch(&self, _input: &SwitchStyleInput) -> SwitchAppearance {
        colorui_switch(&(self.theme)())
    }

    fn checkbox(&self, input: &CheckboxStyleInput) -> CheckboxAppearance {
        colorui_checkbox(&(self.theme)(), input.checked)
    }

    fn alert(&self, input: &AlertStyleInput) -> AlertAppearance {
        colorui_alert(
            &(self.theme)(),
            matches!(input.variant, AlertVariant::Destructive),
        )
    }

    fn separator(&self, _input: &SeparatorStyleInput) -> SeparatorAppearance {
        colorui_separator(&(self.theme)())
    }

    fn label(&self, _input: &LabelStyleInput) -> LabelAppearance {
        colorui_label(&(self.theme)())
    }

    fn skeleton(&self, _input: &SkeletonStyleInput) -> SkeletonAppearance {
        colorui_skeleton(&(self.theme)())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_button_uses_theme_primary_green() {
        let theme = ColorUiTheme::light(PaletteColor::Green);
        let appearance = colorui_button(
            &theme,
            None,
            ButtonSize::Default,
            ButtonVariant::Default,
            false,
            false,
            false,
            false,
            None,
            None,
            None,
        );
        assert_eq!(appearance.background, 0xFF39B54A);
        assert_eq!(appearance.foreground, 0xFFFFFFFF);
        assert_eq!(appearance.height, 32.0);
        assert!(appearance.shadow);
    }

    #[test]
    fn explicit_palette_overrides_primary() {
        let theme = ColorUiTheme::light(PaletteColor::Green);
        let appearance = colorui_button(
            &theme,
            Some(PaletteColor::Red),
            ButtonSize::Default,
            ButtonVariant::Default,
            false,
            false,
            false,
            false,
            None,
            None,
            None,
        );
        assert_eq!(appearance.background, 0xFFE54D42);
    }

    #[test]
    fn gray_button_is_plain() {
        let theme = ColorUiTheme::light(PaletteColor::Green);
        let appearance = colorui_button(
            &theme,
            Some(PaletteColor::Gray),
            ButtonSize::Default,
            ButtonVariant::Default,
            false,
            false,
            false,
            false,
            None,
            None,
            None,
        );
        assert_eq!(appearance.background, 0xFFF0F0F0);
        assert!(!appearance.shadow);
    }

    #[test]
    fn checkbox_is_colorui_24px() {
        let theme = ColorUiTheme::light(PaletteColor::Green);
        let appearance = colorui_checkbox(&theme, true);
        assert_eq!(appearance.size, 24.0);
        assert_eq!(appearance.background, 0xFF39B54A);
    }

    #[test]
    fn progress_uses_colorui_track() {
        let theme = ColorUiTheme::light(PaletteColor::Green);
        let appearance = colorui_progress(&theme, None, None);
        assert_eq!(appearance.height, 14.0);
        assert_eq!(appearance.track_color, 0xFFEBEEF5);
        assert_eq!(appearance.indicator_color, 0xFF39B54A);
    }

    #[test]
    fn alert_uses_light_swatch() {
        let theme = ColorUiTheme::light(PaletteColor::Green);
        let appearance = colorui_alert(&theme, false);
        assert_eq!(appearance.background, 0xFFD7F0DB);
        assert_eq!(appearance.border_width, 0.0);
        assert_eq!(appearance.title_color, 0xFF39B54A);
    }
}
