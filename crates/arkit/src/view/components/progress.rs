use crate::ohos_arkui_binding::component::built_in_component::Progress;

use super::super::core::ComponentElement;

pub type ProgressElement = ComponentElement<Progress>;

pub fn progress_component() -> ProgressElement {
    ComponentElement::new(Progress::new)
}

pub fn progress() -> ProgressElement {
    progress_component()
}
