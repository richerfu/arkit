use crate::ohos_arkui_binding::component::built_in_component::Checkbox;

use super::super::core::ComponentElement;

pub type CheckboxElement = ComponentElement<Checkbox>;

pub fn checkbox_component() -> CheckboxElement {
    ComponentElement::new(Checkbox::new)
}

pub fn checkbox() -> CheckboxElement {
    checkbox_component()
}
