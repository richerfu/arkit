use super::*;

pub fn dialog(_title: impl Into<String>, open: Signal<bool>, content: Vec<Element>) -> Element {
    if !open.get() {
        return arkit::row_component().into();
    }
    let close = open.clone();
    let close_overlay = open.clone();

    arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![
            arkit::row_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .background_color(0x80000000)
                .on_click(move || close_overlay.set(false))
                .into(),
            arkit::column_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .style(
                    ArkUINodeAttributeType::ColumnJustifyContent,
                    FLEX_ALIGN_CENTER,
                )
                .style(ArkUINodeAttributeType::ColumnAlignItems, FLEX_ALIGN_CENTER)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![spacing::SM, spacing::SM, spacing::SM, spacing::SM],
                )
                .children(vec![shadow_sm(
                    arkit::stack_component()
                        .percent_width(1.0)
                        .max_width_constraint(425.0)
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![spacing::XXL, spacing::XXL, spacing::XXL, spacing::XXL],
                        )
                        .style(
                            ArkUINodeAttributeType::BorderRadius,
                            vec![radius::LG, radius::LG, radius::LG, radius::LG],
                        )
                        .style(
                            ArkUINodeAttributeType::BorderWidth,
                            vec![1.0, 1.0, 1.0, 1.0],
                        )
                        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
                        .background_color(color::BACKGROUND)
                        .children(vec![
                            arkit::column_component()
                                .percent_width(1.0)
                                .children(vec![stack(content, spacing::LG)])
                                .into(),
                            arkit::row_component()
                                .percent_width(1.0)
                                .style(ArkUINodeAttributeType::Position, vec![0.0, 0.0])
                                .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_END)
                                .children(vec![icon_button_with_variant("x", ButtonVariant::Ghost)
                                    .width(28.0)
                                    .height(28.0)
                                    .style(
                                        ArkUINodeAttributeType::Padding,
                                        vec![0.0, 0.0, 0.0, 0.0],
                                    )
                                    .style(ArkUINodeAttributeType::Opacity, 0.7_f32)
                                    .on_click(move || close.set(false))
                                    .into()])
                                .into(),
                        ]),
                )
                .into()])
                .into(),
        ])
        .into()
}

pub fn dialog_footer(actions: Vec<Element>) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .children(
            actions
                .into_iter()
                .rev()
                .enumerate()
                .map(|(index, child)| {
                    if index == 0 {
                        child
                    } else {
                        arkit::row_component()
                            .percent_width(1.0)
                            .style(
                                ArkUINodeAttributeType::Margin,
                                vec![spacing::SM, 0.0, 0.0, 0.0],
                            )
                            .children(vec![child])
                            .into()
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into()
}

pub fn dialog_header(title: impl Into<String>, description: impl Into<String>) -> Element {
    let title = title.into();
    let description = description.into();
    let mut children = vec![arkit::text(title)
        .font_size(typography::LG)
        .style(ArkUINodeAttributeType::FontWeight, 5_i32)
        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
        .style(ArkUINodeAttributeType::TextLineHeight, 18.0)
        .into()];
    if !description.is_empty() {
        children.push(
            margin_top(
                arkit::text(description)
                    .font_size(typography::SM)
                    .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
                    .style(ArkUINodeAttributeType::TextLineHeight, 20.0),
                spacing::SM,
            )
            .into(),
        );
    }
    arkit::column_component()
        .percent_width(1.0)
        .children(children)
        .into()
}
