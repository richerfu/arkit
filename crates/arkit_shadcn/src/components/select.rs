use super::floating_layer::{floating_panel_with_builder, FloatingAlign, FloatingSide};
use super::*;
use arkit_icon as lucide;
use std::rc::Rc;

pub fn select<Message: 'static>(
    options: Vec<String>,
    selected: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    on_select: impl Fn(String) + 'static,
) -> Element<Message> {
    let selected = selected.into();
    let on_select = Rc::new(on_select);
    let on_open_change = Rc::new(on_open_change);

    let trigger = shadow_sm(crate::styles::rounded(
        crate::styles::border(
            arkit::row_component::<Message, arkit::Theme>()
                .height(40.0)
                .percent_width(1.0)
                .background_color(color::BACKGROUND)
                .style(ArkUINodeAttributeType::ForegroundColor, color::FOREGROUND)
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
                    {
                        let has_value = !selected.is_empty();
                        let label = if has_value {
                            selected.clone()
                        } else {
                            String::from("Select a fruit")
                        };
                        arkit::text::<Message, arkit::Theme>(label)
                            .font_size(typography::SM)
                            .style(
                                ArkUINodeAttributeType::FontColor,
                                if has_value {
                                    color::FOREGROUND
                                } else {
                                    color::MUTED_FOREGROUND
                                },
                            )
                            .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
                            .into()
                    },
                    lucide::icon("chevron-down")
                        .size(16.0)
                        .color(color::MUTED_FOREGROUND)
                        .render(),
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
            let count = options.len();
            let items = options
                .iter()
                .cloned()
                .map(|option| {
                    let sel = selected.clone();
                    let opt = option.clone();
                    let on_select = on_select.clone();
                    let on_open_change = on_open_change.clone();
                    let active = sel == opt;
                    let opt_click = opt.clone();

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
                            on_select(opt_click.clone());
                            on_open_change(false);
                        })
                        .children(vec![
                            arkit::text::<Message, arkit::Theme>(opt.clone())
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
                                    .color(color::MUTED_FOREGROUND)
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

            let list = if count > 8 {
                arkit::scroll_component::<Message, arkit::Theme>()
                    .height(208.0)
                    .children(vec![arkit::column_component::<Message, arkit::Theme>()
                        .percent_width(1.0)
                        .children(items)
                        .into()])
                    .into()
            } else {
                arkit::column_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .children(items)
                    .into()
            };

            let mut panel = arkit::column_component::<Message, arkit::Theme>()
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS],
                )
                .children(vec![
                    arkit::row_component::<Message, arkit::Theme>()
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![8.0, spacing::SM, 8.0, spacing::SM],
                        )
                        .children(vec![arkit::text::<Message, arkit::Theme>("Fruits")
                            .font_size(typography::XS)
                            .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
                            .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
                            .into()])
                        .into(),
                    list,
                ]);

            if let Some(width) = trigger_width {
                panel = panel.width(width);
            } else {
                panel = panel.percent_width(1.0);
            }

            panel_surface(panel).into()
        }
    });

    let dismiss = {
        let on_open_change = on_open_change.clone();
        Rc::new(move || on_open_change(false))
    };
    floating_panel_with_builder(
        trigger,
        open,
        FloatingSide::Bottom,
        FloatingAlign::Start,
        panel_builder,
        Some(dismiss),
    )
}

pub fn select_message<Message>(
    options: Vec<String>,
    selected: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    on_select: impl Fn(String) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    select(
        options,
        selected,
        open,
        move |value| dispatch_message(on_open_change(value)),
        move |value| dispatch_message(on_select(value)),
    )
}
