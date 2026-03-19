use crate::ohos_arkui_binding::component::built_in_component::XComponent;

use super::super::core::ComponentElement;

pub type XComponentElement = ComponentElement<XComponent>;

pub fn xcomponent_component() -> XComponentElement {
    ComponentElement::new(XComponent::new)
}

pub fn xcomponent() -> XComponentElement {
    xcomponent_component()
}
