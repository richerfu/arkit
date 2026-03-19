use crate::ohos_arkui_binding::component::built_in_component::Custom;

use super::super::core::ComponentElement;

pub type CustomElement = ComponentElement<Custom>;

pub fn custom_component() -> CustomElement {
    ComponentElement::new(Custom::new)
}

pub fn custom() -> CustomElement {
    custom_component()
}
