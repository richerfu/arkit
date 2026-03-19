use crate::ohos_arkui_binding::component::built_in_component::CalendarPicker;

use super::super::core::ComponentElement;

pub type CalendarPickerElement = ComponentElement<CalendarPicker>;

pub fn calendar_picker_component() -> CalendarPickerElement {
    ComponentElement::new(CalendarPicker::new)
}

pub fn calendar_picker() -> CalendarPickerElement {
    calendar_picker_component()
}
