use super::*;
use arkit::ohos_arkui_binding::types::text_alignment::TextAlignment;

const TRACKING_TIGHT: f32 = -0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextVariant {
    Default,
    H1,
    H2,
    H3,
    P,
    Blockquote,
    Code,
    Lead,
    Large,
    Small,
    Muted,
}

fn base_text<Message: 'static>(content: impl Into<String>) -> TextElement<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::MD)
        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
        .style(ArkUINodeAttributeType::TextLineHeight, 24.0)
        .style(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Start),
        )
}

pub fn text<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    text_with_variant(content, TextVariant::Default)
}

/// Equivalent to Tailwind `text-sm` (no extra weight / leading utilities).
pub fn text_sm<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
        .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
        .style(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Start),
        )
        .into()
}

/// Equivalent to Tailwind `text-sm font-medium` (default `text-sm` line height).
pub fn text_sm_medium<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .style(ArkUINodeAttributeType::FontWeight, 4_i32)
        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
        .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
        .style(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Start),
        )
        .into()
}

pub fn text_with_variant<Message: 'static>(
    content: impl Into<String>,
    variant: TextVariant,
) -> Element<Message> {
    let content = content.into();
    match variant {
        TextVariant::Default => base_text::<Message>(content).into(),
        TextVariant::H1 => base_text::<Message>(content)
            .font_size(36.0)
            .style(ArkUINodeAttributeType::FontWeight, 6_i32)
            .style(ArkUINodeAttributeType::TextLineHeight, 40.0)
            .style(ArkUINodeAttributeType::TextLetterSpacing, TRACKING_TIGHT)
            .style(
                ArkUINodeAttributeType::TextAlign,
                i32::from(TextAlignment::Center),
            )
            .into(),
        TextVariant::H2 => arkit::column_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .children(vec![
                base_text::<Message>(content)
                    .font_size(30.0)
                    .style(ArkUINodeAttributeType::FontWeight, 5_i32)
                    .style(ArkUINodeAttributeType::TextLineHeight, 36.0)
                    .style(ArkUINodeAttributeType::TextLetterSpacing, TRACKING_TIGHT)
                    .into(),
                arkit::row_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .height(1.0)
                    .style(ArkUINodeAttributeType::Margin, vec![8.0, 0.0, 0.0, 0.0])
                    .background_color(color::BORDER)
                    .into(),
            ])
            .into(),
        TextVariant::H3 => base_text::<Message>(content)
            .font_size(24.0)
            .style(ArkUINodeAttributeType::FontWeight, 5_i32)
            .style(ArkUINodeAttributeType::TextLineHeight, 32.0)
            .style(ArkUINodeAttributeType::TextLetterSpacing, TRACKING_TIGHT)
            .into(),
        TextVariant::P => base_text::<Message>(content)
            .style(ArkUINodeAttributeType::TextLineHeight, 28.0)
            .into(),
        TextVariant::Blockquote => arkit::row_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![0.0, 0.0, 0.0, 2.0],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
            .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 12.0])
            .children(vec![base_text::<Message>(content)
                .style(ArkUINodeAttributeType::FontStyle, 1_i32)
                .style(ArkUINodeAttributeType::TextLineHeight, 24.0)
                .into()])
            .into(),
        TextVariant::Code => crate::styles::rounded(
            arkit::row_component::<Message, arkit::Theme>()
                .background_color(color::MUTED)
                .style(ArkUINodeAttributeType::Padding, vec![3.0, 5.0, 3.0, 5.0])
                .children(vec![arkit::text::<Message, arkit::Theme>(content)
                    .font_size(typography::SM)
                    .style(ArkUINodeAttributeType::FontFamily, "monospace")
                    .style(ArkUINodeAttributeType::FontWeight, 5_i32)
                    .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                    .style(ArkUINodeAttributeType::TextLineHeight, 18.0)
                    .into()]),
            radius::SM,
        )
        .into(),
        TextVariant::Lead => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::XL)
            .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 28.0)
            .style(
                ArkUINodeAttributeType::TextAlign,
                i32::from(TextAlignment::Start),
            )
            .into(),
        TextVariant::Large => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::LG)
            .style(ArkUINodeAttributeType::FontWeight, 5_i32)
            .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 28.0)
            .style(ArkUINodeAttributeType::TextLetterSpacing, TRACKING_TIGHT)
            .style(
                ArkUINodeAttributeType::TextAlign,
                i32::from(TextAlignment::Start),
            )
            .into(),
        TextVariant::Small => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::SM)
            .style(ArkUINodeAttributeType::FontWeight, 4_i32)
            .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 14.0)
            .style(
                ArkUINodeAttributeType::TextAlign,
                i32::from(TextAlignment::Start),
            )
            .into(),
        TextVariant::Muted => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::SM)
            .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
            .style(
                ArkUINodeAttributeType::TextAlign,
                i32::from(TextAlignment::Start),
            )
            .into(),
    }
}

pub fn text_variant<Message: 'static>(content: impl Into<String>, muted: bool) -> Element<Message> {
    text_with_variant(
        content,
        if muted {
            TextVariant::Muted
        } else {
            TextVariant::Default
        },
    )
}
