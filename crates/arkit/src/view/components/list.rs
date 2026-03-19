use crate::ohos_arkui_binding::component::built_in_component::List;

use super::super::core::ComponentElement;

pub type ListElement = ComponentElement<List>;

pub fn list_component() -> ListElement {
    ComponentElement::new(List::new)
}

pub fn list() -> ListElement {
    list_component()
}
