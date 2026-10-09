//! Content-sized panel scrolling constrained by the application viewport.

use crate::theme::spacing;
use arkit_prelude::*;

pub(crate) fn bounded_panel_width(viewport: arkit_hooks::OverlayViewport, requested: f32) -> f32 {
    let requested = if requested.is_finite() && requested > 0.0 {
        requested
    } else {
        288.0
    };
    if !viewport.frame.is_measured() {
        return requested;
    }
    let available = viewport.frame.width / viewport.scale.max(f32::EPSILON)
        - viewport.safe_area.left
        - viewport.safe_area.right
        - spacing::SM * 2.0;
    requested.min(available.max(1.0))
}

pub(crate) fn panel_available_height(viewport: arkit_hooks::OverlayViewport, top: f32) -> f32 {
    (viewport.frame.height / viewport.scale.max(f32::EPSILON)
        - viewport.safe_area.bottom
        - spacing::SM
        - top)
        .max(1.0)
}

#[component]
pub(crate) fn PanelViewport(
    max_height: f32,
    #[props(default = 132.0)] estimated_height: f32,
    #[props(default)] center: bool,
    children: Element,
) -> Element {
    let reference = arkit_hooks::use_native_element_ref();
    let scale = arkit_hooks::use_window_metrics().scale.max(f32::EPSILON);
    let content_height = use_signal(|| estimated_height.max(1.0));
    arkit_hooks::use_layout_size(reference.clone(), move |size| {
        let mut content_height = content_height;
        let next = size.height / scale;
        if next.is_finite() && next > 0.0 && (next - *content_height.peek()).abs() > 0.5 {
            content_height.set(next);
        }
    });
    let height = content_height().min(max_height.max(1.0));
    rsx! {
        scroll {
            width: "100%",
            height,
            scroll_bar: "auto",
            scroll_enabled: content_height() > height + 0.5,
            column {
                native_ref: reference,
                width: "100%",
                align_items: if center { "center" } else { "start" },
                {children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oversized_panels_use_actual_window_bounds_including_safe_edges() {
        let viewport = arkit_hooks::OverlayViewport {
            frame: arkit_arkui::LayoutFramePx {
                width: 640.0,
                height: 480.0,
                ..Default::default()
            },
            scale: 2.0,
            safe_area: arkit_hooks::EdgeInsets {
                left: 12.0,
                right: 12.0,
                bottom: 20.0,
                ..Default::default()
            },
        };
        assert_eq!(bounded_panel_width(viewport, 520.0), 280.0);
        assert_eq!(panel_available_height(viewport, 80.0), 132.0);
    }
}
