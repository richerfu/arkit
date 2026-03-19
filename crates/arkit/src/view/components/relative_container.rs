use crate::ohos_arkui_binding::component::built_in_component::RelativeContainer;

use super::super::core::ComponentElement;

pub type RelativeContainerElement = ComponentElement<RelativeContainer>;

pub fn relative_container_component() -> RelativeContainerElement {
    ComponentElement::new(RelativeContainer::new)
}

pub fn relative_container() -> RelativeContainerElement {
    relative_container_component()
}
