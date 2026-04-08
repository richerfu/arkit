use super::*;

pub fn switch<Message: 'static>(state: bool) -> ToggleElement<Message> {
    shadow_sm(
        arkit::toggle_component::<Message, arkit::Theme>()
            .patch_attr(ArkUINodeAttributeType::ToggleValue, state)
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
            .height(18.4),
    )
}
