use crate::ohos_arkui_binding::component::built_in_component::Column;

use super::super::core::ComponentElement;
use super::super::element::Element;

pub type ColumnElement = ComponentElement<Column>;

pub fn column_component() -> ColumnElement {
    ComponentElement::new(Column::new)
}

pub fn column(children: Vec<Element>) -> Element {
    column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(children)
        .into()
}
