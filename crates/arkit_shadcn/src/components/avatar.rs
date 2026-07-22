//! Avatar — shadcn-style user avatar.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. The root owns size, clipping, radius, ring and optional image
//! overlay. Fallback content is an explicit slot; Avatar does not synthesize a
//! default fallback when business code omits one.

use crate::theme::*;
use arkit_prelude::*;

const AVATAR_SIZE: f32 = 32.0;
const AVATAR_RING_WIDTH: f32 = 2.0;

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
}

/// A user avatar with image, fallback initials, optional ring and radius.
#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let theme = use_theme();
    let radius = props.radius.unwrap_or(theme.radii.full);
    let ring = props.ring.unwrap_or(false);
    let border_width = if ring { AVATAR_RING_WIDTH } else { 0.0 };
    let border_color = if ring {
        theme.colors.background
    } else {
        0x00000000
    };

    rsx! {
        stack {
            width: AVATAR_SIZE,
            height: AVATAR_SIZE,
            border_radius: radius,
            border_width,
            border_color,
            alignment: "center",
            clip: true,
            if let Some(fallback) = props.fallback {
                {fallback}
            }
            if let Some(src) = props.src.as_ref() {
                image {
                    src: src.clone(),
                    width: AVATAR_SIZE,
                    height: AVATAR_SIZE,
                    border_radius: radius,
                    object_fit: "cover",
                    clip: true,
                }
            }
        }
    }
}

/// Default initials fallback surface for [`Avatar`].
#[component]
pub fn AvatarFallback(content: String) -> Element {
    let theme = use_theme();

    rsx! {
        stack {
            width: AVATAR_SIZE,
            height: AVATAR_SIZE,
            background_color: theme.colors.muted,
            alignment: "center",
            text {
                content,
                font_size: typography::SM,
                font_color: theme.colors.muted_foreground,
                line_height: 20.0,
                text_align: "start",
            }
        }
    }
}
