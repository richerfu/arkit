use crate::ohos_arkui_binding::component::built_in_component::Scroll;

use super::super::core::ComponentElement;

pub type ScrollElement = ComponentElement<Scroll>;

pub fn scroll_component() -> ScrollElement {
    ComponentElement::new(Scroll::new)
}

pub fn scroll() -> ScrollElement {
    scroll_component()
}
