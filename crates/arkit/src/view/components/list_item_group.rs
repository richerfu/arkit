use crate::ohos_arkui_binding::component::built_in_component::ListItemGroup;

use super::super::core::ComponentElement;

pub type ListItemGroupElement = ComponentElement<ListItemGroup>;

pub fn list_item_group_component() -> ListItemGroupElement {
    ComponentElement::new(ListItemGroup::new)
}

pub fn list_item_group() -> ListItemGroupElement {
    list_item_group_component()
}
