//! Slider — shadcn-style slider.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Wraps the native ArkUI `Slider` element with the theme's `primary`
//! block color and `input`-colored track, surfaced through an input-surface
//! shell (border, `md` radius, `background` fill).

use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Slider`].
#[derive(Props, Clone, PartialEq)]
pub struct SliderProps {
    pub value: f32,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub on_change: Option<EventHandler<f32>>,
}

/// A native slider.
#[component]
pub fn Slider(props: SliderProps) -> Element {
    let theme = use_theme();
    let min = props.min.unwrap_or(0.0);
    let max = props.max.unwrap_or(100.0);
    let on_change = props.on_change;

    rsx! {
        slider {
            slider_value: props.value,
            slider_min: min,
            slider_max: max,
            border_width: 1.0,
            border_color: theme.colors.input,
            border_radius: theme.radii.md,
            background_color: theme.colors.background,
            on_change: move |evt| {
                if let Some(handler) = on_change {
                    handler.call(evt.data().float_value);
                }
            },
        }
    }
}
