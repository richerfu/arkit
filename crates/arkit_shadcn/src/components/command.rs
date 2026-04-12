use super::*;

pub fn command<Message>(
    query: impl Into<String>,
    options: Vec<String>,
    on_query_change: impl Fn(String) -> Message + Clone + 'static,
) -> Element<Message>
where
    Message: Clone + Send + 'static,
{
    let query = query.into();
    let keyword = query.to_lowercase();
    let mut children = vec![arkit::row_component()
        .border_width([0.0, 0.0, 1.0, 0.0])
        .border_color(color::BORDER)
        .children(vec![input::<Message>("Search command...")
            .value(query.clone())
            .on_input(on_query_change.clone())
            .into()])
        .into()];
    children.extend(
        options
            .iter()
            .filter(|option| keyword.is_empty() || option.to_lowercase().contains(&keyword))
            .map(|option| {
                let option_label = option.clone();
                let click_option = option.clone();
                arkit::row_component()
                    .percent_width(1.0)
                    .height(32.0)
                    .align_items_center()
                    .padding([6.0, spacing::SM, 6.0, spacing::SM])
                    .border_radius([radius::SM, radius::SM, radius::SM, radius::SM])
                    .on_press(on_query_change(click_option.clone()))
                    .children(vec![arkit::text::<Message, arkit::Theme>(option_label)
                        .font_size(typography::SM)
                        .font_color(color::FOREGROUND)
                        .line_height(20.0)
                        .into()])
                    .into()
            }),
    );
    panel_surface(
        arkit::column_component()
            .percent_width(1.0)
            .padding([spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS])
            .children(children),
    )
    .into()
}
