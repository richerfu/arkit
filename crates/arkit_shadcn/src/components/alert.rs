use super::*;
use arkit_icon as lucide;

const ALERT_BORDER_WIDTH: f32 = 1.0;
const ALERT_ICON_SIZE: f32 = 16.0;
const ALERT_ICON_LEFT: f32 = 14.0;
const ALERT_ICON_TOP: f32 = 12.0;
const ALERT_PADDING_TOP: f32 = 14.0;
const ALERT_PADDING_RIGHT: f32 = 16.0;
const ALERT_PADDING_BOTTOM: f32 = 8.0;
const ALERT_PADDING_LEFT: f32 = 16.0;
const ALERT_CONTENT_LEFT: f32 = 24.0;
const ALERT_CONTENT_OFFSET: f32 = 2.0;
const ALERT_TITLE_BOTTOM: f32 = 4.0;
const ALERT_DESCRIPTION_BOTTOM: f32 = 6.0;
const ALERT_LIST_BOTTOM: f32 = 8.0;
const ALERT_TRACKING_TIGHT: f32 = -0.2;
const ALERT_TITLE_LINE_HEIGHT: f32 = 14.0;
// Tailwind `leading-relaxed` for `text-sm`: 14 * 1.625 = 22.75
const ALERT_DESCRIPTION_LINE_HEIGHT: f32 = 22.75;
const ALERT_LIST_LINE_HEIGHT: f32 = 20.0;
const FONT_WEIGHT_MEDIUM: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    Default,
    Destructive,
}

#[derive(Debug, Clone, Copy)]
struct AlertTone {
    title_color: u32,
    description_color: u32,
    icon_color: u32,
}

pub fn alert_root(
    icon_name: impl Into<String>,
    variant: AlertVariant,
    children: Vec<Element>,
) -> Element {
    let tone = alert_tone(variant);

    arkit::stack_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::LG, radius::LG, radius::LG, radius::LG],
        )
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![
                ALERT_BORDER_WIDTH,
                ALERT_BORDER_WIDTH,
                ALERT_BORDER_WIDTH,
                ALERT_BORDER_WIDTH,
            ],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .background_color(color::CARD)
        .children(vec![
            arkit::row_component()
                .width(ALERT_ICON_SIZE)
                .height(ALERT_ICON_SIZE)
                .style(
                    ArkUINodeAttributeType::Position,
                    vec![ALERT_ICON_LEFT, ALERT_ICON_TOP],
                )
                .children(vec![lucide::icon(icon_name)
                    .size(ALERT_ICON_SIZE)
                    .color(tone.icon_color)
                    .render()])
                .into(),
            arkit::column_component()
                .percent_width(1.0)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![
                        ALERT_PADDING_TOP,
                        ALERT_PADDING_RIGHT,
                        ALERT_PADDING_BOTTOM,
                        ALERT_PADDING_LEFT,
                    ],
                )
                .children(children)
                .into(),
        ])
        .into()
}

pub fn alert_title(content: impl Into<String>, variant: AlertVariant) -> TextElement {
    let tone = alert_tone(variant);

    arkit::text(content)
        .font_size(typography::SM)
        .style(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_MEDIUM)
        .style(ArkUINodeAttributeType::FontColor, tone.title_color)
        .style(
            ArkUINodeAttributeType::TextLineHeight,
            ALERT_TITLE_LINE_HEIGHT,
        )
        .style(
            ArkUINodeAttributeType::TextLetterSpacing,
            ALERT_TRACKING_TIGHT,
        )
        .style(
            ArkUINodeAttributeType::Margin,
            vec![0.0, 0.0, ALERT_TITLE_BOTTOM, ALERT_CONTENT_OFFSET],
        )
        .style(
            ArkUINodeAttributeType::Padding,
            vec![0.0, 0.0, 0.0, ALERT_CONTENT_LEFT],
        )
}

pub fn alert_description(content: impl Into<String>, variant: AlertVariant) -> TextElement {
    let tone = alert_tone(variant);

    arkit::text(content)
        .font_size(typography::SM)
        .style(ArkUINodeAttributeType::FontColor, tone.description_color)
        .style(
            ArkUINodeAttributeType::TextLineHeight,
            ALERT_DESCRIPTION_LINE_HEIGHT,
        )
        .style(
            ArkUINodeAttributeType::Margin,
            vec![0.0, 0.0, 0.0, ALERT_CONTENT_OFFSET],
        )
        .style(
            ArkUINodeAttributeType::Padding,
            vec![0.0, 0.0, ALERT_DESCRIPTION_BOTTOM, ALERT_CONTENT_LEFT],
        )
}

pub fn alert_list(items: Vec<impl Into<String>>, variant: AlertVariant) -> Element {
    let tone = alert_tone(variant);
    let rows = items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let text = format!("\u{2022} {}", item.into());
            let row = arkit::text(text)
                .font_size(typography::SM)
                .style(ArkUINodeAttributeType::FontColor, tone.title_color)
                .style(
                    ArkUINodeAttributeType::TextLineHeight,
                    ALERT_LIST_LINE_HEIGHT,
                );

            if index == 0 {
                row.into()
            } else {
                margin_top(row, 2.0).into()
            }
        })
        .collect::<Vec<_>>();

    arkit::column_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::Margin,
            vec![0.0, 0.0, 0.0, ALERT_CONTENT_OFFSET],
        )
        .style(
            ArkUINodeAttributeType::Padding,
            vec![0.0, 0.0, ALERT_LIST_BOTTOM, ALERT_CONTENT_LEFT],
        )
        .children(rows)
        .into()
}

fn alert_tone(variant: AlertVariant) -> AlertTone {
    match variant {
        AlertVariant::Default => AlertTone {
            title_color: color::FOREGROUND,
            description_color: color::MUTED_FOREGROUND,
            icon_color: color::FOREGROUND,
        },
        AlertVariant::Destructive => AlertTone {
            title_color: color::DESTRUCTIVE,
            description_color: 0xE6EF4444,
            icon_color: color::DESTRUCTIVE,
        },
    }
}
