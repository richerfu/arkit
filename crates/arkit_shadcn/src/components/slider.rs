use super::*;

pub fn slider(value: f32, min: f32, max: f32) -> SliderElement {
    input_surface(arkit::slider(value, min, max))
}
