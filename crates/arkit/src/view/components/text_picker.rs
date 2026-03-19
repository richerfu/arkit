use crate::ohos_arkui_binding::component::built_in_component::TextPicker;

use super::super::core::ComponentElement;

pub type TextPickerElement = ComponentElement<TextPicker>;

pub fn text_picker_component() -> TextPickerElement {
    ComponentElement::new(TextPicker::new)
}

pub fn text_picker() -> TextPickerElement {
    text_picker_component()
}
