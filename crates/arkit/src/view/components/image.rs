use crate::ohos_arkui_binding::component::built_in_component::Image;

use super::super::core::ComponentElement;

pub type ImageElement = ComponentElement<Image>;

pub fn image_component() -> ImageElement {
    ComponentElement::new(Image::new)
}

pub fn image() -> ImageElement {
    image_component()
}
