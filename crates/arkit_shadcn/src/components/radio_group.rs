use super::*;
use std::rc::Rc;

const RADIO_SIZE: f32 = 16.0;
const RADIO_DOT_SIZE: f32 = 8.0;
const RADIO_BORDER_WIDTH: f32 = 1.0;

fn radio_indicator<Message: 'static>(checked: bool) -> Element<Message> {
    let mut indicator = shadow_sm(
        arkit::row_component::<Message, arkit::Theme>()
            .width(RADIO_SIZE)
            .height(RADIO_SIZE)
            .align_items_center()
            .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::FULL, radius::FULL, radius::FULL, radius::FULL],
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![
                    RADIO_BORDER_WIDTH,
                    RADIO_BORDER_WIDTH,
                    RADIO_BORDER_WIDTH,
                    RADIO_BORDER_WIDTH,
                ],
            )
            .style(
                ArkUINodeAttributeType::BorderColor,
                vec![if checked {
                    color::PRIMARY
                } else {
                    color::INPUT
                }],
            )
            .background_color(color::BACKGROUND),
    );

    if checked {
        indicator = indicator.children(vec![arkit::row_component::<Message, arkit::Theme>()
            .width(RADIO_DOT_SIZE)
            .height(RADIO_DOT_SIZE)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::FULL, radius::FULL, radius::FULL, radius::FULL],
            )
            .background_color(color::PRIMARY)
            .into()]);
    }

    indicator.into()
}

fn radio_group_impl<Message: 'static>(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) + 'static,
) -> Element<Message> {
    let selected = selected.into();
    let on_select = Rc::new(on_select);
    let children = options
        .into_iter()
        .enumerate()
        .map(|(index, option)| {
            let selected_value = selected.clone();
            let click_value = option.clone();
            let on_select = on_select.clone();
            let row = arkit::row_component::<Message, arkit::Theme>()
                .percent_width(1.0)
                .align_items_center()
                .on_click(move || on_select(click_value.clone()))
                .children(vec![
                    radio_indicator::<Message>(selected_value == option),
                    arkit::row_component::<Message, arkit::Theme>()
                        .style(
                            ArkUINodeAttributeType::Margin,
                            vec![0.0, 0.0, 0.0, spacing::MD],
                        )
                        .children(vec![label::<Message>(option).into()])
                        .into(),
                ]);

            if index == 0 {
                row.into()
            } else {
                margin_top(row, spacing::MD).into()
            }
        })
        .collect::<Vec<Element<Message>>>();

    arkit::column_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .children(children)
        .into()
}

pub fn radio_group_message<Message>(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    radio_group_impl(options, selected, move |value| {
        dispatch_message(on_select(value))
    })
}
