use crate::ohos_arkui_binding::component::built_in_component::Radio;

use super::super::core::ComponentElement;

pub type RadioElement = ComponentElement<Radio>;

pub fn radio_component() -> RadioElement {
    ComponentElement::new(Radio::new)
}

pub fn radio() -> RadioElement {
    radio_component()
}
