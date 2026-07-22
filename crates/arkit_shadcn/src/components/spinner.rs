//! Spinner — a theme-aware indeterminate loading indicator.
//!
//! The component is backed by ArkUI's native `LoadingProgress`, so animation
//! runs on the platform rather than scheduling Rust-side frame updates. A
//! custom Lucide icon can be supplied when a product-specific loading glyph is
//! needed; only that override uses Arkit's sampled rotation animation.

use std::cell::Cell;

use crate::theme::*;
use arkit_animation::{
    Angle, Animation, AnimationSelector, Composition, Easing, ExecutionPolicy, IterationCount,
    Modifier, TargetName, TimeSpan, Timeline, TimelinePosition, ROTATION,
};
use arkit_prelude::*;

const CUSTOM_ICON_SPIN_DURATION_MS: u64 = 900;

thread_local! {
    static NEXT_SPINNER_TARGET: Cell<u64> = const { Cell::new(0) };
}

/// Props for [`Spinner`].
#[derive(Props, Clone, PartialEq)]
pub struct SpinnerProps {
    /// Width and height in vp. Matches shadcn's `size-4` default.
    #[props(default = 16.0)]
    pub size: f32,
    /// Indicator color. Defaults to the active theme's foreground color.
    #[props(default)]
    pub color: Option<u32>,
    /// Optional Lucide icon name. The default uses native `LoadingProgress`.
    #[props(default)]
    pub icon: Option<String>,
    /// Stroke width used by a custom Lucide icon.
    #[props(default = 2.0)]
    pub stroke_width: f32,
    /// Pauses the loading animation while preserving layout.
    #[props(default = true)]
    pub spinning: bool,
}

/// A compact, continuously animated loading indicator.
#[component]
pub fn Spinner(props: SpinnerProps) -> Element {
    let theme = use_theme();
    let size = props.size.max(1.0);
    let color = props.color.unwrap_or(theme.colors.foreground);

    if let Some(icon) = props.icon {
        return rsx! {
            CustomSpinnerIcon {
                icon,
                size,
                color,
                stroke_width: props.stroke_width.max(0.1),
                spinning: props.spinning,
            }
        };
    }

    rsx! {
        loadingprogress {
            width: size,
            height: size,
            loading_progress_color: color,
            loading_progress_enable_loading: props.spinning,
            hit_test_behavior: "transparent",
        }
    }
}

#[component]
fn CustomSpinnerIcon(
    icon: String,
    size: f32,
    color: u32,
    stroke_width: f32,
    spinning: bool,
) -> Element {
    let target_name = use_hook(next_spinner_target_name);
    let target = arkit_animation::use_animation_target(target_name.clone());
    let controls = arkit_animation::use_animation(custom_icon_timeline(&target_name));
    let animation = controls.clone();

    use_effect(move || {
        if !target.is_ready() || !animation.is_ready() {
            return;
        }
        if spinning {
            animation.play();
        } else {
            animation.pause();
        }
    });

    rsx! {
        row {
            width: size,
            height: size,
            align_items: "center",
            justify_content: "center",
            hit_test_behavior: "transparent",
            {arkit_icon::icon_with_stroke(icon, size, color, stroke_width)}
        }
    }
}

fn next_spinner_target_name() -> String {
    NEXT_SPINNER_TARGET.with(|next| {
        let id = next.get();
        next.set(
            id.checked_add(1)
                .expect("spinner target id space exhausted"),
        );
        format!("arkit-shadcn-spinner-{id}")
    })
}

fn custom_icon_timeline(target_name: &str) -> Timeline {
    let rotation = Animation::new(AnimationSelector::Target(TargetName::owned(target_name)))
        .tween(
            &ROTATION,
            Angle::degrees(0.0),
            Angle::degrees(360.0),
            TimeSpan::from_millis(CUSTOM_ICON_SPIN_DURATION_MS),
        )
        .configure_last(
            Easing::Linear,
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        );

    Timeline::new()
        .add(rotation, TimelinePosition::START)
        .iterations(IterationCount::Infinite)
        .execution_policy(ExecutionPolicy::SampledOnly)
}

#[cfg(test)]
mod tests {
    use super::SpinnerProps;

    #[test]
    fn props_support_custom_icon_style_and_pause() {
        let props = SpinnerProps {
            size: 24.0,
            color: Some(0xFF00_7DFF),
            icon: Some("refresh-cw".to_string()),
            stroke_width: 1.5,
            spinning: false,
        };

        assert_eq!(props.size, 24.0);
        assert_eq!(props.color, Some(0xFF00_7DFF));
        assert_eq!(props.icon.as_deref(), Some("refresh-cw"));
        assert_eq!(props.stroke_width, 1.5);
        assert!(!props.spinning);
    }
}
