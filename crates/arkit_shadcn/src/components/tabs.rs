use super::*;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;

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
            let active_sig = active.clone();

            arkit::dynamic(move || {
                let is_active = active_sig.get() == index;
                let click = click.clone();

                arkit::row_component()
                    .height(35.0)
                    .align_items_center()
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
                    .patch_background_color(if is_active {
                        color::BACKGROUND
                    } else {
                        0x00000000
                    })
                    .on_click(move || click.set(index))
                    .children(vec![body_text(label.clone())
                        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                        .into()])
                    .into()
            })
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

fn tabs_content(panels: Vec<Element>, active: Signal<usize>) -> Element {
    // Render all panels in a stack, each with its own watch_signal
    // to toggle visibility based on the active index. Only the active
    // panel is visible; all others are gone (visibility = 2).
    let panel_containers: Vec<Element> = panels
        .into_iter()
        .enumerate()
        .map(|(index, panel)| {
            let is_active = active.get() == index;
            arkit::column_component()
                .percent_width(1.0)
                .style(
                    ArkUINodeAttributeType::Visibility,
                    if is_active { 0_i32 } else { 2_i32 },
                )
                .watch_signal(active.clone(), move |node, current| {
                    node.set_visibility(if current == index { 0_i32 } else { 2_i32 })
                })
                .children(vec![panel])
                .into()
        })
        .collect();

    arkit::stack_component()
        .percent_width(1.0)
        .children(panel_containers)
        .into()
}
