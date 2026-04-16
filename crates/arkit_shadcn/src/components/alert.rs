use super::*;
use arkit::TextAlignment;
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

pub fn alert_root<Message: 'static>(
    icon_name: impl Into<String>,
    variant: AlertVariant,
    children: Vec<Element<Message>>,
) -> Element<Message> {
    let tone = alert_tone(variant);

    arkit::stack_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .border_radius([radii().lg, radii().lg, radii().lg, radii().lg])
        .border_width([
            ALERT_BORDER_WIDTH,
            ALERT_BORDER_WIDTH,
            ALERT_BORDER_WIDTH,
            ALERT_BORDER_WIDTH,
        ])
        .border_color(colors().border)
        .background_color(colors().card)
        .children(vec![
            arkit::row_component::<Message, arkit::Theme>()
                .width(ALERT_ICON_SIZE)
                .height(ALERT_ICON_SIZE)
                .position(ALERT_ICON_LEFT, ALERT_ICON_TOP)
                .children(vec![lucide::icon(icon_name)
                    .size(ALERT_ICON_SIZE)
                    .color(tone.icon_color)
                    .render::<Message, arkit::Theme>()])
                .into(),
            arkit::column_component::<Message, arkit::Theme>()
                .percent_width(1.0)
                .align_items_start()
                .padding([
                    ALERT_PADDING_TOP,
                    ALERT_PADDING_RIGHT,
                    ALERT_PADDING_BOTTOM,
                    ALERT_PADDING_LEFT,
                ])
                .children(children)
                .into(),
        ])
        .into()
}

pub fn alert_title<Message: 'static>(
    content: impl Into<String>,
    variant: AlertVariant,
) -> TextElement<Message> {
    let tone = alert_tone(variant);

    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_weight(FontWeight::W500)
        .font_color(tone.title_color)
        .line_height(ALERT_TITLE_LINE_HEIGHT)
        .text_letter_spacing(ALERT_TRACKING_TIGHT)
        .text_align(TextAlignment::Start)
        .margin([0.0, 0.0, ALERT_TITLE_BOTTOM, ALERT_CONTENT_OFFSET])
        .padding([0.0, 0.0, 0.0, ALERT_CONTENT_LEFT])
}

pub fn alert_description<Message: 'static>(
    content: impl Into<String>,
    variant: AlertVariant,
) -> TextElement<Message> {
    let tone = alert_tone(variant);

    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_color(tone.description_color)
        .line_height(ALERT_DESCRIPTION_LINE_HEIGHT)
        .text_align(TextAlignment::Start)
        .margin([0.0, 0.0, 0.0, ALERT_CONTENT_OFFSET])
        .padding([0.0, 0.0, ALERT_DESCRIPTION_BOTTOM, ALERT_CONTENT_LEFT])
}

pub fn alert_list<Message: 'static>(
    items: Vec<impl Into<String>>,
    variant: AlertVariant,
) -> Element<Message> {
    let tone = alert_tone(variant);
    let rows = items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let text = format!("\u{2022} {}", item.into());
            let row = arkit::text(text)
                .font_size(typography::SM)
                .font_color(tone.title_color)
                .line_height(ALERT_LIST_LINE_HEIGHT);

            if index == 0 {
                row.into()
            } else {
                margin_top(row, 2.0).into()
            }
        })
        .collect::<Vec<Element<Message>>>();

    arkit::column_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .align_items_start()
        .margin([0.0, 0.0, 0.0, ALERT_CONTENT_OFFSET])
        .padding([0.0, 0.0, ALERT_LIST_BOTTOM, ALERT_CONTENT_LEFT])
        .children(rows)
        .into()
}

fn alert_tone(variant: AlertVariant) -> AlertTone {
    match variant {
        AlertVariant::Default => AlertTone {
            title_color: colors().foreground,
            description_color: colors().muted_foreground,
            icon_color: colors().foreground,
        },
        AlertVariant::Destructive => AlertTone {
            title_color: colors().destructive,
            description_color: with_alpha(colors().destructive, 0xE6),
            icon_color: colors().destructive,
        },
    }
}
