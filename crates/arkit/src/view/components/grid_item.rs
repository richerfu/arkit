use crate::ohos_arkui_binding::component::built_in_component::GridItem;

use super::super::core::ComponentElement;

pub type GridItemElement = ComponentElement<GridItem>;

pub fn grid_item_component() -> GridItemElement {
    ComponentElement::new(GridItem::new)
}

pub fn grid_item() -> GridItemElement {
    grid_item_component()
}
