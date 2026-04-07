use std::rc::Rc;

use crate::ohos_arkui_binding::component::built_in_component::Toggle;

use super::super::core::{queue_guarded_ui_callback, ComponentElement};

pub type ToggleElement = ComponentElement<Toggle>;

pub fn toggle_component() -> ToggleElement {
    ComponentElement::new(Toggle::new)
}

pub fn toggle() -> ToggleElement {
    toggle_component()
}

impl ComponentElement<Toggle> {
    pub fn on_change(self, callback: impl Fn(bool) + 'static) -> Self {
        let callback = Rc::new(callback);
        self.with(move |node| {
            let callback = callback.clone();
            node.on_toggle_change(move |value| {
                let callback = callback.clone();
                queue_guarded_ui_callback("toggle error: on_change callback panicked", move || {
                    (callback.as_ref())(value)
                });
            });
            Ok(())
        })
    }
}
