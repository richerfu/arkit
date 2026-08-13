//! Separator — unstyled divider.

use crate::appearance::{SeparatorAppearance, SeparatorStyleInput};
use crate::style::use_style_kit;
use arkit_prelude::*;

/// Props for [`Separator`].
#[derive(Props, Clone, PartialEq)]
pub struct SeparatorProps {
    /// When `Some`, renders a vertical separator of the given height. When
    /// `None`, renders a horizontal full-width separator.
    pub vertical_height: Option<f32>,
    #[props(default)]
    pub appearance: Option<SeparatorAppearance>,
}

/// A horizontal or vertical divider.
#[component]
pub fn Separator(props: SeparatorProps) -> Element {
    let kit = use_style_kit();
    let appearance = props.appearance.unwrap_or_else(|| {
        kit.separator(&SeparatorStyleInput {
            vertical: props.vertical_height.is_some(),
        })
    });
    match props.vertical_height {
        Some(height) => rsx! {
            column {
                width: appearance.thickness,
                height: height,
                background_color: appearance.color,
            }
        },
        None => rsx! {
            row {
                height: appearance.thickness,
                width: "100%",
                background_color: appearance.color,
            }
        },
    }
}
