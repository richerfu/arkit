use std::rc::Rc;

use crate::ohos_arkui_binding::component::built_in_component::Radio;

use super::super::core::{queue_guarded_ui_callback, ComponentElement};

pub type RadioElement = ComponentElement<Radio>;

pub fn radio_component() -> RadioElement {
    ComponentElement::new(Radio::new)
}

pub fn radio() -> RadioElement {
    radio_component()
}

impl ComponentElement<Radio> {
    pub fn on_change(self, callback: impl Fn(bool) + 'static) -> Self {
        let callback = Rc::new(callback);
        self.with(move |node| {
            let callback = callback.clone();
            node.on_radio_change(move |value| {
                let callback = callback.clone();
                queue_guarded_ui_callback("radio error: on_change callback panicked", move || {
                    (callback.as_ref())(value)
                });
            });
            Ok(())
        })
    }
}
