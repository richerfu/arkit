use arkit::ohos_arkui_binding::types::text_alignment::TextAlignment;
use arkit::prelude::ArkUINodeAttributeType;
use arkit::Node;

use crate::theme::{color, radius, spacing, typography};

const FONT_WEIGHT_W500: i32 = 4;
const FONT_WEIGHT_W600: i32 = 5;
const SHADOW_OUTER_DEFAULT_SM: i32 = 1;

fn edge(top: f32, right: f32, bottom: f32, left: f32) -> Vec<f32> {
    vec![top, right, bottom, left]
}

fn edge_all(value: f32) -> Vec<f32> {
    vec![value, value, value, value]
}

pub fn padding_xy<Message, AppTheme>(
    element: Node<Message, AppTheme>,
    x: f32,
    y: f32,
) -> Node<Message, AppTheme> {
    element.style(ArkUINodeAttributeType::Padding, edge(y, x, y, x))
}

pub fn margin_top<Message, AppTheme>(
    element: Node<Message, AppTheme>,
    value: f32,
) -> Node<Message, AppTheme> {
    element.style(ArkUINodeAttributeType::Margin, edge(value, 0.0, 0.0, 0.0))
}

pub fn rounded<Message, AppTheme>(
    element: Node<Message, AppTheme>,
    value: f32,
) -> Node<Message, AppTheme> {
    element.style(ArkUINodeAttributeType::BorderRadius, edge_all(value))
}

pub fn border<Message, AppTheme>(element: Node<Message, AppTheme>) -> Node<Message, AppTheme> {
    element
        .style(ArkUINodeAttributeType::BorderWidth, edge_all(1.0))
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
}

pub fn border_color<Message, AppTheme>(
    element: Node<Message, AppTheme>,
    color_value: u32,
) -> Node<Message, AppTheme> {
    element.style(ArkUINodeAttributeType::BorderColor, vec![color_value])
}

pub fn shadow_sm<Message, AppTheme>(element: Node<Message, AppTheme>) -> Node<Message, AppTheme> {
    element.style(
        ArkUINodeAttributeType::Shadow,
        vec![SHADOW_OUTER_DEFAULT_SM],
    )
}

pub fn font_weight_medium<Message, AppTheme>(
    element: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    element.style(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_W500)
}

pub fn font_weight_semibold<Message, AppTheme>(
    element: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    element.style(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_W600)
}

pub fn card_surface<Message, AppTheme>(
    element: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    shadow_sm(rounded(
        border(
            element
                .background_color(color::CARD)
                .style(
                    ArkUINodeAttributeType::ForegroundColor,
                    color::CARD_FOREGROUND,
                )
                .style(
                    ArkUINodeAttributeType::Padding,
                    edge(spacing::XXL, 0.0, spacing::XXL, 0.0),
                ),
        ),
        radius::XL,
    ))
}

pub fn input_surface<Message, AppTheme>(
    element: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    shadow_sm(rounded(
        border(
            padding_xy(
                element.background_color(color::BACKGROUND),
                spacing::MD,
                spacing::XXS,
            )
            .style(ArkUINodeAttributeType::ForegroundColor, color::FOREGROUND),
        ),
        radius::MD,
    ))
}

pub fn panel_surface<Message, AppTheme>(
    element: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    shadow_sm(rounded(
        border(element.background_color(color::POPOVER).style(
            ArkUINodeAttributeType::ForegroundColor,
            color::POPOVER_FOREGROUND,
        )),
        radius::MD,
    ))
}

pub fn title_text<Message: 'static>(content: impl Into<String>) -> arkit::TextElement<Message> {
    font_weight_semibold(
        arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::LG)
            .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
            .style(
                ArkUINodeAttributeType::TextAlign,
                i32::from(TextAlignment::Start),
            ),
    )
}

pub fn body_text<Message: 'static>(content: impl Into<String>) -> arkit::TextElement<Message> {
    font_weight_medium(
        arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::SM)
            .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
            .style(
                ArkUINodeAttributeType::TextAlign,
                i32::from(TextAlignment::Start),
            ),
    )
}

pub fn body_text_regular<Message: 'static>(
    content: impl Into<String>,
) -> arkit::TextElement<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::MD)
        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
        .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
        .style(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Start),
        )
}

pub fn muted_text<Message: 'static>(content: impl Into<String>) -> arkit::TextElement<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
        .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
        .style(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Start),
        )
}
