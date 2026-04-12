use super::*;
use std::rc::Rc;

const DRAWER_MAX_WIDTH: f32 = 640.0;

pub fn drawer<Message>(
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
                        .percent_width(1.0)
                        .max_width_constraint(DRAWER_MAX_WIDTH)
                        .padding([spacing::LG, spacing::XXL, spacing::XXL, spacing::XXL])
                        .border_radius([radius::LG, radius::LG, 0.0, 0.0])
                        .border_width([1.0, 0.0, 0.0, 0.0])
                        .border_color(color::BORDER)
                        .background_color(color::BACKGROUND)
                        .children(vec![arkit::column_component::<Message, arkit::Theme>()
                            .percent_width(1.0)
                            .children(vec![
                                arkit::row_component::<Message, arkit::Theme>()
                                    .percent_width(1.0)
                                    .justify_content(JustifyContent::Center)
                                    .children(vec![arkit::row_component::<Message, arkit::Theme>()
                                        .width(40.0)
                                        .height(4.0)
                                        .border_radius([
                                            radius::FULL,
                                            radius::FULL,
                                            radius::FULL,
                                            radius::FULL,
                                        ])
                                        .background_color(color::MUTED_FOREGROUND)
                                        .opacity(0.4_f32)
                                        .into()])
                                    .into(),
                                arkit::column_component::<Message, arkit::Theme>()
                                    .margin([spacing::LG, 0.0, 0.0, 0.0])
                                    .children(vec![super::dialog::dialog_header(title, "")])
                                    .into(),
                                arkit::column_component::<Message, arkit::Theme>()
                                    .margin([spacing::LG, 0.0, 0.0, 0.0])
                                    .children(vec![stack(content, spacing::LG)])
                                    .into(),
                            ])
                            .into()]),
                )
                .into(),
            )
        } else {
            None
        },
        arkit::ModalOverlaySpec {
            open,
            presentation: arkit::ModalPresentation::BottomDrawer,
            dismiss_on_backdrop: true,
            backdrop_color: 0x80000000,
            viewport_inset: 0.0,
        },
        Some(dismiss),
    )
}
