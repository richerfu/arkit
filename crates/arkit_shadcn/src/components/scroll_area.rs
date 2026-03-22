use super::*;

pub fn scroll_area(children: Vec<Element>) -> ScrollElement {
    panel_surface(
        arkit::scroll_component()
            .percent_width(1.0)
            .children(children),
    )
}
