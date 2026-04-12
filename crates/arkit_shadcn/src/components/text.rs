use super::*;
use arkit::TextAlignment;

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
        .font_color(color::FOREGROUND)
        .line_height(24.0)
        .text_align(TextAlignment::Start)
}

pub fn text<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    text_with_variant(content, TextVariant::Default)
}

/// Equivalent to Tailwind `text-sm` (no extra weight / leading utilities).
pub fn text_sm<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_color(color::FOREGROUND)
        .line_height(20.0)
        .text_align(TextAlignment::Start)
        .into()
}

/// Equivalent to Tailwind `text-sm font-medium` (default `text-sm` line height).
pub fn text_sm_medium<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_weight(FontWeight::W500)
        .font_color(color::FOREGROUND)
        .line_height(20.0)
        .text_align(TextAlignment::Start)
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
            .font_weight(FontWeight::W700)
            .line_height(40.0)
            .text_letter_spacing(TRACKING_TIGHT)
            .text_align(TextAlignment::Center)
            .into(),
        TextVariant::H2 => arkit::column_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .children(vec![
                base_text::<Message>(content)
                    .font_size(30.0)
                    .font_weight(FontWeight::W600)
                    .line_height(36.0)
                    .text_letter_spacing(TRACKING_TIGHT)
                    .into(),
                arkit::row_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .height(1.0)
                    .margin_top(8.0)
                    .background_color(color::BORDER)
                    .into(),
            ])
            .into(),
        TextVariant::H3 => base_text::<Message>(content)
            .font_size(24.0)
            .font_weight(FontWeight::W600)
            .line_height(32.0)
            .text_letter_spacing(TRACKING_TIGHT)
            .into(),
        TextVariant::P => base_text::<Message>(content).line_height(28.0).into(),
        TextVariant::Blockquote => arkit::row_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .border_width([0.0, 0.0, 0.0, 2.0])
            .border_color(color::BORDER)
            .padding([0.0, 0.0, 0.0, 12.0])
            .children(vec![base_text::<Message>(content)
                .font_style(FontStyle::Italic)
                .line_height(24.0)
                .into()])
            .into(),
        TextVariant::Code => crate::styles::rounded(
            arkit::row_component::<Message, arkit::Theme>()
                .background_color(color::MUTED)
                .padding([5.0, 3.0])
                .children(vec![arkit::text::<Message, arkit::Theme>(content)
                    .font_size(typography::SM)
                    .font_family("monospace")
                    .font_weight(FontWeight::W600)
                    .font_color(color::FOREGROUND)
                    .line_height(18.0)
                    .into()]),
            radius::SM,
        )
        .into(),
        TextVariant::Lead => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::XL)
            .font_color(color::MUTED_FOREGROUND)
            .line_height(28.0)
            .text_align(TextAlignment::Start)
            .into(),
        TextVariant::Large => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::LG)
            .font_weight(FontWeight::W600)
            .font_color(color::FOREGROUND)
            .line_height(28.0)
            .text_letter_spacing(TRACKING_TIGHT)
            .text_align(TextAlignment::Start)
            .into(),
        TextVariant::Small => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::SM)
            .font_weight(FontWeight::W500)
            .font_color(color::FOREGROUND)
            .line_height(14.0)
            .text_align(TextAlignment::Start)
            .into(),
        TextVariant::Muted => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::SM)
            .font_color(color::MUTED_FOREGROUND)
            .line_height(20.0)
            .text_align(TextAlignment::Start)
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
