//! Progress — shadcn-style determinate progress bar.
//!
//! The track and indicator are rendered explicitly instead of delegating to
//! ArkUI's platform-skinned `Progress`. This keeps the native result aligned
//! with shadcn: an 8vp fully rounded `primary/20` track, a solid primary
//! indicator, and a short `transition-all`-style value animation.

use arkit_animation::{
    use_animatable_with_defaults, use_animation_snapshot, AnimatableDefaults, Easing, TimeSpan,
};
use arkit_prelude::*;

use crate::theme::*;

const DEFAULT_HEIGHT: f32 = 8.0;
const DEFAULT_ANIMATION_DURATION_MS: u64 = 150;

/// Props for [`Progress`].
#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    /// Current progress value. Values outside the range are clamped.
    pub value: f32,
    /// Maximum value. Non-finite and non-positive totals render as empty.
    #[props(default)]
    pub total: Option<f32>,
    /// Track height in vp. Defaults to shadcn's `h-2`.
    #[props(default)]
    pub height: Option<f32>,
    /// Track color. Defaults to the theme's 20%-alpha primary track token.
    #[props(default)]
    pub track_color: Option<u32>,
    /// Filled indicator color. Defaults to the theme primary color.
    #[props(default)]
    pub indicator_color: Option<u32>,
    /// Track corner radius. Defaults to the theme's full radius.
    #[props(default)]
    pub radius: Option<f32>,
    /// Animates controlled value changes.
    #[props(default = true)]
    pub animated: bool,
    /// Value-transition duration. The 150ms default matches shadcn's
    /// `transition-all` utility.
    #[props(default = DEFAULT_ANIMATION_DURATION_MS)]
    pub animation_duration_ms: u64,
}

/// A controlled horizontal progress bar with shadcn styling and animation.
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let theme = use_theme();
    let target = normalized_progress(props.value, props.total.unwrap_or(100.0));
    let height = positive_or(props.height.unwrap_or(DEFAULT_HEIGHT), DEFAULT_HEIGHT);
    let radius = non_negative_or(props.radius.unwrap_or(theme.radii.full), theme.radii.full);
    let track_color = props.track_color.unwrap_or(theme.colors.primary_track);
    let indicator_color = props.indicator_color.unwrap_or(theme.colors.primary);
    let animated = props.animated;
    let duration_ms = props.animation_duration_ms;

    let progress = use_animatable_with_defaults(
        target,
        AnimatableDefaults {
            duration: TimeSpan::from_millis(DEFAULT_ANIMATION_DURATION_MS),
            easing: progress_easing(),
            ..AnimatableDefaults::default()
        },
    );
    let snapshot = use_animation_snapshot(progress.controls());
    let _ = snapshot();
    let animation = progress.clone();

    use_effect(use_reactive(
        (&target, &animated, &duration_ms),
        move |(target, animated, duration_ms)| {
            if !animated || duration_ms == 0 {
                animation.set(target);
                return;
            }
            animation.animate(
                animation.get(),
                target,
                TimeSpan::from_millis(duration_ms),
                TimeSpan::ZERO,
                progress_easing(),
            );
        },
    ));

    let current = progress.get().clamp(0.0, 1.0);

    rsx! {
        row {
            percent_width: 1.0,
            height,
            align_items: "start",
            justify_content: "start",
            background_color: track_color,
            border_radius: radius,
            clip: true,
            hit_test_behavior: 2_i32,
            row {
                percent_width: current,
                percent_height: 1.0,
                background_color: indicator_color,
                hit_test_behavior: 2_i32,
            }
        }
    }
}

fn normalized_progress(value: f32, total: f32) -> f32 {
    if !value.is_finite() || !total.is_finite() || total <= 0.0 {
        return 0.0;
    }
    (value / total).clamp(0.0, 1.0)
}

fn positive_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn non_negative_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        fallback
    }
}

fn progress_easing() -> Easing {
    Easing::cubic_bezier(0.4, 0.0, 0.2, 1.0)
        .expect("progress easing uses a valid static cubic-bezier curve")
}

#[cfg(test)]
mod tests {
    use super::{non_negative_or, normalized_progress, positive_or};

    #[test]
    fn progress_is_normalized_and_clamped() {
        assert_eq!(normalized_progress(50.0, 200.0), 0.25);
        assert_eq!(normalized_progress(-10.0, 100.0), 0.0);
        assert_eq!(normalized_progress(150.0, 100.0), 1.0);
    }

    #[test]
    fn invalid_progress_inputs_render_empty() {
        assert_eq!(normalized_progress(f32::NAN, 100.0), 0.0);
        assert_eq!(normalized_progress(50.0, f32::INFINITY), 0.0);
        assert_eq!(normalized_progress(50.0, 0.0), 0.0);
        assert_eq!(normalized_progress(50.0, -100.0), 0.0);
    }

    #[test]
    fn dimensions_reject_invalid_values() {
        assert_eq!(positive_or(6.0, 8.0), 6.0);
        assert_eq!(positive_or(0.0, 8.0), 8.0);
        assert_eq!(non_negative_or(0.0, 999.0), 0.0);
        assert_eq!(non_negative_or(f32::NAN, 999.0), 999.0);
    }
}
