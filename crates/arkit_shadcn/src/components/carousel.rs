use super::*;

fn carousel(slides: Vec<Element>) -> SwiperElement {
    panel_surface(arkit::swiper_component().children(slides))
}
