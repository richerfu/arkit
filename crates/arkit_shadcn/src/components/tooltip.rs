use super::floating_layer::floating_panel;
use super::*;
use std::rc::Rc;

pub fn tooltip<Message: 'static>(
    trigger: Element<Message>,
    content: impl Into<String> + 'static,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
) -> Element<Message> {
    let content = content.into();
    let dismiss = { Rc::new(move || on_open_change(false)) };

    floating_panel(
        trigger,
        arkit::row_component::<Message, arkit::Theme>()
            .style(ArkUINodeAttributeType::Padding, vec![8.0, 12.0, 8.0, 12.0])
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::MD, radius::MD, radius::MD, radius::MD],
            )
            .background_color(color::PRIMARY)
            .children(vec![arkit::text::<Message, arkit::Theme>(content)
                .font_size(typography::XS)
                .style(ArkUINodeAttributeType::FontColor, color::PRIMARY_FOREGROUND)
                .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
                .into()])
            .into(),
        open,
        super::floating_layer::FloatingSide::Top,
        Some(dismiss),
    )
}

pub fn tooltip_message<Message>(
    trigger: Element<Message>,
    content: impl Into<String> + 'static,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    tooltip(trigger, content, open, move |value| {
        dispatch_message(on_open_change(value))
    })
}
