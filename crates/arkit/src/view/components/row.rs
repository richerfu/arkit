use crate::ohos_arkui_binding::component::built_in_component::Row;
use crate::ohos_arkui_binding::types::advanced::VerticalAlignment;
use crate::prelude::ArkUINodeAttributeType;
use crate::{Horizontal, Vertical};

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
    pub fn align_y(self, value: Vertical) -> Self {
        match value {
            Vertical::Top => self.align_items_top(),
            Vertical::Center => self.align_items_center(),
            Vertical::Bottom => self.align_items_bottom(),
        }
    }

    pub fn align_x(self, value: Horizontal) -> Self {
        let justify = match value {
            Horizontal::Left => 1_i32,
            Horizontal::Center => 2_i32,
            Horizontal::Right => 3_i32,
        };
        self.style(ArkUINodeAttributeType::RowJustifyContent, justify)
    }

    pub fn align_items(self, value: VerticalAlignment) -> Self {
        self.style(ArkUINodeAttributeType::RowAlignItems, value as i32)
            .patch_attr(ArkUINodeAttributeType::RowAlignItems, value as i32)
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
