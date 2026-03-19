use crate::ohos_arkui_binding::component::built_in_component::Flex;

use super::super::core::ComponentElement;

pub type FlexElement = ComponentElement<Flex>;

pub fn flex_component() -> FlexElement {
    ComponentElement::new(Flex::new)
}

pub fn flex() -> FlexElement {
    flex_component()
}
