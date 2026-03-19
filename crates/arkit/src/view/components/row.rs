use crate::ohos_arkui_binding::component::built_in_component::Row;

use super::super::core::ComponentElement;
use super::super::element::Element;

pub type RowElement = ComponentElement<Row>;

pub fn row_component() -> RowElement {
    ComponentElement::new(Row::new)
}

pub fn row(children: Vec<Element>) -> Element {
    row_component().percent_width(1.0).children(children).into()
}
