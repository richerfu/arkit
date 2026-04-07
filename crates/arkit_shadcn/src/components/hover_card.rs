use super::floating_layer::{floating_panel, FloatingSide};
use super::*;

const HOVER_CARD_DEFAULT_WIDTH: f32 = 256.0; // Tailwind `w-64`

pub fn hover_card(
    trigger: Element,
    content: Vec<Element>,
    show: bool,
    on_show_change: impl Fn(bool) + 'static,
) -> Element {
    hover_card_with_width(trigger, content, show, on_show_change, HOVER_CARD_DEFAULT_WIDTH)
}

pub fn hover_card_with_width(
    trigger: Element,
    content: Vec<Element>,
    show: bool,
    _on_show_change: impl Fn(bool) + 'static,
    width: f32,
) -> Element {
    floating_panel(
        trigger,
        panel_surface(
            arkit::column_component()
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
        None,
    )
}
