use crate::ohos_arkui_binding::component::built_in_component::DatePicker;

use super::super::core::ComponentElement;

pub type DatePickerElement = ComponentElement<DatePicker>;

pub fn date_picker_component() -> DatePickerElement {
    ComponentElement::new(DatePicker::new)
}

pub fn date_picker() -> DatePickerElement {
    date_picker_component()
}
