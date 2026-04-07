use super::floating_layer::{floating_panel_with_builder, FloatingAlign, FloatingSide};
use super::*;
use arkit::component;
use arkit_icon as lucide;
use std::rc::Rc;

#[derive(Clone)]
struct SelectOpenMarker;

#[component]
pub fn select(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) + 'static,
) -> Element {
    let selected = selected.into();
    let open = local_bool_state(SelectOpenMarker, false);
    let toggle_open = open.clone();
    let on_select = Rc::new(on_select);

    let trigger = shadow_sm(crate::styles::rounded(
        crate::styles::border(
            arkit::row_component()
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
                    arkit::dynamic({
                        let selected = selected.clone();
                        move || {
                            let current = selected.clone();
                            let has_value = !current.is_empty();
                            let label = if has_value {
                                current
                            } else {
                                String::from("Select a fruit")
                            };
                            arkit::text(label)
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
                        }
                    }),
                    lucide::icon("chevron-down")
                        .size(16.0)
                        .color(color::MUTED_FOREGROUND)
                        .render(),
                ]),
        ),
        radius::MD,
    ))
    .on_click(move || {
        toggle_open.set(!toggle_open.get());
        request_runtime_rerender();
    })
    .into();

    let panel_builder: Rc<dyn Fn(Option<f32>) -> Element> = Rc::new({
        let options = options.clone();
        let selected = selected.clone();
        let close = open.clone();
        let on_select = on_select.clone();
        move |trigger_width| {
            let count = options.len();
            let items = options
                .iter()
                .cloned()
                .map(|option| {
                    let close_dropdown = close.clone();
                    let sel = selected.clone();
                    let opt = option.clone();
                    let on_select = on_select.clone();
                    let active = sel == opt;
                    let opt_click = opt.clone();

                    arkit::row_component()
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
                            close_dropdown.set(false);
                            request_runtime_rerender();
                        })
                        .children(vec![
                            arkit::text(opt.clone())
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
                                    .render()
                            } else {
                                arkit::row_component().width(16.0).height(16.0).into()
                            },
                        ])
                        .into()
                })
                .collect::<Vec<_>>();

            let list = if count > 8 {
                arkit::scroll_component()
                    .height(208.0)
                    .children(vec![arkit::column_component()
                        .percent_width(1.0)
                        .children(items)
                        .into()])
                    .into()
            } else {
                arkit::column_component()
                    .percent_width(1.0)
                    .children(items)
                    .into()
            };

            let mut panel = arkit::column_component()
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS],
                )
                .children(vec![
                    arkit::row_component()
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![8.0, spacing::SM, 8.0, spacing::SM],
                        )
                        .children(vec![arkit::text("Fruits")
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
        let open = open.clone();
        Rc::new(move || {
            open.set(false);
            request_runtime_rerender();
        })
    };
    floating_panel_with_builder(
        trigger,
        open.get(),
        FloatingSide::Bottom,
        FloatingAlign::Start,
        panel_builder,
        Some(dismiss),
    )
}
