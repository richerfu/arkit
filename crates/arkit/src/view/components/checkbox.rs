use std::rc::Rc;

use crate::ohos_arkui_binding::component::built_in_component::Checkbox;

use super::super::core::{queue_guarded_ui_callback, ComponentElement};

pub type CheckboxElement = ComponentElement<Checkbox>;

pub fn checkbox_component() -> CheckboxElement {
    ComponentElement::new(Checkbox::new)
}

pub fn checkbox() -> CheckboxElement {
    checkbox_component()
}

impl ComponentElement<Checkbox> {
    pub fn on_change(self, callback: impl Fn(bool) + 'static) -> Self {
        let callback = Rc::new(callback);
        self.with(move |node| {
            let callback = callback.clone();
            node.on_checkbox_change(move |value| {
                let callback = callback.clone();
                queue_guarded_ui_callback(
                    "checkbox error: on_change callback panicked",
                    move || (callback.as_ref())(value),
                );
            });
            Ok(())
        })
    }
}
