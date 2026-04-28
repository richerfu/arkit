use super::*;

pub(super) fn scroll_area<Message: 'static>(
    children: Vec<Element<Message>>,
) -> ScrollElement<Message> {
    panel_surface(
        arkit::scroll_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .children(children),
    )
}
