use super::scroll_area::scroll_area;
use super::*;

fn table<Message: 'static>(headers: Vec<String>, rows: Vec<Vec<String>>) -> Element<Message> {
    let header_row = arkit::row_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .border_width([0.0, 0.0, 1.0, 0.0])
        .border_color(colors().border)
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
                            .font_color(colors().muted_foreground)
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
                .border_color(colors().border)
                .children(
                    row.into_iter()
                        .map(|cell| {
                            arkit::row_component::<Message, arkit::Theme>()
                                .layout_weight(1.0_f32)
                                .align_items_center()
                                .padding([8.0, 8.0, 8.0, 8.0])
                                .children(vec![arkit::text::<Message, arkit::Theme>(cell)
                                    .font_size(typography::SM)
                                    .font_color(colors().foreground)
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
            .border_color(colors().border)
            .border_radius([radii().sm, radii().sm, radii().sm, radii().sm])
            .background_color(colors().card)
            .children(
                std::iter::once(header_row)
                    .chain(body_rows)
                    .collect::<Vec<_>>(),
            ),
    )
    .into()
}

fn data_table<Message: 'static>(headers: Vec<String>, rows: Vec<Vec<String>>) -> Element<Message> {
    table(headers, rows)
}

fn scrollable_table<Message: 'static>(
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
) -> Element<Message> {
    scroll_area::<Message>(vec![table(headers, rows)]).into()
}

fn table_row<Message: 'static>(cells: Vec<String>) -> Element<Message> {
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

// Struct component API
pub struct Table<Message = ()> {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> Table<Message> {
    pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self {
            headers,
            rows,
            _marker: std::marker::PhantomData,
        }
    }
}

impl_component_widget!(Table<Message>, Message, |value: &Table<Message>| {
    table(value.headers.clone(), value.rows.clone())
});
