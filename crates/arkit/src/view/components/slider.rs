use crate::ohos_arkui_binding::component::built_in_component::Slider;

use super::super::core::ComponentElement;

pub type SliderElement = ComponentElement<Slider>;

pub fn slider_component() -> SliderElement {
    ComponentElement::new(Slider::new)
}

pub fn slider() -> SliderElement {
    slider_component()
}
