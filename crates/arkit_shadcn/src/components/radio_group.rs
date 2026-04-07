use super::*;

pub fn radio_group(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) + 'static,
) -> Element {
    let selected = selected.into();
    let on_select = std::rc::Rc::new(on_select);
    let children = options
        .into_iter()
        .map(|option| {
            let opt = option.clone();
            let is_checked = selected == opt;
            let on_select = on_select.clone();
            arkit::row_component()
                .percent_width(1.0)
                .align_items_center()
                .children(vec![
                    shadow_sm(
                        arkit::radio_component()
                            .patch_attr(ArkUINodeAttributeType::RadioChecked, is_checked)
                            .style(ArkUINodeAttributeType::BorderColor, vec![color::INPUT])
                            .style(
                                ArkUINodeAttributeType::BorderWidth,
                                vec![1.0, 1.0, 1.0, 1.0],
                            )
                            .style(ArkUINodeAttributeType::RadioStyle, vec![color::PRIMARY])
                            .width(16.0)
                            .height(16.0)
                            .on_change(move |value| {
                                if value {
                                    on_select(opt.clone());
                                }
                            }),
                    )
                    .into(),
                    arkit::row_component()
                        .style(
                            ArkUINodeAttributeType::Margin,
                            vec![0.0, 0.0, 0.0, spacing::MD],
                        )
                        .children(vec![label(option).into()])
                        .into(),
                ])
                .into()
        })
        .collect::<Vec<Element>>();

    arkit::column_component()
        .percent_width(1.0)
        .children(
            children
                .into_iter()
                .enumerate()
                .map(|(index, child)| {
                    if index == 0 {
                        child
                    } else {
                        arkit::row_component()
                            .style(
                                ArkUINodeAttributeType::Margin,
                                vec![spacing::MD, 0.0, 0.0, 0.0],
                            )
                            .children(vec![child])
                            .into()
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into()
}
