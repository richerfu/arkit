use arkit::{FontWeight, Node, ShadowStyle, TextAlignment};

use crate::theme::{color, radius, spacing, typography};

pub fn padding_xy<Message, AppTheme>(
    element: Node<Message, AppTheme>,
    x: f32,
    y: f32,
) -> Node<Message, AppTheme> {
    element.padding([x, y])
}

pub fn margin_top<Message, AppTheme>(
    element: Node<Message, AppTheme>,
    value: f32,
) -> Node<Message, AppTheme> {
    element.margin_top(value)
}

pub fn rounded<Message, AppTheme>(
    element: Node<Message, AppTheme>,
    value: f32,
) -> Node<Message, AppTheme> {
    element.border_radius(value)
}

pub fn border<Message, AppTheme>(element: Node<Message, AppTheme>) -> Node<Message, AppTheme> {
    element.border_width(1.0).border_color(color::BORDER)
}

pub fn border_color<Message, AppTheme>(
    element: Node<Message, AppTheme>,
    color_value: u32,
) -> Node<Message, AppTheme> {
    element.border_color(color_value)
}

pub fn shadow_sm<Message, AppTheme>(element: Node<Message, AppTheme>) -> Node<Message, AppTheme> {
    element.shadow(ShadowStyle::OuterDefaultSm)
}

pub fn font_weight_medium<Message, AppTheme>(
    element: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    element.font_weight(FontWeight::W500)
}

pub fn font_weight_semibold<Message, AppTheme>(
    element: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    element.font_weight(FontWeight::W600)
}

pub fn card_surface<Message, AppTheme>(
    element: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    shadow_sm(rounded(
        border(
            element
                .background_color(color::CARD)
                .foreground_color(color::CARD_FOREGROUND)
                .padding([0.0, spacing::XXL]),
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
            .foreground_color(color::FOREGROUND),
        ),
        radius::MD,
    ))
}

pub fn panel_surface<Message, AppTheme>(
    element: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    shadow_sm(rounded(
        border(
            element
                .background_color(color::POPOVER)
                .foreground_color(color::POPOVER_FOREGROUND),
        ),
        radius::MD,
    ))
}

pub fn title_text<Message: 'static>(content: impl Into<String>) -> arkit::TextElement<Message> {
    font_weight_semibold(
        arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::LG)
            .font_color(color::FOREGROUND)
            .line_height(20.0)
            .text_align(TextAlignment::Start),
    )
}

pub fn body_text<Message: 'static>(content: impl Into<String>) -> arkit::TextElement<Message> {
    font_weight_medium(
        arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::SM)
            .font_color(color::FOREGROUND)
            .line_height(20.0)
            .text_align(TextAlignment::Start),
    )
}

pub fn body_text_regular<Message: 'static>(
    content: impl Into<String>,
) -> arkit::TextElement<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::MD)
        .font_color(color::FOREGROUND)
        .line_height(20.0)
        .text_align(TextAlignment::Start)
}

pub fn muted_text<Message: 'static>(content: impl Into<String>) -> arkit::TextElement<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_color(color::MUTED_FOREGROUND)
        .line_height(20.0)
        .text_align(TextAlignment::Start)
}
