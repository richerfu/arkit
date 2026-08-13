//! ColorUI spinner — primary hue instead of shadcn foreground.

use arkit_component::components::Spinner as HeadlessSpinner;
use arkit_prelude::*;

use crate::theme::use_colorui_theme;

#[component]
pub fn Spinner(
    #[props(default = 16.0)] size: f32,
    #[props(default)] color: Option<u32>,
    #[props(default)] icon: Option<String>,
    #[props(default = 2.0)] stroke_width: f32,
    #[props(default = true)] spinning: bool,
) -> Element {
    let theme = use_colorui_theme();
    let color = color.unwrap_or(theme.tokens().colors.primary);
    rsx! {
        HeadlessSpinner {
            size,
            color: Some(color),
            icon,
            stroke_width,
            spinning,
        }
    }
}
