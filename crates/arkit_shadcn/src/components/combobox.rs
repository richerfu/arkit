use super::floating_layer::{floating_panel_with_builder, FloatingAlign, FloatingSide};
use super::*;
use arkit_icon as lucide;
use std::rc::Rc;

pub fn combobox<Message>(
    options: Vec<String>,
    selected: impl Into<String> + 'static,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    on_select: impl Fn(String) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    let selected = selected.into();
    let on_open_change = Rc::new(move |value| dispatch_message(on_open_change(value)));
    let on_select = Rc::new(move |value| dispatch_message(on_select(value)));

    let trigger = shadow_sm(crate::styles::rounded(
        crate::styles::border(
            arkit::row_component::<Message, arkit::Theme>()
                .height(40.0)
                .percent_width(1.0)
                .background_color(color::BACKGROUND)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![8.0, spacing::MD, 8.0, spacing::MD],
                )
                .align_items_center()
                .style(
                    ArkUINodeAttributeType::RowJustifyContent,
                    FLEX_ALIGN_SPACE_BETWEEN,
                )
                .children(vec![
                    arkit::row_component::<Message, arkit::Theme>()
                        .align_items_center()
                        .children(vec![
                            lucide::icon("search")
                                .size(16.0)
                                .color(color::MUTED_FOREGROUND)
                                .render::<Message, arkit::Theme>(),
                            arkit::row_component::<Message, arkit::Theme>()
                                .style(
                                    ArkUINodeAttributeType::Margin,
                                    vec![0.0, 0.0, 0.0, spacing::SM],
                                )
                                .children(vec![arkit::text::<Message, arkit::Theme>(
                                    if selected.is_empty() {
                                        String::from("Search an option")
                                    } else {
                                        selected.clone()
                                    },
                                )
                                .font_size(typography::SM)
                                .style(
                                    ArkUINodeAttributeType::FontColor,
                                    if selected.is_empty() {
                                        color::MUTED_FOREGROUND
                                    } else {
                                        color::FOREGROUND
                                    },
                                )
                                .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
                                .into()])
                                .into(),
                        ])
                        .into(),
                    lucide::icon("chevrons-up-down")
                        .size(16.0)
                        .color(color::MUTED_FOREGROUND)
                        .render::<Message, arkit::Theme>(),
                ]),
        ),
        radius::MD,
    ))
    .on_click({
        let on_open_change = on_open_change.clone();
        move || on_open_change(!open)
    })
    .into();

    let panel_builder: Rc<dyn Fn(Option<f32>) -> Element<Message>> = Rc::new({
        let options = options.clone();
        let selected = selected.clone();
        let on_select = on_select.clone();
        let on_open_change = on_open_change.clone();
        move |trigger_width| {
            let items = options
                .iter()
                .cloned()
                .map(|option| {
                    let active = selected == option;
                    let option_value = option.clone();
                    let on_select = on_select.clone();
                    let on_open_change = on_open_change.clone();
                    arkit::row_component::<Message, arkit::Theme>()
                        .percent_width(1.0)
                        .height(36.0)
                        .align_items_center()
                        .style(
                            ArkUINodeAttributeType::RowJustifyContent,
                            FLEX_ALIGN_SPACE_BETWEEN,
                        )
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![8.0, spacing::SM, 8.0, spacing::SM],
                        )
                        .style(
                            ArkUINodeAttributeType::BorderRadius,
                            vec![radius::SM, radius::SM, radius::SM, radius::SM],
                        )
                        .background_color(if active { color::ACCENT } else { 0x00000000 })
                        .on_click(move || {
                            on_select(option_value.clone());
                            on_open_change(false);
                        })
                        .children(vec![
                            arkit::text::<Message, arkit::Theme>(option.clone())
                                .font_size(typography::SM)
                                .style(
                                    ArkUINodeAttributeType::FontColor,
                                    if active {
                                        color::ACCENT_FOREGROUND
                                    } else {
                                        color::FOREGROUND
                                    },
                                )
                                .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
                                .into(),
                            if active {
                                lucide::icon("check")
                                    .size(16.0)
                                    .color(color::FOREGROUND)
                                    .render::<Message, arkit::Theme>()
                            } else {
                                arkit::row_component::<Message, arkit::Theme>()
                                    .width(16.0)
                                    .height(16.0)
                                    .into()
                            },
                        ])
                        .into()
                })
                .collect::<Vec<_>>();

            let mut panel = arkit::column_component::<Message, arkit::Theme>().children(vec![
                arkit::row_component::<Message, arkit::Theme>()
                    .style(
                        ArkUINodeAttributeType::Padding,
                        vec![8.0, spacing::SM, 8.0, spacing::SM],
                    )
                    .children(vec![arkit::text::<Message, arkit::Theme>("Suggestions")
                        .font_size(typography::XS)
                        .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
                        .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
                        .into()])
                    .into(),
                arkit::column_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .style(
                        ArkUINodeAttributeType::Padding,
                        vec![spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS],
                    )
                    .children(items)
                    .into(),
            ]);

            if let Some(width) = trigger_width {
                panel = panel.width(width);
            } else {
                panel = panel.percent_width(1.0);
            }

            panel_surface(panel).into()
        }
    });

    floating_panel_with_builder(
        trigger,
        open,
        FloatingSide::Bottom,
        FloatingAlign::Start,
        panel_builder,
        Some(Rc::new(move || on_open_change(false))),
    )
}
