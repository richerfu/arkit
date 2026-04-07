use std::rc::Rc;

use crate::ohos_arkui_binding::component::built_in_component::TextArea;
use crate::prelude::ArkUINodeAttributeType;

use super::super::core::{queue_guarded_ui_callback, run_guarded_ui_callback, ComponentElement};

pub type TextAreaElement = ComponentElement<TextArea>;

pub fn text_area_component() -> TextAreaElement {
    ComponentElement::new(TextArea::new)
}

pub fn text_area(placeholder: impl Into<String>, value: impl Into<String>) -> TextAreaElement {
    text_area_component().placeholder(placeholder).value(value)
}

impl ComponentElement<TextArea> {
    pub fn value(self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.style(ArkUINodeAttributeType::TextAreaText, value.clone())
            .patch_attr(ArkUINodeAttributeType::TextAreaText, value)
    }

    pub fn placeholder(self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.style(ArkUINodeAttributeType::TextAreaPlaceholder, value.clone())
            .patch_attr(ArkUINodeAttributeType::TextAreaPlaceholder, value)
    }

    pub fn placeholder_color(self, value: u32) -> Self {
        self.style(ArkUINodeAttributeType::TextAreaPlaceholderColor, value)
            .patch_attr(ArkUINodeAttributeType::TextAreaPlaceholderColor, value)
    }

    pub fn line_height(self, value: f32) -> Self {
        self.style(ArkUINodeAttributeType::TextAreaLineHeight, value)
            .patch_attr(ArkUINodeAttributeType::TextAreaLineHeight, value)
    }

    pub fn on_change(self, callback: impl Fn(String) + 'static) -> Self {
        let callback = Rc::new(callback);
        self.with(move |node| {
            let callback = callback.clone();
            node.on_text_area_change(move |value| {
                let callback = callback.clone();
                queue_guarded_ui_callback(
                    "text area error: on_change callback panicked",
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
            node.on_text_area_submit(move |enter_key| {
                run_guarded_ui_callback("text area error: on_submit callback panicked", || {
                    (callback.as_ref())(enter_key)
                });
            });
            Ok(())
        })
    }
}
