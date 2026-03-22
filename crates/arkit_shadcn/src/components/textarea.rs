use super::*;

pub fn textarea(placeholder: impl Into<String>) -> TextAreaElement {
    input_surface(arkit::text_area_component())
        .background_color(0x00000000)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![spacing::SM, spacing::MD, spacing::SM, spacing::MD],
        )
        .style(
            ArkUINodeAttributeType::TextAreaPlaceholder,
            placeholder.into(),
        )
        .style(
            ArkUINodeAttributeType::TextAreaPlaceholderColor,
            color::MUTED_FOREGROUND,
        )
        .font_size(typography::MD)
        .height(64.0)
}
