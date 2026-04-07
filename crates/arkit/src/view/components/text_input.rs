use std::rc::Rc;

use crate::ohos_arkui_binding::component::built_in_component::TextInput;
use crate::prelude::ArkUINodeAttributeType;

use super::super::core::{queue_guarded_ui_callback, run_guarded_ui_callback, ComponentElement};

pub type TextInputElement = ComponentElement<TextInput>;

pub fn text_input_component() -> TextInputElement {
    ComponentElement::new(TextInput::new)
}

pub fn text_input(placeholder: impl Into<String>, value: impl Into<String>) -> TextInputElement {
    text_input_component().placeholder(placeholder).value(value)
}

impl ComponentElement<TextInput> {
    pub fn value(self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.style(ArkUINodeAttributeType::TextInputText, value.clone())
            .patch_attr(ArkUINodeAttributeType::TextInputText, value)
    }

    pub fn placeholder(self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.style(ArkUINodeAttributeType::TextInputPlaceholder, value.clone())
            .patch_attr(ArkUINodeAttributeType::TextInputPlaceholder, value)
    }

    pub fn placeholder_color(self, value: u32) -> Self {
        self.style(ArkUINodeAttributeType::TextInputPlaceholderColor, value)
            .patch_attr(ArkUINodeAttributeType::TextInputPlaceholderColor, value)
    }

    pub fn line_height(self, value: f32) -> Self {
        self.style(ArkUINodeAttributeType::TextInputLineHeight, value)
            .patch_attr(ArkUINodeAttributeType::TextInputLineHeight, value)
    }

    pub fn on_change(self, callback: impl Fn(String) + 'static) -> Self {
        let callback = Rc::new(callback);
        self.with(move |node| {
            let callback = callback.clone();
            node.on_text_input_change(move |value| {
                let callback = callback.clone();
                queue_guarded_ui_callback(
                    "text input error: on_change callback panicked",
                    move || (callback.as_ref())(value),
                );
            });
            Ok(())
        })
    }

    pub fn on_submit(self, callback: impl Fn(i32) + 'static) -> Self {
        let callback = Rc::new(callback);
        self.with(move |node| {
            let callback = callback.clone();
            node.on_text_input_submit(move |enter_key| {
                run_guarded_ui_callback("text input error: on_submit callback panicked", || {
                    (callback.as_ref())(enter_key)
                });
            });
            Ok(())
        })
    }
}
