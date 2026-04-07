use super::*;

pub fn radio_group(options: Vec<String>, selected: Signal<String>) -> Element {
    let children = options
        .into_iter()
        .map(|option| {
            let sel = selected.clone();
            let sel_val = selected.clone();
            let opt = option.clone();

            // Wrap each radio option in `dynamic` so the checked state
            // reactively updates when the `selected` signal changes.
            // The previous code read `selected.get()` once at construction time,
            // so the radio checked visual never updated.
            arkit::dynamic(move || {
                let is_checked = sel.get() == opt;
                let sel_val = sel_val.clone();
                let opt_click = opt.clone();

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
                                        sel_val.set(opt_click.clone());
                                    }
                                }),
                        )
                        .into(),
                        arkit::row_component()
                            .style(
                                ArkUINodeAttributeType::Margin,
                                vec![0.0, 0.0, 0.0, spacing::MD],
                            )
                            .children(vec![label(opt.clone()).into()])
                            .into(),
                    ])
                    .into()
            })
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
