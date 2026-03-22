use super::*;

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
                .map(|header| {
                    arkit::row_component()
                        .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
                        .height(40.0)
                        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                        .style(ArkUINodeAttributeType::Padding, vec![0.0, 8.0, 0.0, 8.0])
                        .children(vec![body_text(header)
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

pub fn scrollable_table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Element {
    scroll_area(vec![table(headers, rows)]).into()
}

pub fn table_row(cells: Vec<String>) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .children(cells.into_iter().map(body_text).map(Into::into).collect())
        .into()
}
