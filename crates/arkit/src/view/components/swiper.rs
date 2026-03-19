use crate::ohos_arkui_binding::component::built_in_component::Swiper;

use super::super::core::ComponentElement;

pub type SwiperElement = ComponentElement<Swiper>;

pub fn swiper_component() -> SwiperElement {
    ComponentElement::new(Swiper::new)
}

pub fn swiper() -> SwiperElement {
    swiper_component()
}
