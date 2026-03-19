use crate::ohos_arkui_binding::component::built_in_component::Button;

use super::super::core::ComponentElement;

pub type ButtonElement = ComponentElement<Button>;

pub fn button_component() -> ButtonElement {
    ComponentElement::new(Button::new)
}

pub fn button(label: impl Into<String>) -> ButtonElement {
    button_component().label(label)
}

impl ComponentElement<Button> {
    pub fn label(self, label: impl Into<String>) -> Self {
        let label = label.into();
        self.with(move |node| node.set_button_label(label))
    }
}
