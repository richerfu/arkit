use crate::ohos_arkui_binding::component::built_in_component::Span;

use super::super::core::ComponentElement;

pub type SpanElement = ComponentElement<Span>;

pub fn span_component() -> SpanElement {
    ComponentElement::new(Span::new)
}

pub fn span() -> SpanElement {
    span_component()
}
