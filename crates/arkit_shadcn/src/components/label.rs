use super::*;
use arkit::ohos_arkui_binding::types::text_alignment::TextAlignment;

pub fn label(content: impl Into<String>) -> TextElement {
    arkit::text(content)
        .font_size(typography::SM)
        .style(ArkUINodeAttributeType::FontWeight, 4_i32)
        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
        .style(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Start),
        )
}
