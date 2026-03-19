use crate::ohos_arkui_binding::component::built_in_component::Toggle;

use super::super::core::ComponentElement;

pub type ToggleElement = ComponentElement<Toggle>;

pub fn toggle_component() -> ToggleElement {
    ComponentElement::new(Toggle::new)
}

pub fn toggle() -> ToggleElement {
    toggle_component()
}
