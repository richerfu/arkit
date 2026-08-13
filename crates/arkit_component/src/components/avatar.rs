//! Avatar — unstyled user avatar.

use crate::appearance::{AvatarAppearance, AvatarStyleInput};
use crate::style::use_style_kit;
use arkit_prelude::*;

/// Props for [`Avatar`].
#[derive(Props, Clone, PartialEq)]
pub struct AvatarProps {
    #[props(default)]
    pub src: Option<String>,
    #[props(default)]
    pub fallback: Option<Element>,
    #[props(default)]
    pub ring: Option<bool>,
    #[props(default)]
    pub radius: Option<f32>,
    #[props(default)]
    pub appearance: Option<AvatarAppearance>,
}

/// A user avatar with image, fallback initials, optional ring and radius.
#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let kit = use_style_kit();
    let appearance = props.appearance.unwrap_or_else(|| {
        kit.avatar(&AvatarStyleInput {
            ring: props.ring.unwrap_or(false),
            radius: props.radius,
        })
    });

    rsx! {
        stack {
            width: appearance.size,
            height: appearance.size,
            border_radius: appearance.radius,
            border_width: appearance.border_width,
            border_color: appearance.border_color,
            alignment: "center",
            clip: true,
            if let Some(fallback) = props.fallback {
                {fallback}
            }
            if let Some(src) = props.src.as_ref() {
                image {
                    src: src.clone(),
                    width: appearance.size,
                    height: appearance.size,
                    border_radius: appearance.radius,
                    object_fit: "cover",
                    clip: true,
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AvatarFallbackProps {
    pub content: String,
    #[props(default)]
    pub appearance: Option<AvatarAppearance>,
}

/// Default initials fallback surface for [`Avatar`].
#[component]
pub fn AvatarFallback(props: AvatarFallbackProps) -> Element {
    let kit = use_style_kit();
    let appearance = props.appearance.unwrap_or_else(|| {
        kit.avatar(&AvatarStyleInput {
            ring: false,
            radius: None,
        })
    });
    let content = props.content;

    rsx! {
        stack {
            width: appearance.size,
            height: appearance.size,
            background_color: appearance.fallback_background,
            alignment: "center",
            text {
                content,
                font_size: appearance.fallback_font_size,
                font_color: appearance.fallback_foreground,
                line_height: 20.0,
                text_align: "start",
            }
        }
    }
}
