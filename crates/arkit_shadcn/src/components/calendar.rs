use super::*;

pub fn calendar() -> CalendarPickerElement {
    panel_surface(arkit::calendar_picker_component().height(320.0))
}

pub fn calendar_card() -> Element {
    card(vec![calendar().into()])
}
