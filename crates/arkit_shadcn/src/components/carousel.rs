//! Carousel — shadcn-style swiper carousel.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Wraps the native ArkUI `Swiper` in a panel surface (`popover` fill,
//! 1px `border`, `md` radius, `shadow-sm`), matching the legacy
//! `panel_surface(swiper.children(slides))`.

use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Carousel`].
#[derive(Props, Clone, PartialEq)]
pub struct CarouselProps {
    pub children: Element,
}

/// A native swiper carousel on a panel surface.
#[component]
pub fn Carousel(props: CarouselProps) -> Element {
    let theme = use_theme();
    rsx! {
        column {
            background_color: theme.colors.popover,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.md,
            shadow: 1,
            clip: true,
            swiper {
                {props.children}
            }
        }
    }
}
