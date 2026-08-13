//! Label — unstyled form label.

use crate::appearance::{LabelAppearance, LabelStyleInput};
use crate::style::use_style_kit;
use arkit_prelude::*;

/// Props for [`Label`].
#[derive(Props, Clone, PartialEq)]
pub struct LabelProps {
    pub content: String,
    #[props(default)]
    pub appearance: Option<LabelAppearance>,
}

/// A form label.
#[component]
pub fn Label(props: LabelProps) -> Element {
    let kit = use_style_kit();
    let appearance = props
        .appearance
        .unwrap_or_else(|| kit.label(&LabelStyleInput));
    rsx! {
        text {
            content: props.content.clone(),
            width: "100%",
            font_size: appearance.font_size,
            font_weight: appearance.font_weight,
            font_color: appearance.color,
            line_height: appearance.line_height,
            text_align: "start",
        }
    }
}
