use crate::ohos_arkui_binding::component::built_in_component::TextInput;
use crate::Signal;

use super::super::core::ComponentElement;

pub type TextInputElement = ComponentElement<TextInput>;

pub fn text_input_component() -> TextInputElement {
    ComponentElement::new(TextInput::new)
}

pub fn text_input() -> TextInputElement {
    text_input_component()
}

impl ComponentElement<TextInput> {
    pub fn value(self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.with(move |node| node.set_text_input_text(value))
    }

    pub fn placeholder(self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.with(move |node| node.set_text_input_placeholder(value))
    }

    pub fn on_change(self, callback: impl Fn(String) + 'static) -> Self {
        self.with(move |node| {
            node.on_text_input_change(move |value| callback(value));
            Ok(())
        })
    }

    pub fn on_submit(self, callback: impl Fn(i32) + 'static) -> Self {
        self.with(move |node| {
            node.on_text_input_submit(move |enter_key| callback(enter_key));
            Ok(())
        })
    }

    pub fn bind(self, state: Signal<String>) -> Self {
        let value_state = state.clone();
        self.value(value_state.get())
            .on_change(move |value| {
                if state.get() != value {
                    state.set(value);
                }
            })
    }
}
