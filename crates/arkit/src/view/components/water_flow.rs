use crate::ohos_arkui_binding::component::built_in_component::WaterFlow;

use super::super::core::ComponentElement;

pub type WaterFlowElement = ComponentElement<WaterFlow>;

pub fn water_flow_component() -> WaterFlowElement {
    ComponentElement::new(WaterFlow::new)
}

pub fn water_flow() -> WaterFlowElement {
    water_flow_component()
}
