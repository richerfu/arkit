//! Mobile-first shadcn sliders with stable pointer-driven interaction.
//!
//! ArkUI's native Slider is useful for a single uncontrolled value, but its
//! change events race controlled value updates and it cannot represent range
//! or multiple-thumb selection. These components therefore share a small
//! custom track and pointer state machine. The value displayed while a finger
//! is down comes from local drag state; the controlled prop takes over again
//! when the gesture ends. That keeps the thumb under the finger without a
//! one-frame snap back.

use crate::theme::*;
use arkit_prelude::*;

const MIN_TOUCH_TARGET: f32 = 44.0;
const DEFAULT_VERTICAL_LENGTH: f32 = 200.0;
const DEFAULT_TRACK_THICKNESS: f32 = 4.0;
const DEFAULT_THUMB_SIZE: f32 = 16.0;
const DEFAULT_THUMB_BORDER_WIDTH: f32 = 2.0;
const MAX_STEP_MARKERS: usize = 100;

/// Slider layout direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SliderOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Theme-aware visual and interaction sizing overrides shared by all sliders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderStyle {
    /// Minimum size of the draggable interaction area in vp.
    pub touch_target: f32,
    /// Thickness of the visible track.
    pub track_thickness: f32,
    /// Diameter of each thumb.
    pub thumb_size: f32,
    /// Thumb fill color. Defaults to the active theme's background color.
    pub thumb_color: Option<u32>,
    /// Thumb border color. Defaults to the active theme's primary color.
    pub thumb_border_color: Option<u32>,
    /// Thumb border width.
    pub thumb_border_width: f32,
    /// Unselected track color.
    pub track_color: Option<u32>,
    /// Selected track color.
    pub selected_color: Option<u32>,
    /// Optional fixed step-marker color. By default markers adapt to the track.
    pub step_marker_color: Option<u32>,
    /// Opacity applied while disabled.
    pub disabled_opacity: f32,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            touch_target: MIN_TOUCH_TARGET,
            track_thickness: DEFAULT_TRACK_THICKNESS,
            thumb_size: DEFAULT_THUMB_SIZE,
            thumb_color: None,
            thumb_border_color: None,
            thumb_border_width: DEFAULT_THUMB_BORDER_WIDTH,
            track_color: None,
            selected_color: None,
            step_marker_color: None,
            disabled_opacity: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ResolvedSliderStyle {
    touch_target: f32,
    track_thickness: f32,
    thumb_size: f32,
    thumb_color: u32,
    thumb_border_color: u32,
    thumb_border_width: f32,
    track_color: u32,
    selected_color: u32,
    step_marker_color: Option<u32>,
    disabled_opacity: f32,
}

impl SliderStyle {
    fn resolve(self, theme: Theme) -> ResolvedSliderStyle {
        let touch_target = finite_or(self.touch_target, MIN_TOUCH_TARGET).max(MIN_TOUCH_TARGET);
        let thumb_size = finite_or(self.thumb_size, DEFAULT_THUMB_SIZE).clamp(8.0, touch_target);
        let track_thickness =
            finite_or(self.track_thickness, DEFAULT_TRACK_THICKNESS).clamp(1.0, thumb_size);
        let thumb_border_width = finite_or(self.thumb_border_width, DEFAULT_THUMB_BORDER_WIDTH)
            .clamp(0.0, thumb_size / 2.0);

        ResolvedSliderStyle {
            touch_target,
            track_thickness,
            thumb_size,
            thumb_color: self.thumb_color.unwrap_or(theme.colors.background),
            thumb_border_color: self.thumb_border_color.unwrap_or(theme.colors.primary),
            thumb_border_width,
            track_color: self.track_color.unwrap_or(theme.colors.primary_track),
            selected_color: self.selected_color.unwrap_or(theme.colors.primary),
            step_marker_color: self.step_marker_color,
            disabled_opacity: finite_or(self.disabled_opacity, 0.5).clamp(0.0, 1.0),
        }
    }
}

impl ResolvedSliderStyle {
    fn disabled(self, background: u32) -> Self {
        let opacity = self.disabled_opacity;
        Self {
            thumb_color: blend_onto_opaque_background(self.thumb_color, background, opacity),
            thumb_border_color: blend_onto_opaque_background(
                self.thumb_border_color,
                background,
                opacity,
            ),
            track_color: blend_onto_opaque_background(self.track_color, background, opacity),
            selected_color: blend_onto_opaque_background(self.selected_color, background, opacity),
            step_marker_color: self
                .step_marker_color
                .map(|color| blend_onto_opaque_background(color, background, opacity)),
            disabled_opacity: 1.0,
            ..self
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct NormalizedSliderValues {
    values: Vec<f32>,
    min: f32,
    max: f32,
    step: f32,
}

/// Props for the single-thumb [`Slider`].
#[derive(Props, Clone, PartialEq)]
pub struct SliderProps {
    /// Controlled slider value.
    pub value: f32,
    #[props(default)]
    pub min: Option<f32>,
    #[props(default)]
    pub max: Option<f32>,
    #[props(default)]
    pub step: Option<f32>,
    #[props(default)]
    pub orientation: SliderOrientation,
    /// Reverses the value direction. Vertical sliders commonly enable this so
    /// the minimum is at the bottom and the maximum is at the top.
    #[props(default)]
    pub reversed: bool,
    #[props(default)]
    pub disabled: bool,
    /// Shows markers at step boundaries when there are at most 100 intervals.
    #[props(default)]
    pub show_steps: bool,
    /// Optional explicit width. Horizontal sliders otherwise fill the parent.
    #[props(default)]
    pub width: Option<f32>,
    /// Optional explicit height. Vertical sliders default to 200vp.
    #[props(default)]
    pub height: Option<f32>,
    #[props(default)]
    pub style: SliderStyle,
    #[props(default)]
    pub on_change: Option<EventHandler<f32>>,
}

/// Props for the two-thumb [`RangeSlider`].
#[derive(Props, Clone, PartialEq)]
pub struct RangeSliderProps {
    /// Controlled lower and upper values.
    pub value: [f32; 2],
    #[props(default)]
    pub min: Option<f32>,
    #[props(default)]
    pub max: Option<f32>,
    #[props(default)]
    pub step: Option<f32>,
    #[props(default)]
    pub orientation: SliderOrientation,
    #[props(default)]
    pub reversed: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub show_steps: bool,
    #[props(default)]
    pub width: Option<f32>,
    #[props(default)]
    pub height: Option<f32>,
    #[props(default)]
    pub style: SliderStyle,
    #[props(default)]
    pub on_change: Option<EventHandler<[f32; 2]>>,
}

/// Props for the arbitrary-thumb [`MultiSlider`].
#[derive(Props, Clone, PartialEq)]
pub struct MultiSliderProps {
    /// Controlled values. Values are sorted and each value becomes one thumb.
    pub values: Vec<f32>,
    #[props(default)]
    pub min: Option<f32>,
    #[props(default)]
    pub max: Option<f32>,
    #[props(default)]
    pub step: Option<f32>,
    #[props(default)]
    pub orientation: SliderOrientation,
    #[props(default)]
    pub reversed: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub show_steps: bool,
    #[props(default)]
    pub width: Option<f32>,
    #[props(default)]
    pub height: Option<f32>,
    #[props(default)]
    pub style: SliderStyle,
    #[props(default)]
    pub on_change: Option<EventHandler<Vec<f32>>>,
}

/// A controlled single-thumb slider with shadcn mobile defaults.
#[component]
pub fn Slider(props: SliderProps) -> Element {
    let values = normalize_slider_values(&[props.value], props.min, props.max, props.step);
    let on_change = props.on_change;

    rsx! {
        SliderTrack {
            values,
            orientation: props.orientation,
            reversed: props.reversed,
            disabled: props.disabled,
            show_steps: props.show_steps,
            width: props.width,
            height: props.height,
            style: props.style,
            on_change: move |values: Vec<f32>| {
                if let (Some(handler), Some(value)) = (on_change, values.first()) {
                    handler.call(*value);
                }
            },
        }
    }
}

/// A controlled two-thumb slider for selecting a lower and upper bound.
#[component]
pub fn RangeSlider(props: RangeSliderProps) -> Element {
    let values = normalize_slider_values(&props.value, props.min, props.max, props.step);
    let on_change = props.on_change;

    rsx! {
        SliderTrack {
            values,
            orientation: props.orientation,
            reversed: props.reversed,
            disabled: props.disabled,
            show_steps: props.show_steps,
            width: props.width,
            height: props.height,
            style: props.style,
            on_change: move |values: Vec<f32>| {
                if let (Some(handler), [lower, upper]) = (on_change, values.as_slice()) {
                    handler.call([*lower, *upper]);
                }
            },
        }
    }
}

/// A controlled slider with any number of ordered thumbs.
#[component]
pub fn MultiSlider(props: MultiSliderProps) -> Element {
    let values = normalize_slider_values(&props.values, props.min, props.max, props.step);
    let on_change = props.on_change;

    rsx! {
        SliderTrack {
            values,
            orientation: props.orientation,
            reversed: props.reversed,
            disabled: props.disabled,
            show_steps: props.show_steps,
            width: props.width,
            height: props.height,
            style: props.style,
            on_change: move |values: Vec<f32>| {
                if let Some(handler) = on_change {
                    handler.call(values);
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActiveDrag {
    thumb_index: usize,
    pointer_id: i32,
}

#[derive(Props, Clone, PartialEq)]
struct SliderTrackProps {
    values: NormalizedSliderValues,
    orientation: SliderOrientation,
    reversed: bool,
    disabled: bool,
    show_steps: bool,
    width: Option<f32>,
    height: Option<f32>,
    style: SliderStyle,
    on_change: EventHandler<Vec<f32>>,
}

#[component]
fn SliderTrack(props: SliderTrackProps) -> Element {
    let theme = use_theme();
    let style = props.style.resolve(theme);
    let style = if props.disabled {
        style.disabled(theme.colors.background)
    } else {
        style
    };
    let native_width = props.width.or_else(|| {
        (props.orientation == SliderOrientation::Vertical).then_some(style.touch_target)
    });
    let native_height = props.height.unwrap_or(match props.orientation {
        SliderOrientation::Horizontal => style.touch_target,
        SliderOrientation::Vertical => DEFAULT_VERTICAL_LENGTH,
    });
    let percent_width = (props.orientation == SliderOrientation::Horizontal
        && native_width.is_none())
    .then_some(1.0_f32);
    let initial_length = match props.orientation {
        SliderOrientation::Horizontal => native_width.unwrap_or_default(),
        SliderOrientation::Vertical => native_height.max(style.touch_target),
    };
    let mut live_values = use_signal(|| props.values.values.clone());
    let mut active_drag = use_signal(|| None::<ActiveDrag>);
    let mut measured_length = use_signal(|| initial_length);
    let active = active_drag();
    let display_values = if active.is_some() {
        live_values()
    } else {
        props.values.values.clone()
    };
    let selected_bounds = selection_bounds(
        &display_values,
        props.values.min,
        props.values.max,
        props.reversed,
    );
    let track = render_track(
        props.orientation,
        style,
        selected_bounds,
        props.show_steps,
        props.values.step,
        props.values.max - props.values.min,
        measured_length(),
    );
    let thumbs = display_values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            render_thumb(
                index,
                value_fraction(*value, props.values.min, props.values.max, props.reversed),
                props.orientation,
                style,
                active.is_some_and(|drag| drag.thumb_index == index),
                measured_length(),
            )
        })
        .collect::<Vec<_>>();
    let values = props.values.clone();
    let orientation = props.orientation;
    let reversed = props.reversed;
    let disabled = props.disabled;
    let on_change = props.on_change;
    let density = display_vp_ratio();

    rsx! {
        stack {
            width: if let Some(width) = native_width { width.max(style.touch_target) },
            percent_width: if let Some(width) = percent_width { width },
            height: native_height.max(style.touch_target),
            alignment: 0_i32,
            enabled: !disabled,
            on_layout: move |event| {
                let frame = event.data().frame;
                if !frame.is_measured() {
                    return;
                }
                let length = match orientation {
                    SliderOrientation::Horizontal => frame.width,
                    SliderOrientation::Vertical => frame.height,
                } / density;
                if length.is_finite()
                    && length > 0.0
                    && (length - *measured_length.peek()).abs() > 0.25
                {
                    measured_length.set(length);
                }
            },
            on_touch: move |event| {
                if disabled {
                    return;
                }
                let Some(pointer) = event.data().pointer else {
                    return;
                };

                match pointer.action {
                    dioxus_elements::event::PointerAction::Down => {
                        if active_drag.peek().is_some() || !pointer_hits_target(pointer) {
                            return;
                        }
                        let Some(value) = pointer_value(
                            pointer,
                            orientation,
                            reversed,
                            style.thumb_size,
                            density,
                            &values,
                        ) else {
                            return;
                        };
                        let thumb_index = nearest_thumb_index(&values.values, value);
                        let next = update_thumb_value(
                            &values.values,
                            thumb_index,
                            value,
                            values.min,
                            values.max,
                            values.step,
                        );
                        live_values.set(next.clone());
                        active_drag.set(Some(ActiveDrag {
                            thumb_index,
                            pointer_id: pointer.pointer_id,
                        }));
                        if next != values.values {
                            on_change.call(next);
                        }
                    }
                    dioxus_elements::event::PointerAction::Move => {
                        let Some(drag) = *active_drag.peek() else {
                            return;
                        };
                        if drag.pointer_id != pointer.pointer_id {
                            return;
                        }
                        let Some(value) = pointer_value(
                            pointer,
                            orientation,
                            reversed,
                            style.thumb_size,
                            density,
                            &values,
                        ) else {
                            return;
                        };
                        let current = live_values.peek().clone();
                        let next = update_thumb_value(
                            &current,
                            drag.thumb_index,
                            value,
                            values.min,
                            values.max,
                            values.step,
                        );
                        if next != current {
                            live_values.set(next.clone());
                            on_change.call(next);
                        }
                    }
                    dioxus_elements::event::PointerAction::Up => {
                        let Some(drag) = *active_drag.peek() else {
                            return;
                        };
                        if drag.pointer_id != pointer.pointer_id {
                            return;
                        }
                        if let Some(value) = pointer_value(
                            pointer,
                            orientation,
                            reversed,
                            style.thumb_size,
                            density,
                            &values,
                        ) {
                            let current = live_values.peek().clone();
                            let next = update_thumb_value(
                                &current,
                                drag.thumb_index,
                                value,
                                values.min,
                                values.max,
                                values.step,
                            );
                            if next != current {
                                live_values.set(next.clone());
                                on_change.call(next);
                            }
                        }
                        active_drag.set(None);
                    }
                    dioxus_elements::event::PointerAction::Cancel => {
                        if active_drag
                            .peek()
                            .is_some_and(|drag| drag.pointer_id == pointer.pointer_id)
                        {
                            active_drag.set(None);
                        }
                    }
                    dioxus_elements::event::PointerAction::Unknown => {}
                }
            },
            {track}
            for thumb in thumbs {
                {thumb}
            }
        }
    }
}

fn render_track(
    orientation: SliderOrientation,
    style: ResolvedSliderStyle,
    selected_bounds: (f32, f32),
    show_steps: bool,
    step: f32,
    range: f32,
    length: f32,
) -> Element {
    let marker_count = step_marker_count(show_steps, step, range);
    let usable_length = (length - style.thumb_size).max(0.0);
    let selected_start = selected_bounds.0 * usable_length;
    let selected_length = (selected_bounds.1 - selected_bounds.0) * usable_length;

    match orientation {
        SliderOrientation::Horizontal => rsx! {
            row {
                position: format!(
                    "{},{}",
                    style.thumb_size / 2.0,
                    (style.touch_target - style.track_thickness) / 2.0,
                ),
                width: usable_length,
                height: style.track_thickness,
                background_color: style.track_color,
                border_radius: style.track_thickness / 2.0,
                hit_test_behavior: 2_i32,
            }
            if selected_length > f32::EPSILON {
                row {
                    position: format!(
                        "{},{}",
                        style.thumb_size / 2.0 + selected_start,
                        (style.touch_target - style.track_thickness) / 2.0,
                    ),
                    width: selected_length,
                    height: style.track_thickness,
                    background_color: style.selected_color,
                    border_radius: style.track_thickness / 2.0,
                    hit_test_behavior: 2_i32,
                }
            }
            if let Some(marker_count) = marker_count {
                for index in 0..marker_count {
                    row {
                        key: "marker-{index}",
                        position: format!(
                            "{},{}",
                            style.thumb_size / 2.0
                                + marker_fraction(index, marker_count) * usable_length
                                - 1.0,
                            style.touch_target / 2.0 - 1.0,
                        ),
                        width: 2.0,
                        height: 2.0,
                        border_radius: 1.0,
                        background_color: marker_color(
                            index,
                            marker_count,
                            selected_bounds,
                            style,
                        ),
                        hit_test_behavior: 2_i32,
                    }
                }
            }
        },
        SliderOrientation::Vertical => rsx! {
            column {
                position: format!(
                    "{},{}",
                    (style.touch_target - style.track_thickness) / 2.0,
                    style.thumb_size / 2.0,
                ),
                width: style.track_thickness,
                height: usable_length,
                background_color: style.track_color,
                border_radius: style.track_thickness / 2.0,
                hit_test_behavior: 2_i32,
            }
            if selected_length > f32::EPSILON {
                column {
                    position: format!(
                        "{},{}",
                        (style.touch_target - style.track_thickness) / 2.0,
                        style.thumb_size / 2.0 + selected_start,
                    ),
                    width: style.track_thickness,
                    height: selected_length,
                    background_color: style.selected_color,
                    border_radius: style.track_thickness / 2.0,
                    hit_test_behavior: 2_i32,
                }
            }
            if let Some(marker_count) = marker_count {
                for index in 0..marker_count {
                    row {
                        key: "marker-{index}",
                        position: format!(
                            "{},{}",
                            style.touch_target / 2.0 - 1.0,
                            style.thumb_size / 2.0
                                + marker_fraction(index, marker_count) * usable_length
                                - 1.0,
                        ),
                        width: 2.0,
                        height: 2.0,
                        border_radius: 1.0,
                        background_color: marker_color(
                            index,
                            marker_count,
                            selected_bounds,
                            style,
                        ),
                        hit_test_behavior: 2_i32,
                    }
                }
            }
        },
    }
}

fn marker_fraction(index: usize, marker_count: usize) -> f32 {
    index as f32 / marker_count.saturating_sub(1).max(1) as f32
}

fn render_thumb(
    index: usize,
    fraction: f32,
    orientation: SliderOrientation,
    style: ResolvedSliderStyle,
    active: bool,
    length: f32,
) -> Element {
    let fraction = fraction.clamp(0.0, 1.0);
    let usable_length = (length - style.thumb_size).max(0.0);
    let (x, y) = match orientation {
        SliderOrientation::Horizontal => (
            fraction * usable_length,
            (style.touch_target - style.thumb_size) / 2.0,
        ),
        SliderOrientation::Vertical => (
            (style.touch_target - style.thumb_size) / 2.0,
            fraction * usable_length,
        ),
    };
    let border_width = if active {
        (style.thumb_border_width + 1.0).min(style.thumb_size / 2.0)
    } else {
        style.thumb_border_width
    };

    rsx! {
        row {
            key: "thumb-{index}",
            position: format!("{x},{y}"),
            width: style.thumb_size,
            height: style.thumb_size,
            background_color: style.thumb_color,
            border_width,
            border_color: style.thumb_border_color,
            border_radius: style.thumb_size / 2.0,
            hit_test_behavior: 2_i32,
        }
    }
}

fn marker_color(
    index: usize,
    marker_count: usize,
    selected_bounds: (f32, f32),
    style: ResolvedSliderStyle,
) -> u32 {
    if let Some(color) = style.step_marker_color {
        return color;
    }
    let denominator = marker_count.saturating_sub(1).max(1) as f32;
    let fraction = index as f32 / denominator;
    if fraction >= selected_bounds.0 && fraction <= selected_bounds.1 {
        style.thumb_color
    } else {
        style.selected_color
    }
}

fn step_marker_count(show_steps: bool, step: f32, range: f32) -> Option<usize> {
    if !show_steps || !step.is_finite() || !range.is_finite() || step <= 0.0 || range <= 0.0 {
        return None;
    }
    let intervals = (range / step).round();
    if !(1.0..=MAX_STEP_MARKERS as f32).contains(&intervals) {
        return None;
    }
    Some(intervals as usize + 1)
}

fn selection_bounds(values: &[f32], min: f32, max: f32, reversed: bool) -> (f32, f32) {
    let Some(first) = values.first() else {
        return (0.0, 0.0);
    };
    let start_value = if values.len() == 1 { min } else { *first };
    let end_value = *values.last().unwrap_or(first);
    let first_fraction = value_fraction(start_value, min, max, reversed);
    let last_fraction = value_fraction(end_value, min, max, reversed);
    (
        first_fraction.min(last_fraction),
        first_fraction.max(last_fraction),
    )
}

fn normalize_slider_values(
    values: &[f32],
    min: Option<f32>,
    max: Option<f32>,
    step: Option<f32>,
) -> NormalizedSliderValues {
    let min = min.map_or(0.0, |value| finite_or(value, 0.0));
    let max = max.map_or(100.0, |value| finite_or(value, 100.0));
    let (min, max) = if min < max { (min, max) } else { (0.0, 100.0) };
    let range = max - min;
    let requested_step = finite_or(step.unwrap_or(1.0), 1.0);
    let step = if requested_step > 0.0 {
        requested_step.min(range)
    } else {
        1.0_f32.min(range)
    };
    let source = if values.is_empty() {
        &[min][..]
    } else {
        values
    };
    let mut values = source
        .iter()
        .map(|value| snap_slider_value(*value, min, max, step))
        .collect::<Vec<_>>();
    values.sort_by(f32::total_cmp);

    NormalizedSliderValues {
        values,
        min,
        max,
        step,
    }
}

fn update_thumb_value(
    values: &[f32],
    index: usize,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
) -> Vec<f32> {
    let mut next = values.to_vec();
    let Some(slot) = next.get_mut(index) else {
        return next;
    };
    let lower = index
        .checked_sub(1)
        .and_then(|previous| values.get(previous))
        .copied()
        .unwrap_or(min);
    let upper = values.get(index + 1).copied().unwrap_or(max);
    *slot = snap_slider_value(value, min, max, step).clamp(lower, upper);
    next
}

fn nearest_thumb_index(values: &[f32], value: f32) -> usize {
    values
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| (*left - value).abs().total_cmp(&(*right - value).abs()))
        .map_or(0, |(index, _)| index)
}

fn pointer_value(
    pointer: dioxus_elements::event::PointerPayload,
    orientation: SliderOrientation,
    reversed: bool,
    thumb_size: f32,
    density: f32,
    values: &NormalizedSliderValues,
) -> Option<f32> {
    if !pointer.has_target_bounds() {
        return None;
    }
    let (position, length) = match orientation {
        SliderOrientation::Horizontal => (pointer.x, pointer.target_width),
        SliderOrientation::Vertical => (pointer.y, pointer.target_height),
    };
    if !position.is_finite() || !length.is_finite() || length <= 0.0 {
        return None;
    }
    let inset = (thumb_size * density.max(1.0) / 2.0).min(length / 2.0);
    let usable_length = (length - inset * 2.0).max(f32::EPSILON);
    let mut fraction = ((position - inset) / usable_length).clamp(0.0, 1.0);
    if reversed {
        fraction = 1.0 - fraction;
    }
    Some(snap_slider_value(
        values.min + fraction * (values.max - values.min),
        values.min,
        values.max,
        values.step,
    ))
}

fn pointer_hits_target(pointer: dioxus_elements::event::PointerPayload) -> bool {
    pointer.has_target_bounds()
        && pointer.x.is_finite()
        && pointer.y.is_finite()
        && (0.0..=pointer.target_width).contains(&pointer.x)
        && (0.0..=pointer.target_height).contains(&pointer.y)
}

fn value_fraction(value: f32, min: f32, max: f32, reversed: bool) -> f32 {
    let mut fraction = ((clamp_slider_value(value, min, max) - min) / (max - min)).clamp(0.0, 1.0);
    if reversed {
        fraction = 1.0 - fraction;
    }
    fraction
}

fn snap_slider_value(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let value = clamp_slider_value(value, min, max);
    if value <= min || value >= max {
        return value;
    }
    (min + ((value - min) / step).round() * step).clamp(min, max)
}

fn clamp_slider_value(value: f32, min: f32, max: f32) -> f32 {
    finite_or(value, min).clamp(min, max)
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

fn blend_onto_opaque_background(foreground: u32, background: u32, opacity: f32) -> u32 {
    let source_alpha = ((foreground >> 24) & 0xFF) as f32 / 255.0;
    let alpha = (source_alpha * opacity.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let blend_channel = |shift: u32| {
        let source = ((foreground >> shift) & 0xFF) as f32;
        let destination = ((background >> shift) & 0xFF) as f32;
        (source * alpha + destination * (1.0 - alpha)).round() as u32
    };

    0xFF00_0000 | (blend_channel(16) << 16) | (blend_channel(8) << 8) | blend_channel(0)
}

fn display_vp_ratio() -> f32 {
    let ratio = ohos_display_binding::default_display_virtual_pixel_ratio();
    if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_snapped_sorted_and_clamped() {
        assert_eq!(
            normalize_slider_values(&[120.0, 41.0, -8.0], Some(0.0), Some(100.0), Some(5.0),),
            NormalizedSliderValues {
                values: vec![0.0, 40.0, 100.0],
                min: 0.0,
                max: 100.0,
                step: 5.0,
            }
        );
    }

    #[test]
    fn invalid_bounds_numbers_and_empty_values_use_stable_defaults() {
        assert_eq!(
            normalize_slider_values(&[], Some(10.0), Some(10.0), Some(-2.0)),
            NormalizedSliderValues {
                values: vec![0.0],
                min: 0.0,
                max: 100.0,
                step: 1.0,
            }
        );
    }

    #[test]
    fn range_thumbs_cannot_cross() {
        let values = vec![20.0, 80.0];
        assert_eq!(
            update_thumb_value(&values, 0, 95.0, 0.0, 100.0, 1.0),
            vec![80.0, 80.0]
        );
        assert_eq!(
            update_thumb_value(&values, 1, 5.0, 0.0, 100.0, 1.0),
            vec![20.0, 20.0]
        );
    }

    #[test]
    fn selection_uses_minimum_for_one_thumb_and_outer_values_for_many() {
        assert_eq!(selection_bounds(&[25.0], 0.0, 100.0, false), (0.0, 0.25));
        assert_eq!(
            selection_bounds(&[20.0, 50.0, 80.0], 0.0, 100.0, false),
            (0.2, 0.8)
        );
        assert_eq!(
            selection_bounds(&[20.0, 80.0], 0.0, 100.0, true),
            (0.2, 0.8)
        );
    }

    #[test]
    fn pointer_mapping_matches_thumb_insets_and_reverse_direction() {
        let pointer = dioxus_elements::event::PointerPayload {
            x: 100.0,
            y: 50.0,
            target_width: 200.0,
            target_height: 100.0,
            ..dioxus_elements::event::PointerPayload::default()
        };
        let values = NormalizedSliderValues {
            values: vec![50.0],
            min: 0.0,
            max: 100.0,
            step: 1.0,
        };
        assert_eq!(
            pointer_value(
                pointer,
                SliderOrientation::Horizontal,
                false,
                16.0,
                2.0,
                &values,
            ),
            Some(50.0)
        );
        assert_eq!(
            pointer_value(
                pointer,
                SliderOrientation::Vertical,
                true,
                16.0,
                2.0,
                &values,
            ),
            Some(50.0)
        );
    }

    #[test]
    fn pointer_down_must_be_inside_the_slider_touch_target() {
        let inside = dioxus_elements::event::PointerPayload {
            x: 20.0,
            y: 20.0,
            target_width: 200.0,
            target_height: 44.0,
            ..dioxus_elements::event::PointerPayload::default()
        };
        let outside = dioxus_elements::event::PointerPayload { y: 120.0, ..inside };

        assert!(pointer_hits_target(inside));
        assert!(!pointer_hits_target(outside));
    }

    #[test]
    fn style_preserves_mobile_touch_target_and_safe_thumb_geometry() {
        let resolved = SliderStyle {
            touch_target: 20.0,
            track_thickness: 40.0,
            thumb_size: 10.0,
            thumb_border_width: 20.0,
            disabled_opacity: 2.0,
            ..SliderStyle::default()
        }
        .resolve(Theme::default());

        assert_eq!(resolved.touch_target, 44.0);
        assert_eq!(resolved.track_thickness, 10.0);
        assert_eq!(resolved.thumb_border_width, 5.0);
        assert_eq!(resolved.disabled_opacity, 1.0);
    }

    #[test]
    fn disabled_style_uses_opaque_colors_so_the_track_cannot_bleed_through_the_thumb() {
        let background = 0xFFFF_FFFF;
        let disabled = SliderStyle::default()
            .resolve(Theme::default())
            .disabled(background);

        assert_eq!(disabled.thumb_color, background);
        assert_eq!(disabled.thumb_color >> 24, 0xFF);
        assert_eq!(disabled.thumb_border_color >> 24, 0xFF);
        assert_eq!(disabled.track_color >> 24, 0xFF);
        assert_eq!(disabled.selected_color >> 24, 0xFF);
        assert_eq!(disabled.disabled_opacity, 1.0);
    }

    #[test]
    fn excessive_step_markers_are_suppressed() {
        assert_eq!(step_marker_count(true, 1.0, 10.0), Some(11));
        assert_eq!(step_marker_count(true, 0.1, 100.0), None);
        assert_eq!(step_marker_count(false, 1.0, 10.0), None);
    }
}
