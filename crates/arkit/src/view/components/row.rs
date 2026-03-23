use crate::logging;
use crate::ohos_arkui_binding::component::built_in_component::Row;
use crate::ohos_arkui_binding::types::advanced::VerticalAlignment;

use super::super::core::ComponentElement;
use super::super::element::Element;

pub type RowElement = ComponentElement<Row>;

pub fn row_component() -> RowElement {
    ComponentElement::new(Row::new)
}

pub fn row(children: Vec<Element>) -> Element {
    row_component().percent_width(1.0).children(children).into()
}

impl ComponentElement<Row> {
    pub fn align_items(self, value: VerticalAlignment) -> Self {
        self.with(move |node| {
            node.set_row_align_items(value as i32).map_err(|error| {
                logging::error(format!(
                    "row error: failed to set align items {value:?}: {error}"
                ));
                error
            })
        })
    }

    pub fn align_items_top(self) -> Self {
        self.align_items(VerticalAlignment::Top)
    }

    pub fn align_items_center(self) -> Self {
        self.align_items(VerticalAlignment::Center)
    }

    pub fn align_items_bottom(self) -> Self {
        self.align_items(VerticalAlignment::Bottom)
    }
}
