use super::card::card;
use super::*;

fn calendar() -> CalendarPickerElement {
    panel_surface(arkit::calendar_picker_component().height(320.0))
}

fn calendar_card() -> Element {
    card(vec![calendar().into()])
}
