use super::*;
use std::rc::Rc;

pub fn tabs(
    tab_labels: Vec<String>,
    active: usize,
    on_change: impl Fn(usize) + 'static,
    panels: Vec<Element>,
) -> Element {
    arkit::column_component()
        .width(arkit::Length::Fill)
        .children(vec![
            tabs_list(tab_labels, active, Rc::new(on_change)),
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

fn tabs_list(tab_labels: Vec<String>, active: usize, on_change: Rc<dyn Fn(usize)>) -> Element {
    let children = tab_labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            let is_active = active == index;
            let on_change = on_change.clone();
            arkit::row_component()
                .height(35.0)
                .align_items_center()
                .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
                .padding([spacing::XXS, spacing::SM, spacing::XXS, spacing::SM])
                .style(
                    ArkUINodeAttributeType::BorderRadius,
                    vec![radius::MD, radius::MD, radius::MD, radius::MD],
                )
                .style(
                    ArkUINodeAttributeType::BorderWidth,
                    vec![1.0, 1.0, 1.0, 1.0],
                )
                .style(ArkUINodeAttributeType::BorderColor, vec![0x00000000])
                .patch_background_color(if is_active {
                    color::BACKGROUND
                } else {
                    0x00000000
                })
                .on_click(move || on_change(index))
                .children(vec![body_text(label)
                    .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                    .into()])
                .into()
        })
        .collect::<Vec<_>>();

    rounded_tabs_list_surface(
        arkit::row_component()
            .align_items_center()
            .children(children),
    )
    .into()
}

fn tabs_content(panels: Vec<Element>, active: usize) -> Element {
    let panel_containers: Vec<Element> = panels
        .into_iter()
        .enumerate()
        .map(|(index, panel)| {
            let is_active = active == index;
            arkit::column_component()
                .width(arkit::Length::Fill)
                .style(
                    ArkUINodeAttributeType::Visibility,
                    if is_active { 0_i32 } else { 2_i32 },
                )
                .children(vec![panel])
                .into()
        })
        .collect();

    arkit::stack_component()
        .width(arkit::Length::Fill)
        .children(panel_containers)
        .into()
}
