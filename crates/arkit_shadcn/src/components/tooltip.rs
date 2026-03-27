use super::floating_layer::floating_panel;
use super::*;
use arkit::{component, create_signal};
use std::rc::Rc;

#[component]
pub fn tooltip(trigger_label: impl Into<String>, content: impl Into<String>) -> Element {
    let trigger_label = trigger_label.into();
    let content = content.into();
    let open = create_signal(false);
    let toggle = open.clone();
    let dismiss = {
        let open = open.clone();
        Rc::new(move || open.set(false))
    };

    floating_panel(
        button(trigger_label, ButtonVariant::Outline)
            .on_click(move || toggle.update(|value| *value = !*value))
            .into(),
        arkit::row_component()
            .style(ArkUINodeAttributeType::Padding, vec![8.0, 12.0, 8.0, 12.0])
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::MD, radius::MD, radius::MD, radius::MD],
            )
            .background_color(color::PRIMARY)
            .children(vec![arkit::text(content)
                .font_size(typography::XS)
                .style(ArkUINodeAttributeType::FontColor, color::PRIMARY_FOREGROUND)
                .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
                .into()])
            .into(),
        open.get(),
        super::floating_layer::FloatingSide::Top,
        Some(dismiss),
    )
}
