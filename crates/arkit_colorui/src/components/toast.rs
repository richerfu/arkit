//! Toast / Sonner — white card + ColorUI semantic fills (no official toast).

use arkit_component::components::{
    Sonner as HeadlessSonner, SonnerProps, SonnerStyle, Toast as HeadlessToast, ToastProps,
    ToastStyle,
};
use arkit_prelude::*;

use crate::spec;

fn colorui_toast_style() -> ToastStyle {
    ToastStyle {
        background_color: Some(spec::BG_WHITE),
        foreground_color: Some(spec::TEXT),
        description_color: Some(spec::TEXT_MUTED),
        border_color: Some(spec::FORM_LINE),
        icon_color: Some(spec::BG_GREEN),
        action_background_color: Some(spec::BG_GREEN),
        action_foreground_color: Some(spec::INK_ON_FILL),
        border_radius: Some(spec::RADIUS_CARD),
        shadow: Some(true),
        ..ToastStyle::default()
    }
}

#[component]
pub fn Toast(mut props: ToastProps) -> Element {
    if props.style == ToastStyle::default() {
        props.style = colorui_toast_style();
    }
    super::paint::forward(HeadlessToast, props, "Toast")
}

#[component]
pub fn Sonner(mut props: SonnerProps) -> Element {
    if props.style.toast == ToastStyle::default() {
        props.style = SonnerStyle {
            toast: colorui_toast_style(),
            ..props.style
        };
    }
    super::paint::forward(HeadlessSonner, props, "Sonner")
}
