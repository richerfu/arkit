use super::*;

pub fn table<Message: 'static>(headers: Vec<String>, rows: Vec<Vec<String>>) -> Element<Message> {
    let header_row = arkit::row_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .border_width([0.0, 0.0, 1.0, 0.0])
        .border_color(color::BORDER)
        .children(
            headers
                .into_iter()
                .map(|header| {
                    arkit::row_component::<Message, arkit::Theme>()
                        .layout_weight(1.0_f32)
                        .height(40.0)
                        .align_items_center()
                        .padding([0.0, 8.0, 0.0, 8.0])
                        .children(vec![body_text(header)
                            .font_color(color::MUTED_FOREGROUND)
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
            arkit::row_component::<Message, arkit::Theme>()
                .percent_width(1.0)
                .align_items_center()
                .border_width(if index + 1 == total_rows {
                    [0.0, 0.0, 0.0, 0.0]
                } else {
                    [0.0, 0.0, 1.0, 0.0]
                })
                .border_color(color::BORDER)
                .children(
                    row.into_iter()
                        .map(|cell| {
                            arkit::row_component::<Message, arkit::Theme>()
                                .layout_weight(1.0_f32)
                                .align_items_center()
                                .padding([8.0, 8.0, 8.0, 8.0])
                                .children(vec![arkit::text::<Message, arkit::Theme>(cell)
                                    .font_size(typography::SM)
                                    .font_color(color::FOREGROUND)
                                    .line_height(20.0)
                                    .into()])
                                .into()
                        })
                        .collect::<Vec<_>>(),
                )
                .into()
        })
        .collect::<Vec<Element<Message>>>();

    rounded_table_surface(
        arkit::column_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .border_width([1.0, 1.0, 1.0, 1.0])
            .border_color(color::BORDER)
            .border_radius([radius::SM, radius::SM, radius::SM, radius::SM])
            .background_color(color::CARD)
            .children(
                std::iter::once(header_row)
                    .chain(body_rows)
                    .collect::<Vec<_>>(),
            ),
    )
    .into()
}

pub fn data_table<Message: 'static>(
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
) -> Element<Message> {
    table(headers, rows)
}

pub fn scrollable_table<Message: 'static>(
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
) -> Element<Message> {
    scroll_area::<Message>(vec![table(headers, rows)]).into()
}

pub fn table_row<Message: 'static>(cells: Vec<String>) -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .children(
            cells
                .into_iter()
                .map(body_text::<Message>)
                .map(Into::into)
                .collect(),
        )
        .into()
}
