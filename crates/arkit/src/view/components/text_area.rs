use crate::ohos_arkui_binding::component::built_in_component::TextArea;

use super::super::core::ComponentElement;

pub type TextAreaElement = ComponentElement<TextArea>;

pub fn text_area_component() -> TextAreaElement {
    ComponentElement::new(TextArea::new)
}

pub fn text_area() -> TextAreaElement {
    text_area_component()
}
