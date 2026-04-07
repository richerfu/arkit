use super::*;
use std::rc::Rc;

pub(crate) const DIALOG_MAX_WIDTH: f32 = 512.0;
const DIALOG_VIEWPORT_INSET: f32 = spacing::LG;

pub(crate) fn modal_overlay(
    open: Signal<bool>,
    panel: Element,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element {
    let mut backdrop = arkit::row_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .background_color(0x80000000);

    if let Some(dismiss) = on_dismiss {
        let dismiss_open = open.clone();
        backdrop = backdrop.on_click(move || {
            if dismiss_open.get() {
                dismiss();
            }
        });
    }

    visibility_gate(
        arkit::stack_component()
            .percent_width(1.0)
            .percent_height(1.0),
        open.clone(),
    )
    .children(vec![
        backdrop.into(),
        arkit::column_component()
            .percent_width(1.0)
            .percent_height(1.0)
            .style(
                ArkUINodeAttributeType::ColumnJustifyContent,
                FLEX_ALIGN_CENTER,
            )
            .align_items_center()
            .style(
                ArkUINodeAttributeType::Padding,
                vec![
                    DIALOG_VIEWPORT_INSET,
                    DIALOG_VIEWPORT_INSET,
                    DIALOG_VIEWPORT_INSET,
                    DIALOG_VIEWPORT_INSET,
                ],
            )
            .children(vec![panel])
            .into(),
    ])
    .into()
}

pub fn dialog(_title: impl Into<String>, open: Signal<bool>, content: Vec<Element>) -> Element {
    let dismiss = {
        let open = open.clone();
        Rc::new(move || open.set(false))
    };
    let close = dismiss.clone();

    modal_overlay(
        open,
        shadow_sm(
            arkit::stack_component()
                .percent_width(1.0)
                .max_width_constraint(DIALOG_MAX_WIDTH)
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
                            .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
                            .style(ArkUINodeAttributeType::Opacity, 0.7_f32)
                            .on_click(move || close())
                            .into()])
                        .into(),
                ]),
        )
        .into(),
        Some(dismiss),
    )
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
