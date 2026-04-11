use super::*;
use std::rc::Rc;

const SHEET_WIDTH: f32 = 384.0;

pub fn sheet<Message>(
    title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    content: Vec<Element<Message>>,
) -> Element<Message>
where
    Message: Send + 'static,
{
    let title = title.into();
    let dismiss = Rc::new(move || dispatch_message(on_open_change(false)));

    arkit::modal_overlay(
        if open {
            Some(
                shadow_sm(
                    arkit::stack_component::<Message, arkit::Theme>()
                        .width(SHEET_WIDTH)
                        .percent_height(1.0)
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![spacing::XXL, spacing::XXL, spacing::XXL, spacing::XXL],
                        )
                        .style(
                            ArkUINodeAttributeType::BorderRadius,
                            vec![0.0, 0.0, radius::LG, radius::LG],
                        )
                        .style(
                            ArkUINodeAttributeType::BorderWidth,
                            vec![0.0, 0.0, 0.0, 1.0],
                        )
                        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
                        .background_color(color::BACKGROUND)
                        .children(vec![
                            arkit::column_component::<Message, arkit::Theme>()
                                .percent_width(1.0)
                                .percent_height(1.0)
                                .children(vec![stack(
                                    vec![
                                        super::dialog::dialog_header(title, ""),
                                        stack(content, spacing::LG),
                                    ],
                                    spacing::LG,
                                )])
                                .into(),
                            arkit::row_component::<Message, arkit::Theme>()
                                .percent_width(1.0)
                                .style(ArkUINodeAttributeType::Position, vec![0.0, 0.0])
                                .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_END)
                                .children(vec![icon_button_with_variant::<Message>(
                                    "x",
                                    ButtonVariant::Ghost,
                                )
                                .width(28.0)
                                .height(28.0)
                                .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
                                .style(ArkUINodeAttributeType::Opacity, 0.7_f32)
                                .on_click({
                                    let dismiss = dismiss.clone();
                                    move || dismiss()
                                })
                                .into()])
                                .into(),
                        ]),
                )
                .into(),
            )
        } else {
            None
        },
        arkit::ModalOverlaySpec {
            open,
            presentation: arkit::ModalPresentation::RightSheet,
            dismiss_on_backdrop: true,
            backdrop_color: 0x80000000,
            viewport_inset: 0.0,
        },
        Some(dismiss),
    )
}
