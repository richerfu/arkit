use crate::ohos_arkui_binding::component::built_in_component::Undefined;

use super::super::core::ComponentElement;

pub type UndefinedElement = ComponentElement<Undefined>;

pub fn undefined_component() -> UndefinedElement {
    ComponentElement::new(Undefined::new)
}

pub fn undefined() -> UndefinedElement {
    undefined_component()
}
