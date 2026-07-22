//! Carousel — a mobile-first carousel backed by ArkUI's native `Swiper`.
//!
//! Native swiping remains the source of truth for touch interaction. The
//! component adds controlled and uncontrolled selection, large touch targets,
//! optional navigation, configurable indicators, looping, autoplay, and
//! transition configuration around that primitive.

use crate::icon::icon_placeholder;
use crate::theme::*;
use arkit_prelude::*;

use super::ARKUI_BORDER_STYLE_SOLID;

const TRANSPARENT: u32 = 0x00000000;
const MIN_TOUCH_TARGET: f32 = 40.0;

/// Placement of the navigation and indicator row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselControlsPlacement {
    /// Keeps controls outside the content so they never obstruct a swipe.
    #[default]
    Below,
    /// Floats controls over the bottom of the viewport.
    Overlay,
    /// Floats previous and next controls at the vertical center of the viewport.
    OverlayCenter,
}

/// Built-in page indicator presentations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselIndicatorVariant {
    /// Equal-sized dots.
    Dot,
    /// Expands the active dot into a pill.
    #[default]
    Pill,
    /// Compact `current / total` text.
    Fraction,
}

/// Native transition curves exposed without leaking ArkUI integer constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselTransitionCurve {
    Linear,
    Ease,
    EaseInOut,
    #[default]
    Smooth,
    Friction,
}

impl CarouselTransitionCurve {
    const fn arkui_value(self) -> i32 {
        match self {
            Self::Linear => 0,
            Self::Ease => 1,
            Self::EaseInOut => 4,
            Self::Smooth => 11,
            Self::Friction => 12,
        }
    }
}

/// Visual overrides for [`Carousel`]. `None` colors and radii resolve from the
/// active theme, while dimensions have mobile-friendly defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarouselStyle {
    pub viewport_background: Option<u32>,
    pub viewport_border_color: Option<u32>,
    pub viewport_border_width: f32,
    pub viewport_radius: Option<f32>,
    /// Whether the viewport draws the small elevation shadow.
    pub viewport_shadow: bool,
    pub controls_background: Option<u32>,
    pub controls_height: f32,
    pub controls_gap: f32,
    pub navigation_background: Option<u32>,
    pub navigation_foreground: Option<u32>,
    pub navigation_size: f32,
    pub navigation_disabled_opacity: f32,
    pub indicator_active_color: Option<u32>,
    pub indicator_inactive_color: Option<u32>,
    pub indicator_size: f32,
    pub active_indicator_width: f32,
    pub indicator_gap: f32,
}

impl Default for CarouselStyle {
    fn default() -> Self {
        Self {
            viewport_background: None,
            viewport_border_color: None,
            viewport_border_width: 0.0,
            viewport_radius: None,
            viewport_shadow: true,
            controls_background: None,
            controls_height: 48.0,
            controls_gap: spacing::SM,
            navigation_background: None,
            navigation_foreground: None,
            navigation_size: 40.0,
            navigation_disabled_opacity: 0.32,
            indicator_active_color: None,
            indicator_inactive_color: None,
            indicator_size: 7.0,
            active_indicator_width: 18.0,
            indicator_gap: spacing::XXS,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ResolvedCarouselStyle {
    viewport_background: u32,
    viewport_border_color: u32,
    viewport_border_width: f32,
    viewport_radius: f32,
    viewport_shadow: bool,
    controls_background: u32,
    controls_height: f32,
    controls_gap: f32,
    navigation_background: u32,
    navigation_foreground: u32,
    navigation_size: f32,
    navigation_disabled_opacity: f32,
    indicator_active_color: u32,
    indicator_inactive_color: u32,
    indicator_size: f32,
    active_indicator_width: f32,
    indicator_gap: f32,
}

impl CarouselStyle {
    fn resolve(self, theme: Theme, placement: CarouselControlsPlacement) -> ResolvedCarouselStyle {
        ResolvedCarouselStyle {
            viewport_background: self.viewport_background.unwrap_or(theme.colors.card),
            viewport_border_color: self.viewport_border_color.unwrap_or(theme.colors.border),
            viewport_border_width: self.viewport_border_width.max(0.0),
            viewport_radius: self.viewport_radius.unwrap_or(theme.radii.xl),
            viewport_shadow: self.viewport_shadow,
            controls_background: self.controls_background.unwrap_or_else(|| match placement {
                CarouselControlsPlacement::Below => TRANSPARENT,
                CarouselControlsPlacement::Overlay => with_alpha(theme.colors.surface, 0xE8),
                CarouselControlsPlacement::OverlayCenter => TRANSPARENT,
            }),
            controls_height: self.controls_height.max(MIN_TOUCH_TARGET),
            controls_gap: self.controls_gap.max(0.0),
            navigation_background: self.navigation_background.unwrap_or(theme.colors.secondary),
            navigation_foreground: self
                .navigation_foreground
                .unwrap_or(theme.colors.secondary_foreground),
            navigation_size: self.navigation_size.max(MIN_TOUCH_TARGET),
            navigation_disabled_opacity: self.navigation_disabled_opacity.clamp(0.0, 1.0),
            indicator_active_color: self.indicator_active_color.unwrap_or(theme.colors.primary),
            indicator_inactive_color: self
                .indicator_inactive_color
                .unwrap_or(theme.colors.primary_track),
            indicator_size: self.indicator_size.max(2.0),
            active_indicator_width: self
                .active_indicator_width
                .max(self.indicator_size.max(2.0)),
            indicator_gap: self.indicator_gap.max(0.0),
        }
    }
}

/// Props for [`Carousel`].
#[derive(Props, Clone, PartialEq)]
pub struct CarouselProps {
    /// One root element per page.
    pub slides: Vec<Element>,
    /// Controlled active page. When `Some`, the caller owns selection state.
    #[props(default)]
    pub index: Option<usize>,
    /// Initially active page for uncontrolled usage.
    #[props(default)]
    pub default_index: usize,
    /// Native Swiper viewport height.
    #[props(default = 240.0)]
    pub height: f32,
    /// Whether the last and first pages connect.
    #[props(default)]
    pub looping: bool,
    /// Whether pages advance automatically.
    #[props(default)]
    pub autoplay: bool,
    /// Delay between autoplay transitions, in milliseconds.
    #[props(default = 3000)]
    pub interval_ms: i32,
    /// Transition duration, in milliseconds.
    #[props(default = 300)]
    pub duration_ms: i32,
    /// Transition interpolation used by native Swiper.
    #[props(default)]
    pub transition_curve: CarouselTransitionCurve,
    /// Space between adjacent pages.
    #[props(default)]
    pub item_spacing: f32,
    /// Whether touch swiping is enabled.
    #[props(default = true)]
    pub swipe_enabled: bool,
    /// Shows previous and next buttons.
    #[props(default = true)]
    pub show_controls: bool,
    /// Shows a page indicator.
    #[props(default = true)]
    pub show_indicators: bool,
    /// Indicator presentation.
    #[props(default)]
    pub indicator_variant: CarouselIndicatorVariant,
    /// Controls row placement.
    #[props(default)]
    pub controls_placement: CarouselControlsPlacement,
    /// Theme-aware visual overrides.
    #[props(default)]
    pub style: CarouselStyle,
    /// Fires after native Swiper commits a page selected by swipe or controls.
    #[props(default)]
    pub on_change: EventHandler<usize>,
}

fn normalized_index(index: usize, slide_count: usize) -> usize {
    index.min(slide_count.saturating_sub(1))
}

fn previous_index(index: usize, slide_count: usize, looping: bool) -> usize {
    if slide_count == 0 {
        0
    } else if index == 0 && looping {
        slide_count - 1
    } else {
        index.saturating_sub(1)
    }
}

fn next_index(index: usize, slide_count: usize, looping: bool) -> usize {
    if slide_count == 0 {
        0
    } else if index + 1 >= slide_count {
        if looping {
            0
        } else {
            slide_count - 1
        }
    } else {
        index + 1
    }
}

/// A native, horizontally swipeable carousel with mobile-sized controls.
#[component]
pub fn Carousel(props: CarouselProps) -> Element {
    let theme = use_theme();
    let slide_count = props.slides.len();
    let initial = normalized_index(props.default_index, slide_count);
    let local_index = use_signal(move || initial);
    let mut touch_start_x = use_signal(|| None::<f32>);
    let mut touch_last_x = use_signal(|| None::<f32>);
    let last_emitted_index = use_signal(move || initial);
    let controlled = props.index.is_some();
    let active_index = normalized_index(
        props.index.unwrap_or_else(|| *local_index.read()),
        slide_count,
    );
    let active_index_i32 = i32::try_from(active_index).unwrap_or(i32::MAX);
    let cached_count = i32::try_from(slide_count).unwrap_or(i32::MAX);
    let on_change = props.on_change;
    let looping = props.looping;
    let placement = props.controls_placement;
    let style = props.style.resolve(theme, placement);

    let slides = props.slides.into_iter().enumerate().map(|(index, slide)| {
        rsx! {
            column {
                key: "{index}",
                width: "100%",
                height: "100%",
                align_items: "center",
                justify_content: "center",
                {slide}
            }
        }
    });

    let previous = previous_index(active_index, slide_count, looping);
    let next = next_index(active_index, slide_count, looping);
    let previous_disabled = slide_count < 2 || (!looping && active_index == 0);
    let next_disabled = slide_count < 2 || (!looping && active_index + 1 >= slide_count);
    let has_controls = slide_count > 1 && (props.show_controls || props.show_indicators);
    let mut previous_local = local_index;
    let mut next_local = local_index;
    let mut selected_local = local_index;
    let mut previous_emitted = last_emitted_index;
    let mut next_emitted = last_emitted_index;
    let mut selected_emitted = last_emitted_index;
    let on_previous = EventHandler::new(move |_| {
        if !controlled {
            previous_local.set(previous);
        }
        if *previous_emitted.read() != previous {
            previous_emitted.set(previous);
            on_change.call(previous);
        }
    });
    let on_next = EventHandler::new(move |_| {
        if !controlled {
            next_local.set(next);
        }
        if *next_emitted.read() != next {
            next_emitted.set(next);
            on_change.call(next);
        }
    });
    let on_select = EventHandler::new(move |index: usize| {
        if !controlled {
            selected_local.set(index);
        }
        if *selected_emitted.read() != index {
            selected_emitted.set(index);
            on_change.call(index);
        }
    });

    let mut native_local = local_index;
    let mut native_emitted = last_emitted_index;
    let mut touch_local = local_index;
    let mut touch_emitted = last_emitted_index;

    let viewport = rsx! {
        swiper {
            width: "100%",
            height: props.height.max(1.0),
            swiper_index: active_index_i32,
            swiper_swipe_to_index: active_index_i32,
            swiper_loop: looping,
            swiper_auto_play: props.autoplay && slide_count > 1,
            swiper_show_indicator: false,
            swiper_disable_swipe: !props.swipe_enabled || slide_count < 2,
            swiper_cached_count: cached_count,
            swiper_display_count: 1_i32,
            swiper_vertical: false,
            swiper_interval: props.interval_ms.max(0),
            swiper_duration: props.duration_ms.max(0),
            swiper_curve: props.transition_curve.arkui_value(),
            swiper_item_space: props.item_spacing.max(0.0),
            background_color: style.viewport_background,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: style.viewport_border_width,
            border_color: style.viewport_border_color,
            border_radius: style.viewport_radius,
            shadow: if style.viewport_shadow { "sm" },
            clip: true,
            on_swiper_change: move |event| {
                let index = usize::try_from(event.data().index).unwrap_or_default();
                let index = normalized_index(index, slide_count);
                if index != active_index {
                    if !controlled {
                        native_local.set(index);
                    }
                    if *native_emitted.read() != index {
                        native_emitted.set(index);
                        on_change.call(index);
                    }
                }
            },
            on_touch: move |event| {
                let Some(pointer) = event.data().pointer else {
                    return;
                };
                let pointer_x = if pointer.has_window_position() {
                    pointer.window_x
                } else {
                    pointer.x
                };
                match pointer.action {
                    dioxus_elements::event::PointerAction::Down => {
                        touch_start_x.set(Some(pointer_x));
                        touch_last_x.set(Some(pointer_x));
                    }
                    dioxus_elements::event::PointerAction::Move => {
                        touch_last_x.set(Some(pointer_x));
                    }
                    dioxus_elements::event::PointerAction::Up => {
                        let end_x = touch_last_x().unwrap_or(pointer_x);
                        let start_x = touch_start_x().unwrap_or(end_x);
                        let threshold = (pointer.target_width * 0.12).max(24.0);
                        let delta = end_x - start_x;
                        let target = if delta <= -threshold {
                            next
                        } else if delta >= threshold {
                            previous
                        } else {
                            active_index
                        };
                        touch_start_x.set(None);
                        touch_last_x.set(None);
                        if target != active_index {
                            if !controlled {
                                touch_local.set(target);
                            }
                            if *touch_emitted.read() != target {
                                touch_emitted.set(target);
                                on_change.call(target);
                            }
                        }
                    }
                    dioxus_elements::event::PointerAction::Cancel => {
                        touch_start_x.set(None);
                        touch_last_x.set(None);
                    }
                    dioxus_elements::event::PointerAction::Unknown => {}
                }
            },
            {slides}
        }
    };

    let controls = render_controls(
        theme,
        style,
        active_index,
        slide_count,
        props.show_controls,
        props.show_indicators,
        props.indicator_variant,
        previous_disabled,
        next_disabled,
        on_previous,
        on_next,
        on_select,
    );

    match (has_controls, placement) {
        (false, _) => viewport,
        (true, CarouselControlsPlacement::Below) => rsx! {
            column {
                width: "100%",
                {viewport}
                row { height: style.controls_gap }
                {controls}
            }
        },
        (true, CarouselControlsPlacement::Overlay) => rsx! {
            stack {
                width: "100%",
                height: props.height.max(1.0),
                {viewport}
                column {
                    width: "100%",
                    height: "100%",
                    align_items: "center",
                    justify_content: "end",
                    padding_right: spacing::MD,
                    padding_bottom: spacing::MD,
                    padding_left: spacing::MD,
                    hit_test_behavior: "transparent",
                    {controls}
                }
            }
        },
        (true, CarouselControlsPlacement::OverlayCenter) => rsx! {
            stack {
                width: "100%",
                height: props.height.max(1.0),
                {viewport}
                column {
                    width: "100%",
                    height: "100%",
                    align_items: "center",
                    justify_content: "center",
                    padding_right: spacing::MD,
                    padding_left: spacing::MD,
                    hit_test_behavior: "transparent",
                    {controls}
                }
            }
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn render_controls(
    theme: Theme,
    style: ResolvedCarouselStyle,
    active_index: usize,
    slide_count: usize,
    show_navigation: bool,
    show_indicators: bool,
    indicator_variant: CarouselIndicatorVariant,
    previous_disabled: bool,
    next_disabled: bool,
    on_previous: EventHandler<()>,
    on_next: EventHandler<()>,
    on_select: EventHandler<usize>,
) -> Element {
    let indicators: Vec<Element> = if show_indicators {
        match indicator_variant {
            CarouselIndicatorVariant::Fraction => vec![rsx! {
                text {
                    content: format!("{} / {slide_count}", active_index + 1),
                    font_size: typography::XS,
                    font_weight: 600_i32,
                    font_color: theme.colors.muted_foreground,
                    line_height: 16.0,
                }
            }],
            CarouselIndicatorVariant::Dot | CarouselIndicatorVariant::Pill => (0..slide_count)
                .map(|index| {
                    let selected = index == active_index;
                    let visual_width = if selected
                        && indicator_variant == CarouselIndicatorVariant::Pill
                    {
                        style.active_indicator_width
                    } else {
                        style.indicator_size
                    };
                    rsx! {
                        row {
                            key: "{index}",
                            width: (visual_width + 20.0).max(MIN_TOUCH_TARGET),
                            height: style.controls_height,
                            margin_left: if index == 0 { 0.0 } else { style.indicator_gap },
                            align_items: "center",
                            justify_content: "center",
                            focusable: false,
                            focus_on_touch: false,
                            onclick: move |_| on_select.call(index),
                            row {
                                width: visual_width,
                                height: style.indicator_size,
                                background_color: if selected { style.indicator_active_color } else { style.indicator_inactive_color },
                                border_radius: theme.radii.full,
                            }
                        }
                    }
                })
                .collect(),
        }
    } else {
        Vec::new()
    };

    rsx! {
        row {
            width: "100%",
            height: style.controls_height,
            align_items: "center",
            justify_content: "center",
            background_color: style.controls_background,
            border_radius: theme.radii.full,
            if show_navigation {
                CarouselNavigationButton {
                    icon: "chevron-left".to_string(),
                    disabled: previous_disabled,
                    style,
                    onclick: move |_| on_previous.call(()),
                }
            }
            row {
                layout_weight: 1.0,
                align_items: "center",
                justify_content: "center",
                {indicators.into_iter()}
            }
            if show_navigation {
                CarouselNavigationButton {
                    icon: "chevron-right".to_string(),
                    disabled: next_disabled,
                    style,
                    onclick: move |_| on_next.call(()),
                }
            }
        }
    }
}

#[component]
fn CarouselNavigationButton(
    icon: String,
    disabled: bool,
    style: ResolvedCarouselStyle,
    onclick: EventHandler<()>,
) -> Element {
    let theme = use_theme();
    rsx! {
        button {
            button_type: "normal",
            focusable: false,
            focus_on_touch: false,
            enabled: !disabled,
            width: style.navigation_size,
            height: style.navigation_size,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            alignment: "center",
            background_color: style.navigation_background,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 0.0,
            border_color: TRANSPARENT,
            border_radius: theme.radii.full,
            opacity: if disabled { style.navigation_disabled_opacity } else { 1.0 },
            onclick: move |_| onclick.call(()),
            {icon_placeholder(icon.as_str(), 20.0, style.navigation_foreground)}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        next_index, normalized_index, previous_index, CarouselStyle, CarouselTransitionCurve,
    };

    #[test]
    fn selection_is_clamped_to_available_slides() {
        assert_eq!(normalized_index(2, 4), 2);
        assert_eq!(normalized_index(9, 4), 3);
        assert_eq!(normalized_index(9, 0), 0);
    }

    #[test]
    fn navigation_stops_or_wraps_at_the_edges() {
        assert_eq!(previous_index(0, 4, false), 0);
        assert_eq!(previous_index(0, 4, true), 3);
        assert_eq!(next_index(3, 4, false), 3);
        assert_eq!(next_index(3, 4, true), 0);
        assert_eq!(next_index(0, 0, true), 0);
    }

    #[test]
    fn style_and_transition_defaults_are_mobile_sized() {
        let style = CarouselStyle::default();
        assert!(style.navigation_size >= 40.0);
        assert!(style.controls_height >= 40.0);
        assert_eq!(CarouselTransitionCurve::Smooth.arkui_value(), 11);
    }
}
