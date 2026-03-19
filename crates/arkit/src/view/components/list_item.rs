use crate::ohos_arkui_binding::component::built_in_component::ListItem;

use super::super::core::ComponentElement;

pub type ListItemElement = ComponentElement<ListItem>;

pub fn list_item_component() -> ListItemElement {
    ComponentElement::new(ListItem::new)
}

pub fn list_item() -> ListItemElement {
    list_item_component()
}
