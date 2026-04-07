use super::*;

pub fn command(
    query: impl Into<String>,
    options: Vec<String>,
    on_query_change: impl Fn(String) + 'static,
) -> Element {
    let query = query.into();
    let keyword = query.to_lowercase();
    let on_query_change = std::rc::Rc::new(on_query_change);
    let mut children = vec![arkit::row_component()
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![0.0, 0.0, 1.0, 0.0],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .children(vec![input("Search command...")
            .value(query.clone())
            .on_change({
                let on_query_change = on_query_change.clone();
                move |value| on_query_change(value)
            })
            .into()])
        .into()];
    children.extend(
        options
            .iter()
            .filter(|option| keyword.is_empty() || option.to_lowercase().contains(&keyword))
            .map(|option| {
                let on_query_change = on_query_change.clone();
                let option_label = option.clone();
                let click_option = option.clone();
                arkit::row_component()
                    .percent_width(1.0)
                    .height(32.0)
                    .align_items_center()
                    .style(
                        ArkUINodeAttributeType::Padding,
                        vec![6.0, spacing::SM, 6.0, spacing::SM],
                    )
                    .style(
                        ArkUINodeAttributeType::BorderRadius,
                        vec![radius::SM, radius::SM, radius::SM, radius::SM],
                    )
                    .on_click(move || on_query_change(click_option.clone()))
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
