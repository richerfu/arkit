use crate::ohos_arkui_binding::component::built_in_component::ImageSpan;

use super::super::core::ComponentElement;

pub type ImageSpanElement = ComponentElement<ImageSpan>;

pub fn image_span_component() -> ImageSpanElement {
    ComponentElement::new(ImageSpan::new)
}

pub fn image_span() -> ImageSpanElement {
    image_span_component()
}
