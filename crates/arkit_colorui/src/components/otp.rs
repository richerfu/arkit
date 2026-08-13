//! Input OTP — form-group cells, primary caret.

use arkit_component::components::{InputOtp as HeadlessOtp, InputOtpProps, InputOtpStyle};
use arkit_prelude::*;

use crate::spec;
use crate::theme::{swatch, use_colorui_theme};

#[component]
pub fn InputOtp(mut props: InputOtpProps) -> Element {
    if props.style == InputOtpStyle::default() {
        let fill = swatch(use_colorui_theme().primary).fill;
        props.style = InputOtpStyle {
            cell_size: spec::BTN_HEIGHT_LG,
            border_radius: Some(spec::RADIUS),
            background_color: Some(spec::BG_WHITE),
            foreground_color: Some(spec::TEXT),
            border_color: Some(spec::FORM_LINE),
            active_border_color: Some(fill),
            caret_color: Some(fill),
            ..InputOtpStyle::default()
        };
    }
    super::paint::forward(HeadlessOtp, props, "InputOtp")
}
