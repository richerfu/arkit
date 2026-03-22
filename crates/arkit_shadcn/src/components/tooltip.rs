use super::*;
use arkit::{component, use_signal};

#[component]
pub fn tooltip(trigger_label: impl Into<String>, content: impl Into<String>) -> Element {
    let trigger_label = trigger_label.into();
    let content = content.into();
    let open = use_signal(|| false);
    let toggle = open.clone();

    arkit::column_component()
        .style(ArkUINodeAttributeType::ColumnAlignItems, FLEX_ALIGN_CENTER)
        .children(vec![
            if open.get() {
                shadow_sm(
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
                            .into()]),
                )
                .into()
            } else {
                arkit::row_component().height(0.0).into()
            },
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![if open.get() { spacing::XXS } else { 0.0 }, 0.0, 0.0, 0.0],
                )
                .children(vec![button(trigger_label, ButtonVariant::Outline)
                    .on_click(move || toggle.update(|value| *value = !*value))
                    .into()])
                .into(),
        ])
        .into()
}
