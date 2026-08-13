//! Carousel — ColorUI `square-dot` / 10upx card radius.

use arkit_component::components::{Carousel as HeadlessCarousel, CarouselProps, CarouselStyle};
use arkit_prelude::*;

use crate::spec;
use crate::theme::{swatch, use_colorui_theme};

pub(crate) fn colorui_carousel_style() -> CarouselStyle {
    let fill = swatch(use_colorui_theme().primary).fill;
    CarouselStyle {
        viewport_radius: Some(spec::RADIUS_CARD),
        viewport_shadow: false,
        viewport_border_width: 0.0,
        navigation_background: Some(fill),
        navigation_foreground: Some(spec::INK_ON_FILL),
        indicator_active_color: Some(fill),
        indicator_inactive_color: Some(0x66FFFFFF),
        indicator_size: spec::SWIPER_DOT,
        active_indicator_width: spec::SWIPER_DOT_ACTIVE,
        ..CarouselStyle::default()
    }
}

#[component]
pub fn Carousel(mut props: CarouselProps) -> Element {
    if props.style == CarouselStyle::default() {
        props.style = colorui_carousel_style();
    }
    super::paint::forward(HeadlessCarousel, props, "Carousel")
}
