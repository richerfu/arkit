use std::rc::Rc;

use crate::logging;
use crate::ohos_arkui_binding::component::built_in_component::TextArea;
use crate::Signal;

use super::super::core::{queue_guarded_ui_callback, run_guarded_ui_callback, ComponentElement};

pub type TextAreaElement = ComponentElement<TextArea>;

pub fn text_area_component() -> TextAreaElement {
    ComponentElement::new(TextArea::new)
}

pub fn text_area() -> TextAreaElement {
    text_area_component()
}

impl ComponentElement<TextArea> {
    pub fn value(self, value: impl Into<String>) -> Self {
        let value = value.into();
        let value_len = value.len();
        self.with(move |node| {
            node.set_text_area_text(value).map_err(|error| {
                logging::error(format!(
                    "text area error: failed to set text (len={value_len}): {error}"
                ));
                error
            })
        })
    }

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

    pub fn bind(self, state: Signal<String>) -> Self {
        let value_state = state.clone();
        self.watch_signal(value_state, move |node, value| {
            let value_len = value.len();
            node.set_text_area_text(value).map_err(|error| {
                logging::error(format!(
                    "text area error: failed to sync bound text (len={value_len}): {error}"
                ));
                error
            })
        })
        .on_change(move |value| {
            if state.get() != value {
                state.set(value);
            }
        })
    }
}
