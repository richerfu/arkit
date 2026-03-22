use super::*;

pub fn aspect_ratio(ratio: f32, child: Element) -> Element {
    arkit::stack_component()
        .style(ArkUINodeAttributeType::AspectRatio, ratio)
        .children(vec![child])
        .into()
}
