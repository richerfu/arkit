//! Style injection for headless composites that already accept a style struct.

use arkit_component::components::{
    Calendar as HeadlessCalendar, CalendarProps, Carousel as HeadlessCarousel, CarouselProps,
    CarouselStyle, InputOtp as HeadlessOtp, InputOtpProps, InputOtpStyle,
    MultiSlider as HeadlessMulti, MultiSliderProps, RangeSlider as HeadlessRange, RangeSliderProps,
    Slider as HeadlessSlider, SliderProps, SliderStyle, Sonner as HeadlessSonner, SonnerProps,
    SonnerStyle, Toast as HeadlessToast, ToastProps, ToastStyle,
};
use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

use super::paint::forward;

#[component]
pub fn Calendar(mut props: CalendarProps) -> Element {
    let theme = use_theme();
    if props.selection_color.is_none() {
        props.selection_color = Some(theme.colors.primary);
    }
    if props.today_color.is_none() {
        props.today_color = Some(theme.colors.primary);
    }
    forward(HeadlessCalendar, props, "Calendar")
}

fn slider_style() -> SliderStyle {
    let theme = use_theme();
    SliderStyle {
        track_thickness: spec::PROGRESS_H,
        thumb_size: 16.0,
        thumb_color: Some(theme.colors.background),
        thumb_border_color: Some(theme.colors.primary),
        thumb_border_width: 2.0,
        track_color: Some(theme.colors.secondary),
        selected_color: Some(theme.colors.primary),
        ..SliderStyle::default()
    }
}

#[component]
pub fn Slider(mut props: SliderProps) -> Element {
    if props.style == SliderStyle::default() {
        props.style = slider_style();
    }
    forward(HeadlessSlider, props, "Slider")
}

#[component]
pub fn RangeSlider(mut props: RangeSliderProps) -> Element {
    if props.style == SliderStyle::default() {
        props.style = slider_style();
    }
    forward(HeadlessRange, props, "RangeSlider")
}

#[component]
pub fn MultiSlider(mut props: MultiSliderProps) -> Element {
    if props.style == SliderStyle::default() {
        props.style = slider_style();
    }
    forward(HeadlessMulti, props, "MultiSlider")
}

#[component]
pub fn Carousel(mut props: CarouselProps) -> Element {
    if props.style == CarouselStyle::default() {
        let theme = use_theme();
        props.style = CarouselStyle {
            viewport_radius: Some(spec::RADIUS_XL),
            viewport_shadow: true,
            navigation_background: Some(theme.colors.primary),
            navigation_foreground: Some(theme.colors.primary_foreground),
            indicator_active_color: Some(theme.colors.primary),
            ..CarouselStyle::default()
        };
    }
    forward(HeadlessCarousel, props, "Carousel")
}

#[component]
pub fn Toast(mut props: ToastProps) -> Element {
    if props.style == ToastStyle::default() {
        let theme = use_theme();
        props.style = ToastStyle {
            background_color: Some(theme.colors.card),
            foreground_color: Some(theme.colors.foreground),
            description_color: Some(theme.colors.muted_foreground),
            border_color: Some(theme.colors.border),
            border_radius: Some(spec::RADIUS_LG),
            shadow: Some(true),
            ..ToastStyle::default()
        };
    }
    forward(HeadlessToast, props, "Toast")
}

#[component]
pub fn Sonner(mut props: SonnerProps) -> Element {
    if props.style.toast == ToastStyle::default() {
        let theme = use_theme();
        props.style = SonnerStyle {
            toast: ToastStyle {
                background_color: Some(theme.colors.card),
                foreground_color: Some(theme.colors.foreground),
                description_color: Some(theme.colors.muted_foreground),
                border_color: Some(theme.colors.border),
                border_radius: Some(spec::RADIUS_LG),
                shadow: Some(true),
                ..ToastStyle::default()
            },
            ..props.style
        };
    }
    forward(HeadlessSonner, props, "Sonner")
}

#[component]
pub fn InputOtp(mut props: InputOtpProps) -> Element {
    if props.style == InputOtpStyle::default() {
        let theme = use_theme();
        props.style = InputOtpStyle {
            cell_size: spec::BTN_HEIGHT,
            border_radius: Some(spec::RADIUS_MD),
            background_color: Some(theme.colors.background),
            foreground_color: Some(theme.colors.foreground),
            border_color: Some(theme.colors.input),
            active_border_color: Some(theme.colors.ring),
            caret_color: Some(theme.colors.primary),
            ..InputOtpStyle::default()
        };
    }
    forward(HeadlessOtp, props, "InputOtp")
}
