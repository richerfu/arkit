use crate::logging;
use crate::ohos_arkui_binding::component::built_in_component::Column;
use crate::ohos_arkui_binding::types::advanced::HorizontalAlignment;

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

impl ComponentElement<Column> {
    pub fn align_items(self, value: HorizontalAlignment) -> Self {
        self.with(move |node| {
            node.set_column_align_items(value as i32).map_err(|error| {
                logging::error(format!(
                    "column error: failed to set align items {value:?}: {error}"
                ));
                error
            })
        })
    }

    pub fn align_items_start(self) -> Self {
        self.align_items(HorizontalAlignment::Start)
    }

    pub fn align_items_center(self) -> Self {
        self.align_items(HorizontalAlignment::Center)
    }

    pub fn align_items_end(self) -> Self {
        self.align_items(HorizontalAlignment::End)
    }
}
