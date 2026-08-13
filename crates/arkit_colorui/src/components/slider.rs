//! Slider — track/thumb mapped to `.cu-progress` + switch knob.

use arkit_component::components::{
    MultiSlider as HeadlessMulti, MultiSliderProps, RangeSlider as HeadlessRange, RangeSliderProps,
    Slider as HeadlessSlider, SliderProps, SliderStyle,
};
use arkit_prelude::*;

use crate::spec;
use crate::theme::{swatch, use_colorui_theme};

fn colorui_slider_style() -> SliderStyle {
    let fill = swatch(use_colorui_theme().primary).fill;
    SliderStyle {
        track_thickness: spec::PROGRESS_HEIGHT,
        thumb_size: spec::SWITCH_H,
        thumb_color: Some(spec::BG_WHITE),
        thumb_border_color: Some(fill),
        thumb_border_width: 0.0,
        track_color: Some(spec::PROGRESS_TRACK),
        selected_color: Some(fill),
        ..SliderStyle::default()
    }
}

#[component]
pub fn Slider(mut props: SliderProps) -> Element {
    if props.style == SliderStyle::default() {
        props.style = colorui_slider_style();
    }
    super::paint::forward(HeadlessSlider, props, "Slider")
}

#[component]
pub fn RangeSlider(mut props: RangeSliderProps) -> Element {
    if props.style == SliderStyle::default() {
        props.style = colorui_slider_style();
    }
    super::paint::forward(HeadlessRange, props, "RangeSlider")
}

#[component]
pub fn MultiSlider(mut props: MultiSliderProps) -> Element {
    if props.style == SliderStyle::default() {
        props.style = colorui_slider_style();
    }
    super::paint::forward(HeadlessMulti, props, "MultiSlider")
}
