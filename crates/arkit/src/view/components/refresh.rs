use crate::ohos_arkui_binding::component::built_in_component::Refresh;

use super::super::core::ComponentElement;

pub type RefreshElement = ComponentElement<Refresh>;

pub fn refresh_component() -> RefreshElement {
    ComponentElement::new(Refresh::new)
}

pub fn refresh() -> RefreshElement {
    refresh_component()
}
