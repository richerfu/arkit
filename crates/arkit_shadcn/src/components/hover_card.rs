use super::floating_layer::{floating_panel, FloatingSide};
use super::*;

const HOVER_CARD_DEFAULT_WIDTH: f32 = 256.0; // Tailwind `w-64`

pub fn hover_card<Message: 'static>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    show: bool,
    on_show_change: impl Fn(bool) + 'static,
) -> Element<Message> {
    hover_card_with_width(
        trigger,
        content,
        show,
        on_show_change,
        HOVER_CARD_DEFAULT_WIDTH,
    )
}

pub fn hover_card_message<Message>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    show: bool,
    on_show_change: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    hover_card(trigger, content, show, move |value| {
        dispatch_message(on_show_change(value))
    })
}

pub fn hover_card_with_width<Message: 'static>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    show: bool,
    on_show_change: impl Fn(bool) + 'static,
    width: f32,
) -> Element<Message> {
    floating_panel(
        trigger,
        panel_surface(
            arkit::column_component::<Message, arkit::Theme>()
                .width(width)
                .align_items_start()
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![spacing::LG, spacing::LG, spacing::LG, spacing::LG],
                )
                .children(vec![stack(content, spacing::MD)]),
        )
        .into(),
        show,
        FloatingSide::Bottom,
        Some(std::rc::Rc::new(move || on_show_change(false))),
    )
}

pub fn hover_card_with_width_message<Message>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    show: bool,
    on_show_change: impl Fn(bool) -> Message + 'static,
    width: f32,
) -> Element<Message>
where
    Message: Send + 'static,
{
    hover_card_with_width(
        trigger,
        content,
        show,
        move |value| dispatch_message(on_show_change(value)),
        width,
    )
}
