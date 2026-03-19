use crate::ohos_arkui_binding::common::error::ArkUIResult;
use crate::ohos_arkui_binding::common::node::ArkUINode;
use crate::view::Element;

pub(crate) fn build_element(element: Element) -> ArkUIResult<ArkUINode> {
    element.build()
}
