use arkit::{Component, FontWeight, ShadowStyle, TextAlignment};

use crate::theme::{colors, radii, spacing, typography};

pub fn padding_xy<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
    x: f32,
    y: f32,
) -> Component<Message, AppTheme, Kind> {
    element.padding([x, y])
}

pub fn margin_top<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
    value: f32,
) -> Component<Message, AppTheme, Kind> {
    element.margin_top(value)
}

pub fn rounded<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
    value: f32,
) -> Component<Message, AppTheme, Kind> {
    element.border_radius(value)
}

pub fn border<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
) -> Component<Message, AppTheme, Kind> {
    element.border_width(1.0).border_color(colors().border)
}

pub fn border_color<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
    color_value: u32,
) -> Component<Message, AppTheme, Kind> {
    element.border_color(color_value)
}

pub fn shadow_sm<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
) -> Component<Message, AppTheme, Kind> {
    element.shadow(ShadowStyle::OuterDefaultSm)
}

fn input_shadow_sm<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
) -> Component<Message, AppTheme, Kind> {
    element.custom_shadow(2.0, 0.0, 1.0, 0x0D000000, false)
}

pub fn font_weight_medium<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
) -> Component<Message, AppTheme, Kind> {
    element.font_weight(FontWeight::W500)
}

pub fn font_weight_semibold<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
) -> Component<Message, AppTheme, Kind> {
    element.font_weight(FontWeight::W600)
}

pub fn card_surface<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
) -> Component<Message, AppTheme, Kind> {
    shadow_sm(rounded(
        border(
            element
                .background_color(colors().card)
                .foreground_color(colors().card_foreground)
                .padding([0.0, spacing::XXL]),
        ),
        radii().xl,
    ))
}

pub fn input_surface<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
) -> Component<Message, AppTheme, Kind> {
    input_shadow_sm(rounded(
        border(
            padding_xy(
                element.background_color(colors().background),
                spacing::MD,
                spacing::XXS,
            )
            .foreground_color(colors().foreground),
        ),
        radii().md,
    ))
}

pub fn panel_surface<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
) -> Component<Message, AppTheme, Kind> {
    shadow_sm(rounded(
        border(
            element
                .background_color(colors().popover)
                .foreground_color(colors().popover_foreground),
        ),
        radii().md,
    ))
}

pub(crate) fn title_text<Message: 'static>(
    content: impl Into<String>,
) -> arkit::TextElement<Message> {
    font_weight_semibold(
        arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::LG)
            .font_color(colors().foreground)
            .line_height(20.0)
            .text_align(TextAlignment::Start),
    )
}

pub(crate) fn body_text<Message: 'static>(
    content: impl Into<String>,
) -> arkit::TextElement<Message> {
    font_weight_medium(
        arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::SM)
            .font_color(colors().foreground)
            .line_height(20.0)
            .text_align(TextAlignment::Start),
    )
}

pub(crate) fn body_text_regular<Message: 'static>(
    content: impl Into<String>,
) -> arkit::TextElement<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::MD)
        .font_color(colors().foreground)
        .line_height(20.0)
        .text_align(TextAlignment::Start)
}

pub(crate) fn muted_text<Message: 'static>(
    content: impl Into<String>,
) -> arkit::TextElement<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_color(colors().muted_foreground)
        .line_height(20.0)
        .text_align(TextAlignment::Start)
}
