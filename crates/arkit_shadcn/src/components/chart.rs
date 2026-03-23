use super::*;

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
                            .align_items_center()
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

pub fn chart_card(title: impl Into<String>, values: Vec<f32>) -> Element {
    card(vec![title_text(title).into(), chart(values)])
}
