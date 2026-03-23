use crate::logging;
use crate::ohos_arkui_binding::component::built_in_component::TextArea;

use super::super::core::ComponentElement;

pub type TextAreaElement = ComponentElement<TextArea>;

pub fn text_area_component() -> TextAreaElement {
    ComponentElement::new(TextArea::new)
}

pub fn text_area() -> TextAreaElement {
    text_area_component()
}

impl ComponentElement<TextArea> {
    pub fn placeholder(self, value: impl Into<String>) -> Self {
        let value = value.into();
        let value_len = value.len();
        self.with(move |node| {
            node.set_text_area_placeholder(value).map_err(|error| {
                logging::error(format!(
                    "text area error: failed to set placeholder (len={value_len}): {error}"
                ));
                error
            })
        })
    }

    pub fn placeholder_color(self, value: u32) -> Self {
        self.with(move |node| {
            node.set_text_area_placeholder_color(value)
                .map_err(|error| {
                    logging::error(format!(
                        "text area error: failed to set placeholder color {value:#010x}: {error}"
                    ));
                    error
                })
        })
    }

    pub fn line_height(self, value: f32) -> Self {
        self.with(move |node| {
            node.set_text_area_line_height(value).map_err(|error| {
                logging::error(format!(
                    "text area error: failed to set line height {value}: {error}"
                ));
                error
            })
        })
    }
}
