use super::*;
use arkit::{component, use_signal};

pub fn table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Element {
    let header_row = arkit::row_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![0.0, 0.0, 1.0, 0.0],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .children(
            headers
                .into_iter()
                .map(|h| {
                    arkit::row_component()
                        .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
                        .height(40.0)
                        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                        .style(ArkUINodeAttributeType::Padding, vec![0.0, 8.0, 0.0, 8.0])
                        .children(vec![body_text(h)
                            .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
                            .into()])
                        .into()
                })
                .collect(),
        )
        .into();

    let total_rows = rows.len();
    let body_rows = rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            arkit::row_component()
                .percent_width(1.0)
                .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                .style(
                    ArkUINodeAttributeType::BorderWidth,
                    if index + 1 == total_rows {
                        vec![0.0, 0.0, 0.0, 0.0]
                    } else {
                        vec![0.0, 0.0, 1.0, 0.0]
                    },
                )
                .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
                .children(
                    row.into_iter()
                        .map(|cell| {
                            arkit::row_component()
                                .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
                                .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                                .style(ArkUINodeAttributeType::Padding, vec![8.0, 8.0, 8.0, 8.0])
                                .children(vec![arkit::text(cell)
                                    .font_size(typography::SM)
                                    .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                                    .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
                                    .into()])
                                .into()
                        })
                        .collect::<Vec<_>>(),
                )
                .into()
        })
        .collect::<Vec<Element>>();

    rounded_table_surface(
        arkit::column_component()
            .percent_width(1.0)
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![1.0, 1.0, 1.0, 1.0],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::SM, radius::SM, radius::SM, radius::SM],
            )
            .background_color(color::CARD)
            .children(
                std::iter::once(header_row)
                    .chain(body_rows)
                    .collect::<Vec<_>>(),
            ),
    )
    .into()
}

pub fn data_table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Element {
    table(headers, rows)
}

pub fn form(fields: Vec<Element>, submit_label: impl Into<String>) -> Element {
    card(
        fields
            .into_iter()
            .chain(std::iter::once(
                margin_top(button(submit_label, ButtonVariant::Default), spacing::SM).into(),
            ))
            .collect(),
    )
}

pub fn radio_group(options: Vec<String>, selected: Signal<String>) -> Element {
    let children = options
        .into_iter()
        .map(|option| {
            let value = option.clone();
            let selected_value = selected.clone();
            arkit::row_component()
                .percent_width(1.0)
                .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                .children(vec![
                    shadow_sm(
                        arkit::radio_component()
                            .style(
                                ArkUINodeAttributeType::RadioChecked,
                                selected.get() == option,
                            )
                            .style(ArkUINodeAttributeType::BorderColor, vec![color::INPUT])
                            .style(
                                ArkUINodeAttributeType::BorderWidth,
                                vec![1.0, 1.0, 1.0, 1.0],
                            )
                            .style(ArkUINodeAttributeType::RadioStyle, vec![color::PRIMARY])
                            .width(16.0)
                            .height(16.0)
                            .on_click(move || selected_value.set(value.clone())),
                    )
                    .into(),
                    arkit::row_component()
                        .style(
                            ArkUINodeAttributeType::Margin,
                            vec![0.0, 0.0, 0.0, spacing::SM],
                        )
                        .children(vec![body_text(option).into()])
                        .into(),
                ])
                .into()
        })
        .collect::<Vec<Element>>();

    arkit::column_component()
        .percent_width(1.0)
        .children(
            children
                .into_iter()
                .enumerate()
                .map(|(idx, child)| {
                    if idx == 0 {
                        child
                    } else {
                        arkit::row_component()
                            .style(
                                ArkUINodeAttributeType::Margin,
                                vec![spacing::MD, 0.0, 0.0, 0.0],
                            )
                            .children(vec![child])
                            .into()
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into()
}

#[component]
pub fn select(options: Vec<String>, selected: Signal<String>) -> Element {
    let open = use_signal(|| false);
    let current = selected.get();
    let toggle_open = open.clone();
    let trigger = input_surface(
        arkit::row_component()
            .percent_width(1.0)
            .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
            .style(
                ArkUINodeAttributeType::RowJustifyContent,
                FLEX_ALIGN_SPACE_BETWEEN,
            )
            .children(vec![
                arkit::text(current.clone())
                    .font_size(typography::SM)
                    .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                    .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
                    .into(),
                muted_text(if open.get() { "⌃" } else { "⌄" }).into(),
            ]),
    )
    .on_click(move || toggle_open.update(|v| *v = !*v))
    .into();

    let mut children = vec![trigger];
    if open.get() {
        let close = open.clone();
        let items = options
            .into_iter()
            .map(|option| {
                let value = selected.clone();
                let close_dropdown = close.clone();
                let active = current == option;
                let option_label = option.clone();
                arkit::row_component()
                    .percent_width(1.0)
                    .height(32.0)
                    .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                    .style(
                        ArkUINodeAttributeType::Padding,
                        vec![6.0, spacing::SM, 6.0, spacing::SM],
                    )
                    .style(
                        ArkUINodeAttributeType::BorderRadius,
                        vec![radius::SM, radius::SM, radius::SM, radius::SM],
                    )
                    .background_color(if active { color::ACCENT } else { 0x00000000 })
                    .on_click(move || {
                        value.set(option.clone());
                        close_dropdown.set(false);
                    })
                    .children(vec![arkit::text(option_label)
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
                        .into()])
                    .into()
            })
            .collect::<Vec<_>>();
        children.push(
            margin_top(
                panel_surface(
                    arkit::column_component()
                        .percent_width(1.0)
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS],
                        )
                        .children(items),
                ),
                spacing::XXS,
            )
            .into(),
        );
    }

    arkit::column_component()
        .percent_width(1.0)
        .children(children)
        .into()
}

pub fn combobox(options: Vec<String>, value: Signal<String>) -> Element {
    select(options, value)
}

pub fn command(query: Signal<String>, options: Vec<String>) -> Element {
    let keyword = query.get().to_lowercase();
    let mut children = vec![arkit::row_component()
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![0.0, 0.0, 1.0, 0.0],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .children(vec![input("Search command...").bind(query.clone()).into()])
        .into()];
    children.extend(
        options
            .into_iter()
            .filter(|option| keyword.is_empty() || option.to_lowercase().contains(&keyword))
            .map(|option| {
                let value = query.clone();
                let option_label = option.clone();
                arkit::row_component()
                    .percent_width(1.0)
                    .height(32.0)
                    .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                    .style(
                        ArkUINodeAttributeType::Padding,
                        vec![6.0, spacing::SM, 6.0, spacing::SM],
                    )
                    .style(
                        ArkUINodeAttributeType::BorderRadius,
                        vec![radius::SM, radius::SM, radius::SM, radius::SM],
                    )
                    .on_click(move || value.set(option.clone()))
                    .children(vec![arkit::text(option_label)
                        .font_size(typography::SM)
                        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                        .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
                        .into()])
                    .into()
            }),
    );
    panel_surface(
        arkit::column_component()
            .percent_width(1.0)
            .style(
                ArkUINodeAttributeType::Padding,
                vec![spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS],
            )
            .children(children),
    )
    .into()
}

pub fn input_otp(value: Signal<String>, digits: usize) -> Element {
    let chars = value.get().chars().collect::<Vec<_>>();
    arkit::row_component()
        .percent_width(1.0)
        .children(
            (0..digits)
                .map(|idx| {
                    let otp = value.clone();
                    input_surface(
                        arkit::text_input_component()
                            .value(chars.get(idx).map(char::to_string).unwrap_or_default())
                            .width(36.0)
                            .height(36.0)
                            .font_size(typography::SM)
                            .on_change(move |next| {
                                let mut current = otp.get().chars().collect::<Vec<_>>();
                                if current.len() < digits {
                                    current.resize(digits, '\0');
                                }
                                current[idx] = next.chars().next().unwrap_or('\0');
                                let next_value = current
                                    .into_iter()
                                    .filter(|ch| *ch != '\0')
                                    .collect::<String>();
                                otp.set(next_value);
                            }),
                    )
                    .into()
                })
                .collect(),
        )
        .into()
}

pub fn calendar() -> CalendarPickerElement {
    panel_surface(arkit::calendar_picker_component().height(320.0))
}

pub fn date_picker() -> DatePickerElement {
    input_surface(arkit::date_picker_component())
}

pub fn carousel(slides: Vec<Element>) -> SwiperElement {
    panel_surface(arkit::swiper_component().children(slides))
}

pub fn chart(values: Vec<f32>) -> Element {
    let palette = [
        color::CHART_1,
        color::CHART_2,
        color::CHART_3,
        color::CHART_4,
        color::CHART_5,
    ];

    card(
        values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                let percent = value.clamp(0.0, 100.0);
                let tone = palette[index % palette.len()];

                arkit::column_component()
                    .percent_width(1.0)
                    .children(vec![
                        arkit::row_component()
                            .percent_width(1.0)
                            .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                            .style(
                                ArkUINodeAttributeType::RowJustifyContent,
                                FLEX_ALIGN_SPACE_BETWEEN,
                            )
                            .children(vec![
                                muted_text(format!("Series {}", index + 1)).into(),
                                body_text_regular(format!("{percent:.0}%")).into(),
                            ])
                            .into(),
                        arkit::row_component()
                            .style(
                                ArkUINodeAttributeType::Margin,
                                vec![spacing::XXS, 0.0, 0.0, 0.0],
                            )
                            .children(vec![rounded_progress(
                                arkit::progress_component()
                                    .style(ArkUINodeAttributeType::ProgressValue, percent)
                                    .style(ArkUINodeAttributeType::ProgressTotal, 100.0)
                                    .style(ArkUINodeAttributeType::ProgressColor, tone)
                                    .height(8.0),
                            )
                            .into()])
                            .into(),
                    ])
                    .into()
            })
            .collect(),
    )
}

pub fn avatar(src: Option<String>, fallback_text: impl Into<String>) -> Element {
    let fallback = fallback_text.into();
    if let Some(src) = src {
        arkit::image_component()
            .attr(ArkUINodeAttributeType::ImageSrc, src)
            .width(32.0)
            .height(32.0)
            .style(ArkUINodeAttributeType::BorderRadius, vec![radius::FULL])
            .into()
    } else {
        arkit::row_component()
            .width(32.0)
            .height(32.0)
            .background_color(color::MUTED)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::FULL, radius::FULL, radius::FULL, radius::FULL],
            )
            .children(vec![muted_text(fallback).into()])
            .into()
    }
}

pub fn tooltip(trigger_label: impl Into<String>, content: impl Into<String>) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .children(vec![
            button(trigger_label, ButtonVariant::Ghost).into(),
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![0.0, 0.0, 0.0, spacing::SM],
                )
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![spacing::SM, spacing::MD, spacing::SM, spacing::MD],
                )
                .style(
                    ArkUINodeAttributeType::BorderRadius,
                    vec![radius::MD, radius::MD, radius::MD, radius::MD],
                )
                .background_color(color::PRIMARY)
                .children(vec![arkit::text(content)
                    .font_size(typography::XS)
                    .style(ArkUINodeAttributeType::FontColor, color::PRIMARY_FOREGROUND)
                    .into()])
                .into(),
        ])
        .into()
}

pub fn aspect_ratio(ratio: f32, child: Element) -> Element {
    panel_surface(
        arkit::stack_component()
            .style(ArkUINodeAttributeType::AspectRatio, ratio)
            .children(vec![child]),
    )
    .into()
}

pub fn scrollable_table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Element {
    scroll_area(vec![table(headers, rows)]).into()
}

pub fn text_variant(content: impl Into<String>, muted: bool) -> Element {
    if muted {
        muted_text(content).into()
    } else {
        body_text(content).into()
    }
}

pub fn table_row(cells: Vec<String>) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .children(cells.into_iter().map(body_text).map(Into::into).collect())
        .into()
}

pub fn popover_card(title: impl Into<String>, body: impl Into<String>) -> Element {
    card(vec![title_text(title).into(), muted_text(body).into()])
}

pub fn calendar_card() -> Element {
    card(vec![calendar().into()])
}

pub fn chart_card(title: impl Into<String>, values: Vec<f32>) -> Element {
    card(vec![title_text(title).into(), chart(values)])
}

pub fn form_item(label_text: impl Into<String>, field: Element) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .children(vec![
            label(label_text).into(),
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![spacing::XXS, 0.0, 0.0, 0.0],
                )
                .children(vec![field])
                .into(),
        ])
        .into()
}
