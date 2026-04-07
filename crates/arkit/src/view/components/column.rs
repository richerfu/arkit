use crate::ohos_arkui_binding::component::built_in_component::Column;
use crate::ohos_arkui_binding::types::advanced::HorizontalAlignment;
use crate::prelude::ArkUINodeAttributeType;
use crate::{Horizontal, Vertical};

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
    pub fn align_x(self, value: Horizontal) -> Self {
        match value {
            Horizontal::Left => self.align_items_start(),
            Horizontal::Center => self.align_items_center(),
            Horizontal::Right => self.align_items_end(),
        }
    }

    pub fn align_y(self, value: Vertical) -> Self {
        let justify = match value {
            Vertical::Top => 1_i32,
            Vertical::Center => 2_i32,
            Vertical::Bottom => 3_i32,
        };
        self.style(ArkUINodeAttributeType::ColumnJustifyContent, justify)
    }

    pub fn align_items(self, value: HorizontalAlignment) -> Self {
        self.style(ArkUINodeAttributeType::ColumnAlignItems, value as i32)
            .patch_attr(ArkUINodeAttributeType::ColumnAlignItems, value as i32)
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
