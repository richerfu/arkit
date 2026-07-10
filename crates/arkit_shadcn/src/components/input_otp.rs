//! InputOtp — shadcn-style one-time-password input.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Renders `digits` 36x36 `TextInput` cells in a row; editing a cell
//! replaces the corresponding character of the OTP value and fires `on_change`
//! with the new value.

use crate::theme::*;
use arkit_prelude::*;

const OTP_CELL: f32 = 36.0;

/// Props for [`InputOtp`].
#[derive(Props, Clone, PartialEq)]
pub struct InputOtpProps {
    pub value: String,
    #[props(default = 6usize)]
    pub digits: usize,
    pub on_change: Option<EventHandler<String>>,
}

/// A one-time-password input composed of single-character cells.
#[component]
pub fn InputOtp(props: InputOtpProps) -> Element {
    let theme = use_theme();
    let digits = props.digits;
    let on_change = props.on_change;
    let value = props.value.clone();

    let cells: Vec<Element> = (0..digits)
        .map(|idx| {
            let v = value.clone();
            let oc = on_change;
            let ch = value
                .chars()
                .nth(idx)
                .map(|c| c.to_string())
                .unwrap_or_default();
            rsx! {
                textinput {
                    value: ch,
                    width: OTP_CELL,
                    height: OTP_CELL,
                    font_size: typography::SM,
                    border_style: super::ARKUI_BORDER_STYLE_SOLID,
                    border_width: 1.0,
                    border_color: theme.colors.input,
                    border_radius: theme.radii.md,
                    background_color: theme.colors.background,
                    padding_top: spacing::XXS,
                    padding_right: spacing::MD,
                    padding_bottom: spacing::XXS,
                    padding_left: spacing::MD,
                    on_change: move |evt| {
                        let next_ch = evt.data().string_value.chars().next().unwrap_or('\0');
                        let mut chars: Vec<char> = v.chars().collect();
                        while chars.len() < digits {
                            chars.push('\0');
                        }
                        if idx < chars.len() {
                            chars[idx] = next_ch;
                        }
                        let new_value: String =
                            chars.into_iter().filter(|c| *c != '\0').collect();
                        if let Some(handler) = oc {
                            handler.call(new_value);
                        }
                    },
                }
            }
        })
        .collect();

    rsx! {
        row {
            percent_width: 1.0,
            {cells.into_iter()}
        }
    }
}
