use crate::ohos_arkui_binding::component::built_in_component::EmbeddedComponent;

use super::super::core::ComponentElement;

pub type EmbeddedComponentElement = ComponentElement<EmbeddedComponent>;

pub fn embedded_component_component() -> EmbeddedComponentElement {
    ComponentElement::new(EmbeddedComponent::new)
}

pub fn embedded_component() -> EmbeddedComponentElement {
    embedded_component_component()
}
