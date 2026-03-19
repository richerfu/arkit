use crate::ohos_arkui_binding::component::built_in_component::XComponentTexture;

use super::super::core::ComponentElement;

pub type XComponentTextureElement = ComponentElement<XComponentTexture>;

pub fn xcomponent_texture_component() -> XComponentTextureElement {
    ComponentElement::new(XComponentTexture::new)
}

pub fn xcomponent_texture() -> XComponentTextureElement {
    xcomponent_texture_component()
}
