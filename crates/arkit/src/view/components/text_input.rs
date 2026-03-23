use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use crate::ohos_arkui_binding::component::built_in_component::TextInput;
use crate::{logging, queue_ui_loop, Signal};

use super::super::core::ComponentElement;

pub type TextInputElement = ComponentElement<TextInput>;

pub fn text_input_component() -> TextInputElement {
    ComponentElement::new(TextInput::new)
}

pub fn text_input() -> TextInputElement {
    text_input_component()
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

impl ComponentElement<TextInput> {
    pub fn value(self, value: impl Into<String>) -> Self {
        let value = value.into();
        let value_len = value.len();
        self.with(move |node| {
            node.set_text_input_text(value).map_err(|error| {
                logging::error(format!(
                    "text input error: failed to set text (len={value_len}): {error}"
                ));
                error
            })
        })
    }

    pub fn placeholder(self, value: impl Into<String>) -> Self {
        let value = value.into();
        let value_len = value.len();
        self.with(move |node| {
            node.set_text_input_placeholder(value).map_err(|error| {
                logging::error(format!(
                    "text input error: failed to set placeholder (len={value_len}): {error}"
                ));
                error
            })
        })
    }

    pub fn placeholder_color(self, value: u32) -> Self {
        self.with(move |node| {
            node.set_text_input_placeholder_color(value)
                .map_err(|error| {
                    logging::error(format!(
                        "text input error: failed to set placeholder color {value:#010x}: {error}"
                    ));
                    error
                })
        })
    }

    pub fn line_height(self, value: f32) -> Self {
        self.with(move |node| {
            node.set_text_input_line_height(value).map_err(|error| {
                logging::error(format!(
                    "text input error: failed to set line height {value}: {error}"
                ));
                error
            })
        })
    }

    pub fn on_change(self, callback: impl Fn(String) + 'static) -> Self {
        let callback = Rc::new(callback);
        self.with(move |node| {
            let callback = callback.clone();
            node.on_text_input_change(move |value| {
                let callback = callback.clone();
                queue_ui_loop(move || {
                    if let Err(payload) =
                        catch_unwind(AssertUnwindSafe(|| (callback.as_ref())(value)))
                    {
                        logging::error(format!(
                            "text input error: on_change callback panicked: {}",
                            panic_payload_message(payload.as_ref())
                        ));
                    }
                });
            });
            Ok(())
        })
    }

    pub fn on_submit(self, callback: impl Fn(i32) + 'static) -> Self {
        let callback = Rc::new(callback);
        self.with(move |node| {
            let callback = callback.clone();
            node.on_text_input_submit(move |enter_key| {
                if let Err(payload) =
                    catch_unwind(AssertUnwindSafe(|| (callback.as_ref())(enter_key)))
                {
                    logging::error(format!(
                        "text input error: on_submit callback panicked: {}",
                        panic_payload_message(payload.as_ref())
                    ));
                }
            });
            Ok(())
        })
    }

    pub fn bind(self, state: Signal<String>) -> Self {
        let value_state = state.clone();
        self.watch_signal(value_state.clone(), move |node, value| {
            let value_len = value.len();
            node.set_text_input_text(value).map_err(|error| {
                logging::error(format!(
                    "text input error: failed to sync bound text (len={value_len}): {error}"
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
