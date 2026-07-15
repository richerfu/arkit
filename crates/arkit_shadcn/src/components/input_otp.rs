//! Mobile-first shadcn Input OTP.
//!
//! A single native `TextInput` owns the complete code, so paste, deletion,
//! keyboard suggestions, and one-time-code autofill behave as one field. The
//! visible slots are a presentation layer over that input, matching shadcn's
//! grouped-slot composition without splitting input state across native nodes.

use crate::icon::icon_placeholder;
use crate::theme::*;
use arkit_prelude::*;

use super::ARKUI_BORDER_STYLE_SOLID;

const DEFAULT_CELL_SIZE: f32 = 48.0;
const MIN_CELL_SIZE: f32 = 44.0;
const DEFAULT_SEPARATOR_WIDTH: f32 = 28.0;
const NATIVE_INPUT_TYPE_NORMAL: i32 = 0;
const NATIVE_INPUT_TYPE_NUMBER: i32 = 2;
const NATIVE_INPUT_TYPE_ONE_TIME_CODE: i32 = 14;

/// Accepted character set and native keyboard profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputOtpMode {
    /// Digits only, using the native numeric keyboard.
    #[default]
    Numeric,
    /// ASCII letters and digits, using the normal text keyboard.
    Alphanumeric,
    /// ASCII letters and digits with Harmony's one-time-code input profile.
    /// Some IMEs present a full keyboard for this profile.
    OneTimeCode,
}

impl InputOtpMode {
    const fn native_input_type(self) -> i32 {
        match self {
            Self::Numeric => NATIVE_INPUT_TYPE_NUMBER,
            Self::Alphanumeric => NATIVE_INPUT_TYPE_NORMAL,
            Self::OneTimeCode => NATIVE_INPUT_TYPE_ONE_TIME_CODE,
        }
    }

    const fn native_input_filter(self) -> &'static str {
        match self {
            Self::Numeric => "[0-9]",
            Self::Alphanumeric | Self::OneTimeCode => "[0-9A-Za-z]",
        }
    }

    fn accepts(self, character: char) -> bool {
        match self {
            Self::Numeric => character.is_ascii_digit(),
            Self::Alphanumeric | Self::OneTimeCode => character.is_ascii_alphanumeric(),
        }
    }
}

/// Visual separator inserted between OTP groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputOtpSeparator {
    None,
    #[default]
    Dash,
}

/// Optional mobile slot styling overrides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputOtpStyle {
    /// Square slot size. Values below 44vp are raised to the mobile touch target.
    pub cell_size: f32,
    /// Width reserved for a separator between groups.
    pub separator_width: f32,
    pub border_radius: Option<f32>,
    pub background_color: Option<u32>,
    pub foreground_color: Option<u32>,
    pub border_color: Option<u32>,
    pub active_border_color: Option<u32>,
    pub caret_color: Option<u32>,
}

impl Default for InputOtpStyle {
    fn default() -> Self {
        Self {
            cell_size: DEFAULT_CELL_SIZE,
            separator_width: DEFAULT_SEPARATOR_WIDTH,
            border_radius: None,
            background_color: None,
            foreground_color: None,
            border_color: None,
            active_border_color: None,
            caret_color: None,
        }
    }
}

/// Props for [`InputOtp`].
#[derive(Props, Clone, PartialEq)]
pub struct InputOtpProps {
    /// Controlled complete OTP value.
    pub value: String,
    #[props(default = 6usize)]
    pub digits: usize,
    #[props(default)]
    pub mode: InputOtpMode,
    /// Number of slots per group. Set to zero for one uninterrupted group.
    #[props(default = 3usize)]
    pub group_size: usize,
    #[props(default)]
    pub separator: InputOtpSeparator,
    #[props(default)]
    pub disabled: bool,
    /// Applies destructive validation styling to every slot.
    #[props(default)]
    pub invalid: bool,
    /// Replaces entered characters with bullets, useful for PIN entry.
    #[props(default)]
    pub masked: bool,
    #[props(default)]
    pub style: InputOtpStyle,
    #[props(default)]
    pub on_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_complete: Option<EventHandler<String>>,
}

/// A controlled OTP/PIN input with native paste and autofill behavior.
#[component]
pub fn InputOtp(props: InputOtpProps) -> Element {
    let theme = use_theme();
    let mut focused = use_signal(|| false);
    let digits = props.digits.max(1);
    let value = sanitize_otp(&props.value, digits, props.mode);
    let value_length = value.chars().count();
    let active_index = value_length.min(digits - 1);
    let cell_size = props.style.cell_size.max(MIN_CELL_SIZE);
    let separator_width = props.style.separator_width.max(0.0);
    let separator_count = otp_separator_count(digits, props.group_size, props.separator);
    let total_width = (digits as f32 * cell_size) + (separator_count as f32 * separator_width);
    let radius = props.style.border_radius.unwrap_or(theme.radii.md);
    let background = props
        .style
        .background_color
        .unwrap_or(theme.colors.background);
    let foreground = props
        .style
        .foreground_color
        .unwrap_or(theme.colors.foreground);
    let default_border = props.style.border_color.unwrap_or(theme.colors.input);
    let active_border = props.style.active_border_color.unwrap_or(theme.colors.ring);
    let caret = props.style.caret_color.unwrap_or(theme.colors.primary);
    let border = if props.invalid {
        theme.colors.destructive
    } else {
        default_border
    };
    let active_border = if props.invalid {
        theme.colors.destructive
    } else {
        active_border
    };
    let characters = value.chars().collect::<Vec<_>>();
    let mut slots = Vec::<Element>::with_capacity(digits + separator_count);

    for index in 0..digits {
        let (starts_group, ends_group) = otp_group_edges(index, digits, props.group_size);

        if index > 0 && starts_group && props.separator == InputOtpSeparator::Dash {
            slots.push(rsx! {
                row {
                    key: "separator-{index}",
                    width: separator_width,
                    height: cell_size,
                    align_items: "center",
                    justify_content: "center",
                    hit_test_behavior: 2_i32,
                    {icon_placeholder("minus", 16.0, theme.colors.muted_foreground)}
                }
            });
        }

        let character = characters.get(index).copied();
        let is_active = index == active_index && focused() && !props.disabled;
        let left_radius = if starts_group { radius } else { 0.0 };
        let right_radius = if ends_group { radius } else { 0.0 };
        let slot_radius = format!("{left_radius},{right_radius},{left_radius},{right_radius}");
        let slot_border_width = if is_active {
            String::from("2,2,2,2")
        } else if starts_group {
            String::from("1,1,1,1")
        } else {
            String::from("1,1,1,0")
        };
        let displayed = character.map(|character| {
            if props.masked {
                String::from("•")
            } else {
                character.to_string()
            }
        });

        slots.push(rsx! {
            stack {
                key: "slot-{index}",
                width: cell_size,
                height: cell_size,
                alignment: 4_i32,
                background_color: background,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                border_width: slot_border_width,
                border_color: if is_active { active_border } else { border },
                border_radius: slot_radius,
                clip: true,
                hit_test_behavior: 2_i32,
                if let Some(displayed) = displayed {
                    text {
                        content: displayed,
                        font_size: typography::LG,
                        font_weight: 500_i32,
                        font_color: foreground,
                        line_height: 24.0,
                        text_align: 1_i32,
                        hit_test_behavior: 2_i32,
                    }
                } else if is_active {
                    row {
                        width: 1.5,
                        height: 24.0,
                        border_radius: theme.radii.full,
                        background_color: caret,
                        hit_test_behavior: 2_i32,
                    }
                }
            }
        });
    }

    let current_value = value.clone();
    let on_change = props.on_change;
    let on_complete = props.on_complete;
    let input_type = props.mode.native_input_type();
    let input_filter = props.mode.native_input_filter();
    let max_length = i32::try_from(digits).unwrap_or(i32::MAX);

    rsx! {
        stack {
            width: total_width,
            height: cell_size,
            alignment: 4_i32,
            opacity: if props.disabled { 0.5_f32 } else { 1.0_f32 },
            row {
                width: total_width,
                height: cell_size,
                align_items: "center",
                justify_content: "center",
                hit_test_behavior: 2_i32,
                {slots.into_iter()}
            }
            // One native field owns all input. Keeping it nearly transparent
            // preserves focus, paste, delete, and platform OTP suggestions
            // while the slots above remain the visual source of truth.
            textinput {
                value,
                input_type,
                input_filter,
                max_length,
                width: total_width,
                height: cell_size,
                padding: 0.0,
                font_size: typography::LG,
                font_color: 0x00000000_u32,
                caret_color: 0x00000000_u32,
                background_color: 0x00000000_u32,
                border_width: 0.0,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                text_align: 1_i32,
                opacity: 0.01_f32,
                enabled: !props.disabled,
                on_focus: move |event| focused.set(event.data().focused),
                on_blur: move |event| focused.set(event.data().focused),
                on_change: move |event| {
                    let next = sanitize_otp(&event.data().string_value, digits, props.mode);
                    if next == current_value {
                        return;
                    }
                    if let Some(handler) = on_change {
                        handler.call(next.clone());
                    }
                    if next.chars().count() == digits {
                        if let Some(handler) = on_complete {
                            handler.call(next);
                        }
                    }
                },
            }
        }
    }
}

fn sanitize_otp(value: &str, digits: usize, mode: InputOtpMode) -> String {
    value
        .chars()
        .filter(|character| mode.accepts(*character))
        .take(digits)
        .collect()
}

fn otp_separator_count(digits: usize, group_size: usize, separator: InputOtpSeparator) -> usize {
    if separator == InputOtpSeparator::None || group_size == 0 || digits <= group_size {
        0
    } else {
        (digits - 1) / group_size
    }
}

fn otp_group_edges(index: usize, digits: usize, group_size: usize) -> (bool, bool) {
    let starts_group = index == 0 || (group_size > 0 && index.is_multiple_of(group_size));
    let ends_group =
        index + 1 == digits || (group_size > 0 && (index + 1).is_multiple_of(group_size));
    (starts_group, ends_group)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_input_filters_and_truncates_pasted_content() {
        assert_eq!(
            sanitize_otp("12 a3-4567", 6, InputOtpMode::Numeric),
            "123456"
        );
    }

    #[test]
    fn alphanumeric_input_preserves_ascii_letter_case() {
        assert_eq!(
            sanitize_otp("A-b9_2", 4, InputOtpMode::Alphanumeric),
            "Ab92"
        );
    }

    #[test]
    fn grouping_counts_only_boundaries_between_slots() {
        assert_eq!(otp_separator_count(6, 3, InputOtpSeparator::Dash), 1);
        assert_eq!(otp_separator_count(8, 3, InputOtpSeparator::Dash), 2);
        assert_eq!(otp_separator_count(4, 0, InputOtpSeparator::Dash), 0);
        assert_eq!(otp_separator_count(6, 3, InputOtpSeparator::None), 0);
    }

    #[test]
    fn zero_group_size_creates_one_joined_group() {
        assert_eq!(otp_group_edges(0, 4, 0), (true, false));
        assert_eq!(otp_group_edges(1, 4, 0), (false, false));
        assert_eq!(otp_group_edges(3, 4, 0), (false, true));
        assert_eq!(otp_group_edges(2, 6, 3), (false, true));
        assert_eq!(otp_group_edges(3, 6, 3), (true, false));
    }

    #[test]
    fn mobile_defaults_use_native_numeric_input() {
        let style = InputOtpStyle::default();
        assert_eq!(style.cell_size, 48.0);
        assert_eq!(InputOtpMode::default().native_input_type(), 2);
        assert_eq!(InputOtpMode::default().native_input_filter(), "[0-9]");
        assert_eq!(
            InputOtpMode::Alphanumeric.native_input_filter(),
            "[0-9A-Za-z]"
        );
        assert_eq!(InputOtpMode::OneTimeCode.native_input_type(), 14);
        assert_eq!(InputOtpSeparator::default(), InputOtpSeparator::Dash);
    }
}
