use super::*;

pub fn aspect_ratio(ratio: f32, child: Element) -> Element {
    arkit::stack_component()
        .aspect_ratio(ratio)
        .children(vec![child])
        .into()
}
