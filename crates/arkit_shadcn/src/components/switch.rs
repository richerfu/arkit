use super::*;

pub fn switch<Message: 'static>(state: bool) -> ToggleElement<Message> {
    shadow_sm(
        arkit::toggle_component::<Message, arkit::Theme>()
            .checked(state)
            .toggle_selected_color(color::PRIMARY)
            .toggle_unselected_color(color::INPUT)
            .toggle_switch_point_color(color::BACKGROUND)
            .border_style(BorderStyle::Solid)
            // RN: `border border-transparent shadow-sm`.
            .border_width([1.0, 1.0, 1.0, 1.0])
            .border_color(0x00000000)
            .border_radius([radius::FULL, radius::FULL, radius::FULL, radius::FULL])
            .clip(true)
            .width(32.0)
            .height(18.4),
    )
}
