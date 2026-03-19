use crate::ohos_arkui_binding::component::built_in_component::TimePicker;

use super::super::core::ComponentElement;

pub type TimePickerElement = ComponentElement<TimePicker>;

pub fn time_picker_component() -> TimePickerElement {
    ComponentElement::new(TimePicker::new)
}

pub fn time_picker() -> TimePickerElement {
    time_picker_component()
}
