use super::*;

pub fn switch(state: Signal<bool>) -> ToggleElement {
    let next = state.clone();
    shadow_sm(
        arkit::toggle_component()
            .watch_signal(state.clone(), move |node, value| {
                node.set_toggle_value(value)
            })
            .style(ArkUINodeAttributeType::ToggleSelectedColor, color::PRIMARY)
            .style(ArkUINodeAttributeType::ToggleUnselectedColor, color::INPUT)
            .style(
                ArkUINodeAttributeType::ToggleSwitchPointColor,
                color::BACKGROUND,
            )
            .style(ArkUINodeAttributeType::BorderStyle, 0_i32)
            // RN: `border border-transparent shadow-sm`.
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![1.0, 1.0, 1.0, 1.0],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![0x00000000])
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::FULL, radius::FULL, radius::FULL, radius::FULL],
            )
            .style(ArkUINodeAttributeType::Clip, true)
            .width(32.0)
            .height(18.4)
            .on_click(move || next.update(|value| *value = !*value)),
    )
}
