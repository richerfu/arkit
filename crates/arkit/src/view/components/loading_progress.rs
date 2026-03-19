use crate::ohos_arkui_binding::component::built_in_component::LoadingProgress;

use super::super::core::ComponentElement;

pub type LoadingProgressElement = ComponentElement<LoadingProgress>;

pub fn loading_progress_component() -> LoadingProgressElement {
    ComponentElement::new(LoadingProgress::new)
}

pub fn loading_progress() -> LoadingProgressElement {
    loading_progress_component()
}
