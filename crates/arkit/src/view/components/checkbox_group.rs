use crate::ohos_arkui_binding::component::built_in_component::CheckboxGroup;

use super::super::core::ComponentElement;

pub type CheckboxGroupElement = ComponentElement<CheckboxGroup>;

pub fn checkbox_group_component() -> CheckboxGroupElement {
    ComponentElement::new(CheckboxGroup::new)
}

pub fn checkbox_group() -> CheckboxGroupElement {
    checkbox_group_component()
}
