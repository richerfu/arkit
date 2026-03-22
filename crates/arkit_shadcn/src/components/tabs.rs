use super::*;

pub fn tabs(tab_labels: Vec<String>, active: Signal<usize>, panels: Vec<Element>) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .children(vec![
            tabs_list(tab_labels, active.clone()),
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![spacing::SM, 0.0, 0.0, 0.0],
                )
                .children(vec![tabs_content(panels, active)])
                .into(),
        ])
        .into()
}

fn tabs_list(tab_labels: Vec<String>, active: Signal<usize>) -> Element {
    let children = tab_labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            let click = active.clone();
            let is_active = active.get() == index;
            let trigger = arkit::row_component()
                // Match RN `TabsList h-9 p-[3px]` with `TabsTrigger h-[calc(100%-1px)]`.
                .height(35.0)
                .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![spacing::XXS, spacing::SM, spacing::XXS, spacing::SM],
                )
                .style(
                    ArkUINodeAttributeType::BorderRadius,
                    vec![radius::MD, radius::MD, radius::MD, radius::MD],
                )
                .style(
                    ArkUINodeAttributeType::BorderWidth,
                    vec![1.0, 1.0, 1.0, 1.0],
                )
                .style(ArkUINodeAttributeType::BorderColor, vec![0x00000000])
                .background_color(if is_active {
                    color::BACKGROUND
                } else {
                    0x00000000
                })
                .on_click(move || click.set(index))
                .children(vec![body_text(label)
                    .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                    .into()]);

            trigger.into()
        })
        .collect::<Vec<_>>();

    rounded_tabs_list_surface(
        arkit::row_component()
            .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
            .children(children),
    )
    .into()
}

fn tabs_content(panels: Vec<Element>, active: Signal<usize>) -> Element {
    panels
        .into_iter()
        .nth(active.get())
        .unwrap_or_else(|| arkit::column(vec![]))
}
