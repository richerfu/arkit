//! Shadcn popup elevation, independent of ArkUI's platform shadow theme.
//!
//! Tailwind's md/lg recipes each have two layers with negative spread. Native
//! custom shadows have no spread parameter, so inset underlays provide the
//! corresponding smaller rounded rectangle beneath the opaque panel.

use arkit_prelude::*;
use dioxus_core_macro::component;

#[path = "popup_shadow_recipe.rs"]
mod recipe;
pub(crate) use recipe::PopupShadowKind;

#[component]
pub(crate) fn PopupShadow(
    width: f32,
    radius: f32,
    background: u32,
    kind: PopupShadowKind,
    #[props(default)] position: Option<String>,
    children: Element,
) -> Element {
    let reference = arkit_hooks::use_native_element_ref();
    let scale = arkit_hooks::use_window_metrics().scale.max(f32::EPSILON);
    let height = use_signal(|| 0.0_f32);
    arkit_hooks::use_layout_size(reference.clone(), move |size| {
        let next = size.height / scale;
        if next.is_finite() && next > 0.0 && (next - *height.peek()).abs() > 0.5 {
            let mut height = height;
            height.set(next);
        }
    });

    rsx! {
        stack {
            width,
            position: if let Some(position) = position { position },
            alignment: "top-start",
            clip: false,
            for layer in kind.layers().into_iter().rev() {
                if let Some(bounds) = layer.bounds(width, height(), radius) {
                    row {
                        width: bounds.width,
                        height: bounds.height,
                        position: format!("{},{}", layer.inset, layer.inset),
                        border_radius: bounds.radius,
                        background_color: background,
                        custom_shadow: layer.native_value(scale),
                        clip: false,
                        hit_test_behavior: "none",
                        accessibility_mode: "disabled",
                    }
                }
            }
            column {
                native_ref: reference,
                width: "100%",
                clip: false,
                {children}
            }
        }
    }
}
