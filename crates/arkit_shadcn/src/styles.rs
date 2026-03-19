use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use arkit::prelude::ArkUINodeAttributeType;
use arkit::ComponentElement;

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

pub fn padding_xy<T>(element: ComponentElement<T>, x: f32, y: f32) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    element.style(ArkUINodeAttributeType::Padding, edge(y, x, y, x))
}

pub fn margin_top<T>(element: ComponentElement<T>, value: f32) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    element.style(ArkUINodeAttributeType::Margin, edge(value, 0.0, 0.0, 0.0))
}

pub fn padded<T>(element: ComponentElement<T>, value: f32) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    element.style(ArkUINodeAttributeType::Padding, edge_all(value))
}

pub fn rounded<T>(element: ComponentElement<T>, value: f32) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    element.style(ArkUINodeAttributeType::BorderRadius, edge_all(value))
}

pub fn border<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    element
        .style(ArkUINodeAttributeType::BorderWidth, edge_all(1.0))
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
}

pub fn border_color<T>(element: ComponentElement<T>, color_value: u32) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    element.style(ArkUINodeAttributeType::BorderColor, vec![color_value])
}

pub fn shadow_sm<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    element.style(
        ArkUINodeAttributeType::Shadow,
        vec![SHADOW_OUTER_DEFAULT_SM],
    )
}

pub fn font_weight_medium<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    element.style(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_W500)
}

pub fn font_weight_semibold<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    element.style(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_W600)
}

pub fn card_surface<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
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

pub fn input_surface<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    shadow_sm(rounded(
        border(
            padding_xy(
                element.background_color(color::BACKGROUND),
                spacing::MD,
                spacing::XXS,
            )
            .style(ArkUINodeAttributeType::ForegroundColor, color::FOREGROUND),
        ),
        radius::SM,
    ))
}

pub fn panel_surface<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    shadow_sm(rounded(
        border(
            padded(element.background_color(color::POPOVER), spacing::MD).style(
                ArkUINodeAttributeType::ForegroundColor,
                color::POPOVER_FOREGROUND,
            ),
        ),
        radius::MD,
    ))
}

pub fn chip_surface<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    rounded(
        border(padding_xy(element, spacing::SM, 2.0))
            .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER]),
        radius::FULL,
    )
}

pub fn title_text(content: impl Into<String>) -> arkit::TextElement {
    font_weight_semibold(
        arkit::text(content)
            .font_size(typography::LG)
            .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 20.0),
    )
}

pub fn body_text(content: impl Into<String>) -> arkit::TextElement {
    font_weight_medium(
        arkit::text(content)
            .font_size(typography::SM)
            .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 20.0),
    )
}

pub fn body_text_regular(content: impl Into<String>) -> arkit::TextElement {
    arkit::text(content)
        .font_size(typography::MD)
        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
        .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
}

pub fn muted_text(content: impl Into<String>) -> arkit::TextElement {
    arkit::text(content)
        .font_size(typography::SM)
        .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
        .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
}
