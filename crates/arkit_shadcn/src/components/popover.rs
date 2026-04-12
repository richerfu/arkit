use super::floating_layer::{floating_panel, FloatingSide};
use super::*;
use std::rc::Rc;

const POPOVER_DEFAULT_WIDTH: f32 = 288.0; // Tailwind `w-72`

pub fn popover<Message: 'static>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
) -> Element<Message> {
    popover_with_width(
        trigger,
        content,
        open,
        on_open_change,
        POPOVER_DEFAULT_WIDTH,
    )
}

pub fn popover_message<Message>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    popover(trigger, content, open, move |value| {
        dispatch_message(on_open_change(value))
    })
}

pub fn popover_with_width<Message: 'static>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    width: f32,
) -> Element<Message> {
    let dismiss = Rc::new(move || {
        on_open_change(false);
    });
    floating_panel(
        trigger,
        panel_surface(
            arkit::column_component::<Message, arkit::Theme>()
                .width(width)
                .align_items_start()
                .padding([spacing::LG, spacing::LG, spacing::LG, spacing::LG])
                .children(vec![stack(content, spacing::LG)]),
        )
        .into(),
        open,
        FloatingSide::Bottom,
        Some(dismiss),
    )
}

pub fn popover_with_width_message<Message>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    width: f32,
) -> Element<Message>
where
    Message: Send + 'static,
{
    popover_with_width(
        trigger,
        content,
        open,
        move |value| dispatch_message(on_open_change(value)),
        width,
    )
}

pub fn popover_card<Message: 'static>(
    title: impl Into<String>,
    body: impl Into<String>,
) -> Element<Message> {
    card(vec![title_text(title).into(), muted_text(body).into()])
}
