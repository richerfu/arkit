use crate::ohos_arkui_binding::component::built_in_component::FlowItem;

use super::super::core::ComponentElement;

pub type FlowItemElement = ComponentElement<FlowItem>;

pub fn flow_item_component() -> FlowItemElement {
    ComponentElement::new(FlowItem::new)
}

pub fn flow_item() -> FlowItemElement {
    flow_item_component()
}
