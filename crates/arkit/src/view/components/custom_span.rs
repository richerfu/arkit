use crate::ohos_arkui_binding::component::built_in_component::CustomSpan;

use super::super::core::ComponentElement;

pub type CustomSpanElement = ComponentElement<CustomSpan>;

pub fn custom_span_component() -> CustomSpanElement {
    ComponentElement::new(CustomSpan::new)
}

pub fn custom_span() -> CustomSpanElement {
    custom_span_component()
}
