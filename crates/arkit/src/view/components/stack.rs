use crate::ohos_arkui_binding::component::built_in_component::Stack;

use super::super::core::ComponentElement;

pub type StackElement = ComponentElement<Stack>;

pub fn stack_component() -> StackElement {
    ComponentElement::new(Stack::new)
}

pub fn stack() -> StackElement {
    stack_component()
}
