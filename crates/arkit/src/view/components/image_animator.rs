use crate::ohos_arkui_binding::component::built_in_component::ImageAnimator;

use super::super::core::ComponentElement;

pub type ImageAnimatorElement = ComponentElement<ImageAnimator>;

pub fn image_animator_component() -> ImageAnimatorElement {
    ComponentElement::new(ImageAnimator::new)
}

pub fn image_animator() -> ImageAnimatorElement {
    image_animator_component()
}
