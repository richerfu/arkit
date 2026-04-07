use super::*;

pub fn command(query: Signal<String>, options: Vec<String>) -> Element {
    // Wrap the entire panel in `dynamic` so the filtered option list
    // re-filters whenever the `query` signal changes.
    // The previous code read `query.get()` once at construction time,
    // so the filtered option list never updated when the user typed.
    arkit::dynamic({
        let query = query.clone();
        let options = options.clone();
        move || {
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
                    .iter()
                    .filter(|option| keyword.is_empty() || option.to_lowercase().contains(&keyword))
                    .map(|option| {
                        let value = query.clone();
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
                            .on_click(move || value.set(click_option.clone()))
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
    })
}
