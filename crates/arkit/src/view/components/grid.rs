use crate::ohos_arkui_binding::component::built_in_component::Grid;

use super::super::core::ComponentElement;

pub type GridElement = ComponentElement<Grid>;

pub fn grid_component() -> GridElement {
    ComponentElement::new(Grid::new)
}

pub fn grid() -> GridElement {
    grid_component()
}
