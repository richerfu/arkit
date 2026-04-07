use super::floating_layer::{floating_panel, FloatingSide};
use super::*;
use std::rc::Rc;

const POPOVER_DEFAULT_WIDTH: f32 = 288.0; // Tailwind `w-72`

pub fn popover(trigger: Element, content: Vec<Element>, open: Signal<bool>) -> Element {
    popover_with_width(trigger, content, open, POPOVER_DEFAULT_WIDTH)
}

pub fn popover_with_width(
    trigger: Element,
    content: Vec<Element>,
    open: Signal<bool>,
    width: f32,
) -> Element {
    let dismiss = {
        let open = open.clone();
        Rc::new(move || open.set(false))
    };
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
                .children(vec![stack(content, spacing::LG)]),
        )
        .into(),
        open,
        FloatingSide::Bottom,
        Some(dismiss),
    )
}

pub fn popover_card(title: impl Into<String>, body: impl Into<String>) -> Element {
    card(vec![title_text(title).into(), muted_text(body).into()])
}
