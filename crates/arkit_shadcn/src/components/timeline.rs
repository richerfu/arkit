//! Timeline — shadcn/Origin UI-style chronological event list.
//!
//! Compound primitives match the published Origin UI registry item
//! (`Timeline`, `TimelineItem`, `TimelineHeader`, `TimelineDate`,
//! `TimelineTitle`, `TimelineContent`, `TimelineIndicator`,
//! `TimelineSeparator`). ArkUI percentage heights resolve to a sized
//! ancestor (the page), so [`TimelineItem`] sizes the rail from the
//! measured body and paints the connector *inside* that box. Alternate
//! layout is a 3-slot strip: equal start/end slots with the axis in the
//! middle, never a side-gap that would shift the rail. Horizontal rails
//! sit on one cross-axis and can scroll when
//! [`TimelineProps::item_min_width`] is set.
//!
//! `value` / `default_value` are the active step. Items with `step <= active`
//! render as completed (indicator border and the separator leading *to* the
//! next completed item use primary). The last item sets `last` so its
//! separator is omitted.

use super::ARKUI_BORDER_STYLE_SOLID;
use crate::theme::*;
use arkit_prelude::*;

const DEFAULT_ACTIVE_STEP: i32 = 1;
const DEFAULT_INDICATOR_SIZE: f32 = 16.0;
const INDICATOR_BORDER_WIDTH: f32 = 2.0;
const SEPARATOR_THICKNESS: f32 = 2.0;
const RAIL_CONTENT_GAP: f32 = spacing::LG;
const ITEM_GAP: f32 = spacing::LG;
const SEPARATOR_INSET: f32 = 4.0;
const DATE_LINE_HEIGHT: f32 = 16.0;
const TITLE_LINE_HEIGHT: f32 = 20.0;
const CONTENT_LINE_HEIGHT: f32 = 20.0;
const CONTENT_HEADER_GAP: f32 = 2.0;
const PENDING_SEPARATOR_ALPHA: u8 = 0x1A;
const HIT_FILL: u32 = 0x0100_0000;

/// Layout direction of [`Timeline`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimelineOrientation {
    #[default]
    Vertical,
    Horizontal,
}

/// Where item content sits relative to the rail.
///
/// Vertical: `Right` puts content east of the axis, `Left` west of it.
/// `Alternate` centers the axis and flips sides by `step` parity.
/// Horizontal: `Right` is content below the axis, `Left` above it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimelineAlign {
    #[default]
    Right,
    Left,
    Alternate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentSide {
    Start,
    End,
}

fn content_side(align: TimelineAlign, step: i32) -> ContentSide {
    match align {
        TimelineAlign::Right => ContentSide::End,
        TimelineAlign::Left => ContentSide::Start,
        TimelineAlign::Alternate => {
            if step.rem_euclid(2) == 1 {
                ContentSide::End
            } else {
                ContentSide::Start
            }
        }
    }
}

fn text_align_end(orientation: TimelineOrientation, side: ContentSide) -> bool {
    orientation == TimelineOrientation::Vertical && side == ContentSide::Start
}

fn axis_is_centered(align: TimelineAlign) -> bool {
    matches!(align, TimelineAlign::Alternate)
}

#[derive(Clone, Copy)]
struct TimelineItemText {
    align_end: Signal<bool>,
}

fn use_item_text_end() -> bool {
    try_use_context::<TimelineItemText>()
        .map(|item| (item.align_end)())
        .unwrap_or(false)
}

fn item_text_align(align_end: bool) -> &'static str {
    if align_end {
        "end"
    } else {
        "start"
    }
}

#[derive(Clone, Copy)]
struct TimelineContext {
    active_step: Signal<i32>,
    orientation: Signal<TimelineOrientation>,
    align: Signal<TimelineAlign>,
    interactive: Signal<bool>,
    controlled: Signal<bool>,
    item_min_width: Signal<Option<f32>>,
    on_value_change: Signal<EventHandler<i32>>,
    /// Measured height of the horizontal item row. Written back onto every
    /// item so Left/Right rails share one Y even inside a scroll.
    axis_extent: Signal<f32>,
    /// Per-item body heights. Horizontal alternate uses twice the max plus
    /// the rail so the 1fr/auto/1fr slots actually fit the content.
    cross_extents: Signal<Vec<(i32, f32)>>,
}

fn record_item_extent(mut extents: Signal<Vec<(i32, f32)>>, step: i32, height_vp: f32) {
    if !height_vp.is_finite() || height_vp <= 0.0 {
        return;
    }
    let mut list = extents.peek().clone();
    match list.iter_mut().find(|(item_step, _)| *item_step == step) {
        Some((_, stored)) => {
            if (*stored - height_vp).abs() > 0.5 {
                *stored = height_vp;
                extents.set(list);
            }
        }
        None => {
            list.push((step, height_vp));
            extents.set(list);
        }
    }
}

fn max_item_extent(extents: &[(i32, f32)]) -> f32 {
    extents
        .iter()
        .map(|(_, height)| *height)
        .fold(0.0_f32, f32::max)
}

fn vertical_rail_extent(body_height_vp: f32, indicator_size: f32) -> f32 {
    body_height_vp.max(indicator_size)
}

fn connector_from_center(extent: f32, indicator_size: f32, last: bool) -> f32 {
    if last {
        0.0
    } else {
        (extent - indicator_size * 0.5).max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ItemMainAxis {
    Flex,
    Fixed(f32),
}

fn item_main_axis(orientation: TimelineOrientation, item_min_width: Option<f32>) -> ItemMainAxis {
    match orientation {
        TimelineOrientation::Horizontal => item_min_width
            .filter(|width| width.is_finite() && *width > 0.0)
            .map(ItemMainAxis::Fixed)
            .unwrap_or(ItemMainAxis::Flex),
        TimelineOrientation::Vertical => ItemMainAxis::Flex,
    }
}

fn timeline_scrolls_horizontally(
    orientation: TimelineOrientation,
    item_min_width: Option<f32>,
) -> bool {
    matches!(
        item_main_axis(orientation, item_min_width),
        ItemMainAxis::Fixed(_)
    )
}

fn item_completed(step: i32, active_step: i32, completed: Option<bool>) -> bool {
    completed.unwrap_or(step <= active_step)
}

fn separator_completed(step: i32, active_step: i32) -> bool {
    step < active_step
}

fn indicator_icon_size(indicator_size: f32) -> f32 {
    (indicator_size * 0.625).clamp(8.0, indicator_size)
}

/// Props for [`Timeline`].
#[derive(Props, Clone, PartialEq)]
pub struct TimelineProps {
    /// Controlled active step. `Some` makes the timeline controlled.
    #[props(default)]
    pub value: Option<i32>,
    /// Initial active step when uncontrolled. Defaults to `1`.
    #[props(default)]
    pub default_value: Option<i32>,
    #[props(default)]
    pub orientation: TimelineOrientation,
    /// Content placement relative to the rail. See [`TimelineAlign`].
    #[props(default)]
    pub align: TimelineAlign,
    /// Horizontal item minimum width in vp. When set, items keep that width
    /// and the timeline scrolls sideways instead of compressing them. Vertical
    /// orientation ignores this. `None` shares the container width equally.
    #[props(default)]
    pub item_min_width: Option<f32>,
    /// When true, tapping an item selects its `step`.
    #[props(default)]
    pub interactive: bool,
    #[props(default)]
    pub on_value_change: EventHandler<i32>,
    pub children: Element,
}

/// Chronological list container. Provides active-step context to items.
#[component]
pub fn Timeline(props: TimelineProps) -> Element {
    let initial = props
        .value
        .unwrap_or(props.default_value.unwrap_or(DEFAULT_ACTIVE_STEP));
    let mut active_step = use_signal(|| initial);
    let mut orientation = use_signal(|| props.orientation);
    let mut align = use_signal(|| props.align);
    let mut interactive = use_signal(|| props.interactive);
    let mut controlled = use_signal(|| props.value.is_some());
    let mut item_min_width = use_signal(|| props.item_min_width);
    let mut on_value_change = use_signal(|| props.on_value_change);
    let mut axis_extent = use_signal(|| 0.0_f32);
    let mut cross_extents = use_signal(Vec::<(i32, f32)>::new);
    let controlled_value = props.value;
    let next_orientation = props.orientation;
    let next_align = props.align;
    let next_interactive = props.interactive;
    let next_controlled = props.value.is_some();
    let next_item_min_width = props.item_min_width;
    let next_on_value_change = props.on_value_change;

    use_effect(use_reactive(
        (
            &controlled_value,
            &next_orientation,
            &next_align,
            &next_interactive,
            &next_controlled,
            &next_item_min_width,
            &next_on_value_change,
        ),
        move |(
            controlled_value,
            next_orientation,
            next_align,
            next_interactive,
            next_controlled,
            next_item_min_width,
            next_on_value_change,
        )| {
            if let Some(value) = controlled_value {
                if *active_step.peek() != value {
                    active_step.set(value);
                }
            }
            if *orientation.peek() != next_orientation {
                orientation.set(next_orientation);
                axis_extent.set(0.0);
                cross_extents.set(Vec::new());
            }
            if *align.peek() != next_align {
                align.set(next_align);
                axis_extent.set(0.0);
            }
            if *interactive.peek() != next_interactive {
                interactive.set(next_interactive);
            }
            if *controlled.peek() != next_controlled {
                controlled.set(next_controlled);
            }
            if *item_min_width.peek() != next_item_min_width {
                item_min_width.set(next_item_min_width);
            }
            on_value_change.set(next_on_value_change);
        },
    ));

    let ctx = TimelineContext {
        active_step,
        orientation,
        align,
        interactive,
        controlled,
        item_min_width,
        on_value_change,
        axis_extent,
        cross_extents,
    };
    use_context_provider(|| ctx);

    let children = props.children;
    let scrolls = timeline_scrolls_horizontally(props.orientation, props.item_min_width);
    let row_ref = arkit_hooks::use_native_element_ref();
    let scale = arkit_hooks::use_window_metrics().scale.max(f32::EPSILON);
    arkit_hooks::use_layout_size(row_ref.clone(), move |size| {
        let mut axis_extent = axis_extent;
        let height_vp = size.height / scale;
        // Ignore the first-pass blow-up when weighted slots run without a
        // definite parent height (they inherit the page remaining space).
        if height_vp > 0.0 && height_vp < 2000.0 && (*axis_extent.peek() - height_vp).abs() > 0.5 {
            axis_extent.set(height_vp);
        }
    });
    let body = match props.orientation {
        TimelineOrientation::Vertical => rsx! {
            column {
                width: "100%",
                align_items: "start",
                {children}
            }
        },
        TimelineOrientation::Horizontal => {
            if scrolls {
                rsx! {
                    row {
                        native_ref: row_ref,
                        align_items: "stretch",
                        justify_content: "start",
                        {children}
                    }
                }
            } else {
                rsx! {
                    row {
                        native_ref: row_ref,
                        width: "100%",
                        align_items: "stretch",
                        justify_content: "start",
                        {children}
                    }
                }
            }
        }
    };

    if scrolls {
        rsx! {
            scroll {
                width: "100%",
                scroll_direction: "horizontal",
                scroll_bar: "auto",
                {body}
            }
        }
    } else {
        body
    }
}

/// Props for [`TimelineItem`].
#[derive(Props, Clone, PartialEq)]
pub struct TimelineItemProps {
    pub step: i32,
    /// Hides the separator. Set on the last item.
    #[props(default)]
    pub last: bool,
    /// Overrides step-based completion.
    #[props(default)]
    pub completed: Option<bool>,
    #[props(default)]
    pub date: Option<String>,
    #[props(default)]
    pub title: Option<String>,
    #[props(default)]
    pub description: Option<String>,
    /// Lucide icon name drawn inside the default indicator.
    #[props(default)]
    pub icon: Option<String>,
    /// Replaces the default indicator inner (icon or empty). The circular
    /// marker still wraps this slot.
    #[props(default)]
    pub indicator: Option<Element>,
    #[props(default = DEFAULT_INDICATOR_SIZE)]
    pub indicator_size: f32,
    #[props(default)]
    pub children: Element,
}

/// One event on the rail. Owns the indicator/separator; children are the
/// content column (typically Header + Content, or the date/title/description
/// shortcuts).
#[component]
pub fn TimelineItem(props: TimelineItemProps) -> Element {
    let theme = use_theme();
    let ctx = try_use_context::<TimelineContext>();
    let active_step = ctx
        .map(|ctx| (ctx.active_step)())
        .unwrap_or(DEFAULT_ACTIVE_STEP);
    let orientation = ctx
        .map(|ctx| (ctx.orientation)())
        .unwrap_or(TimelineOrientation::Vertical);
    let align = ctx.map(|ctx| (ctx.align)()).unwrap_or(TimelineAlign::Right);
    let side = content_side(align, props.step);
    let align_end = text_align_end(orientation, side);
    let mut align_end_signal = use_signal(|| align_end);
    use_effect(use_reactive((&align_end,), move |(align_end,)| {
        if *align_end_signal.peek() != align_end {
            align_end_signal.set(align_end);
        }
    }));
    use_context_provider(|| TimelineItemText {
        align_end: align_end_signal,
    });
    let interactive = ctx.map(|ctx| (ctx.interactive)()).unwrap_or(false);
    let min_width = ctx.and_then(|ctx| (ctx.item_min_width)());
    let axis = item_main_axis(orientation, min_width);
    let completed = item_completed(props.step, active_step, props.completed);
    let rail_complete = separator_completed(props.step, active_step);
    let last = props.last;
    let indicator_size = if props.indicator_size.is_finite() && props.indicator_size > 0.0 {
        props.indicator_size
    } else {
        DEFAULT_INDICATOR_SIZE
    };

    let header = rsx! {
        if props.date.is_some() || props.title.is_some() {
            TimelineHeader {
                if let Some(date) = props.date.clone() {
                    TimelineDate { content: date }
                }
                if let Some(title) = props.title.clone() {
                    TimelineTitle { content: title }
                }
            }
        }
    };
    let details = rsx! {
        if let Some(description) = props.description.clone() {
            TimelineContent { content: description }
        }
        {props.children}
    };

    let body_ref = arkit_hooks::use_native_element_ref();
    let body_height_vp = use_signal(|| 0.0_f32);
    let scale = arkit_hooks::use_window_metrics().scale.max(f32::EPSILON);
    let extent_step = props.step;
    let extent_signal = ctx.map(|ctx| ctx.cross_extents);
    arkit_hooks::use_layout_size(body_ref.clone(), move |size| {
        let mut body_height_vp = body_height_vp;
        let height_vp = size.height / scale;
        if (*body_height_vp.peek() - height_vp).abs() > 0.5 {
            body_height_vp.set(height_vp);
        }
        if let Some(extents) = extent_signal {
            record_item_extent(extents, extent_step, height_vp);
        }
    });

    let select_step = props.step;
    let on_press = EventHandler::new(move |_: ()| {
        let Some(mut ctx) = ctx else {
            return;
        };
        if !(ctx.interactive)() {
            return;
        }
        if !(ctx.controlled)() {
            ctx.active_step.set(select_step);
        }
        (ctx.on_value_change)().call(select_step);
    });

    let line_color = if rail_complete {
        theme.colors.primary
    } else {
        with_alpha(theme.colors.primary, PENDING_SEPARATOR_ALPHA)
    };
    let measured = body_height_vp();
    let axis_extent = ctx.map(|ctx| (ctx.axis_extent)()).unwrap_or(0.0);
    let max_body = ctx
        .map(|ctx| max_item_extent(&(ctx.cross_extents)()))
        .unwrap_or(0.0)
        .max(measured);

    match orientation {
        TimelineOrientation::Vertical => {
            let rail_height = vertical_rail_extent(measured, indicator_size);
            let rail = vertical_rail_stack(
                completed,
                last,
                indicator_size,
                rail_height,
                props.icon.clone(),
                props.indicator,
                line_color,
            );
            let body = rsx! {
                column {
                    native_ref: body_ref,
                    width: "100%",
                    align_items: if align_end { "end" } else { "start" },
                    padding_bottom: if last { 0.0 } else { ITEM_GAP },
                    {header}
                    {details}
                }
            };
            rsx! {
                row {
                    width: "100%",
                    align_items: "start",
                    background_color: if interactive { HIT_FILL } else { 0x0000_0000 },
                    onclick: move |_| on_press.call(()),
                    {vertical_item_slots(align, side, RAIL_CONTENT_GAP, rail, body)}
                }
            }
        }
        TimelineOrientation::Horizontal => {
            let rail = horizontal_rail_stack(
                completed,
                last,
                indicator_size,
                props.icon.clone(),
                props.indicator,
                line_color,
            );
            let toward_rail_top = side == ContentSide::End;
            let toward_rail_bottom = side == ContentSide::Start;
            let body = rsx! {
                column {
                    native_ref: body_ref,
                    width: "100%",
                    align_items: "start",
                    padding_top: if toward_rail_top { spacing::SM } else { 0.0 },
                    padding_bottom: if toward_rail_bottom { spacing::SM } else { 0.0 },
                    {header}
                    {details}
                }
            };
            let item_height = if axis_is_centered(align) {
                if max_body > 1.0 {
                    max_body * 2.0 + indicator_size
                } else {
                    0.0
                }
            } else {
                axis_extent
            };
            let content = horizontal_item_column(align, side, last, item_height > 1.0, rail, body);
            let justify = match (axis_is_centered(align), side) {
                (true, _) => "start",
                (false, ContentSide::Start) => "end",
                (false, ContentSide::End) => "start",
            };
            let width = match axis {
                ItemMainAxis::Fixed(width) => Some(width),
                ItemMainAxis::Flex => None,
            };
            horizontal_item_frame(width, item_height, justify, interactive, on_press, content)
        }
    }
}

fn horizontal_item_frame(
    width: Option<f32>,
    height: f32,
    justify: &'static str,
    interactive: bool,
    on_press: EventHandler<()>,
    content: Element,
) -> Element {
    let bg = if interactive { HIT_FILL } else { 0x0000_0000 };
    let sized = height > 1.0;
    match (width, sized) {
        (Some(width), true) => rsx! {
            column {
                width,
                height,
                align_self: "stretch",
                align_items: "start",
                justify_content: justify,
                background_color: bg,
                onclick: move |_| on_press.call(()),
                {content}
            }
        },
        (Some(width), false) => rsx! {
            column {
                width,
                align_self: "stretch",
                align_items: "start",
                justify_content: justify,
                background_color: bg,
                onclick: move |_| on_press.call(()),
                {content}
            }
        },
        (None, true) => rsx! {
            column {
                layout_weight: 1.0,
                height,
                align_self: "stretch",
                align_items: "start",
                justify_content: justify,
                background_color: bg,
                onclick: move |_| on_press.call(()),
                {content}
            }
        },
        (None, false) => rsx! {
            column {
                layout_weight: 1.0,
                align_self: "stretch",
                align_items: "start",
                justify_content: justify,
                background_color: bg,
                onclick: move |_| on_press.call(()),
                {content}
            }
        },
    }
}

fn layout_strut() -> Element {
    rsx! {
        row {
            width: 1.0,
            height: 1.0,
            hit_test_behavior: "none",
        }
    }
}

fn vertical_item_slots(
    align: TimelineAlign,
    side: ContentSide,
    gap: f32,
    rail: Element,
    body: Element,
) -> Element {
    // Gap lives *inside* the content slot so the rail's x is independent of
    // which side the text is on. Alternate is always [1fr | rail | 1fr].
    if axis_is_centered(align) {
        match side {
            ContentSide::Start => rsx! {
                column {
                    layout_weight: 1.0,
                    align_items: "end",
                    padding_right: gap,
                    {body}
                }
                {rail}
                column {
                    layout_weight: 1.0,
                    {layout_strut()}
                }
            },
            ContentSide::End => rsx! {
                column {
                    layout_weight: 1.0,
                    {layout_strut()}
                }
                {rail}
                column {
                    layout_weight: 1.0,
                    align_items: "start",
                    padding_left: gap,
                    {body}
                }
            },
        }
    } else {
        match side {
            ContentSide::Start => rsx! {
                column {
                    layout_weight: 1.0,
                    align_items: "end",
                    padding_right: gap,
                    {body}
                }
                {rail}
            },
            ContentSide::End => rsx! {
                {rail}
                column {
                    layout_weight: 1.0,
                    align_items: "start",
                    padding_left: gap,
                    {body}
                }
            },
        }
    }
}

fn horizontal_item_column(
    align: TimelineAlign,
    side: ContentSide,
    last: bool,
    equalize: bool,
    rail: Element,
    body: Element,
) -> Element {
    let end_pad = if last { 0.0 } else { spacing::SM };
    let body = rsx! {
        column {
            width: "100%",
            align_items: "start",
            padding_right: end_pad,
            {body}
        }
    };
    if axis_is_centered(align) {
        // Weighted 1fr slots are only safe once the item has a definite height.
        // Otherwise ArkUI treats remaining *page* space as the parent and the
        // items inflate to the viewport.
        let weight = if equalize { 1.0 } else { 0.0 };
        match side {
            ContentSide::Start => rsx! {
                column {
                    width: "100%",
                    layout_weight: weight,
                    justify_content: "end",
                    {body}
                }
                {rail}
                column {
                    width: "100%",
                    layout_weight: weight,
                    {layout_strut()}
                }
            },
            ContentSide::End => rsx! {
                column {
                    width: "100%",
                    layout_weight: weight,
                    {layout_strut()}
                }
                {rail}
                column {
                    width: "100%",
                    layout_weight: weight,
                    {body}
                }
            },
        }
    } else {
        match side {
            ContentSide::Start => rsx! {
                column {
                    width: "100%",
                    align_items: "start",
                    {body}
                }
                {rail}
            },
            ContentSide::End => rsx! {
                {rail}
                column {
                    width: "100%",
                    align_items: "start",
                    {body}
                }
            },
        }
    }
}

fn vertical_rail_stack(
    completed: bool,
    last: bool,
    indicator_size: f32,
    rail_height: f32,
    icon: Option<String>,
    indicator: Option<Element>,
    line_color: u32,
) -> Element {
    let line_height = connector_from_center(rail_height, indicator_size, last);
    rsx! {
        stack {
            width: indicator_size,
            height: rail_height,
            alignment: "top",
            if line_height > 0.0 {
                column {
                    width: SEPARATOR_THICKNESS,
                    height: rail_height,
                    padding_top: indicator_size * 0.5,
                    hit_test_behavior: "transparent",
                    column {
                        layout_weight: 1.0,
                        width: SEPARATOR_THICKNESS,
                        background_color: line_color,
                        hit_test_behavior: "transparent",
                    }
                }
            }
            TimelineIndicator {
                completed: Some(completed),
                size: Some(indicator_size),
                icon,
                {indicator}
            }
        }
    }
}

fn horizontal_rail_stack(
    completed: bool,
    last: bool,
    indicator_size: f32,
    icon: Option<String>,
    indicator: Option<Element>,
    line_color: u32,
) -> Element {
    rsx! {
        stack {
            width: "100%",
            height: indicator_size,
            alignment: "center_start",
            if !last {
                row {
                    width: "100%",
                    height: SEPARATOR_THICKNESS,
                    padding_left: indicator_size * 0.5,
                    hit_test_behavior: "transparent",
                    column {
                        layout_weight: 1.0,
                        height: SEPARATOR_THICKNESS,
                        background_color: line_color,
                        hit_test_behavior: "transparent",
                    }
                }
            }
            TimelineIndicator {
                completed: Some(completed),
                size: Some(indicator_size),
                icon,
                {indicator}
            }
        }
    }
}

/// Props for [`TimelineHeader`].
#[derive(Props, Clone, PartialEq)]
pub struct TimelineHeaderProps {
    pub children: Element,
}

/// Stacks date and title at the start of an item.
#[component]
pub fn TimelineHeader(props: TimelineHeaderProps) -> Element {
    let align_end = use_item_text_end();
    rsx! {
        column {
            width: "100%",
            align_items: if align_end { "end" } else { "start" },
            {props.children}
        }
    }
}

/// Props for [`TimelineDate`].
#[derive(Props, Clone, PartialEq)]
pub struct TimelineDateProps {
    pub content: String,
}

/// Small muted timestamp above the title.
#[component]
pub fn TimelineDate(props: TimelineDateProps) -> Element {
    let theme = use_theme();
    let align = item_text_align(use_item_text_end());
    rsx! {
        text {
            content: props.content.clone(),
            width: "100%",
            font_size: typography::XS,
            font_weight: 500,
            font_color: theme.colors.muted_foreground,
            line_height: DATE_LINE_HEIGHT,
            text_align: align,
            margin_bottom: spacing::XXS,
        }
    }
}

/// Props for [`TimelineTitle`].
#[derive(Props, Clone, PartialEq)]
pub struct TimelineTitleProps {
    pub content: String,
}

/// Medium-weight event title.
#[component]
pub fn TimelineTitle(props: TimelineTitleProps) -> Element {
    let theme = use_theme();
    let align = item_text_align(use_item_text_end());
    rsx! {
        text {
            content: props.content.clone(),
            width: "100%",
            font_size: typography::SM,
            font_weight: 500,
            font_color: theme.colors.foreground,
            line_height: TITLE_LINE_HEIGHT,
            text_align: align,
        }
    }
}

/// Props for [`TimelineContent`].
#[derive(Props, Clone, PartialEq)]
pub struct TimelineContentProps {
    #[props(default)]
    pub content: Option<String>,
    #[props(default)]
    pub children: Element,
}

/// Muted supporting copy under the header.
#[component]
pub fn TimelineContent(props: TimelineContentProps) -> Element {
    let theme = use_theme();
    let align_end = use_item_text_end();
    let align = item_text_align(align_end);
    rsx! {
        column {
            width: "100%",
            align_items: if align_end { "end" } else { "start" },
            margin_top: CONTENT_HEADER_GAP,
            if let Some(content) = props.content.clone() {
                text {
                    content,
                    width: "100%",
                    font_size: typography::SM,
                    font_color: theme.colors.muted_foreground,
                    line_height: CONTENT_LINE_HEIGHT,
                    text_align: align,
                }
            }
            {props.children}
        }
    }
}

/// Props for [`TimelineIndicator`].
#[derive(Props, Clone, PartialEq)]
pub struct TimelineIndicatorProps {
    #[props(default)]
    pub completed: Option<bool>,
    #[props(default)]
    pub size: Option<f32>,
    #[props(default)]
    pub icon: Option<String>,
    #[props(default)]
    pub children: Element,
}

/// Circular rail marker. Completed items use a primary border; pending items
/// use `primary/20`.
#[component]
pub fn TimelineIndicator(props: TimelineIndicatorProps) -> Element {
    let theme = use_theme();
    let size = props
        .size
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(DEFAULT_INDICATOR_SIZE);
    let completed = props.completed.unwrap_or(false);
    let border = if completed {
        theme.colors.primary
    } else {
        with_alpha(theme.colors.primary, 0x33)
    };
    let icon_color = if completed {
        theme.colors.primary
    } else {
        theme.colors.muted_foreground
    };
    let icon_size = indicator_icon_size(size);

    rsx! {
        stack {
            width: size,
            height: size,
            alignment: "center",
            border_radius: theme.radii.full,
            border_width: INDICATOR_BORDER_WIDTH,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_color: border,
            background_color: theme.colors.background,
            clip: true,
            if let Some(name) = props.icon.as_ref() {
                {crate::icon::icon_placeholder(name, icon_size, icon_color)}
            }
            {props.children}
        }
    }
}

/// Props for [`TimelineSeparator`].
#[derive(Props, Clone, PartialEq)]
pub struct TimelineSeparatorProps {
    #[props(default)]
    pub completed: Option<bool>,
    #[props(default)]
    pub orientation: Option<TimelineOrientation>,
}

/// Connecting line between items. Hidden by [`TimelineItem`] when `last`.
#[component]
pub fn TimelineSeparator(props: TimelineSeparatorProps) -> Element {
    let theme = use_theme();
    let ctx = try_use_context::<TimelineContext>();
    let completed = props.completed.unwrap_or(false);
    let color = if completed {
        theme.colors.primary
    } else {
        with_alpha(theme.colors.primary, PENDING_SEPARATOR_ALPHA)
    };
    let orientation = props.orientation.unwrap_or_else(|| {
        ctx.map(|ctx| (ctx.orientation)())
            .unwrap_or(TimelineOrientation::Vertical)
    });

    match orientation {
        TimelineOrientation::Vertical => rsx! {
            column {
                width: SEPARATOR_THICKNESS,
                height: ITEM_GAP,
                margin_top: SEPARATOR_INSET,
                background_color: color,
                hit_test_behavior: "transparent",
            }
        },
        TimelineOrientation::Horizontal => rsx! {
            row {
                width: "100%",
                height: SEPARATOR_THICKNESS,
                background_color: color,
                hit_test_behavior: "transparent",
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        axis_is_centered, connector_from_center, content_side, indicator_icon_size, item_completed,
        item_main_axis, max_item_extent, separator_completed, text_align_end,
        timeline_scrolls_horizontally, vertical_rail_extent, ContentSide, ItemMainAxis,
        TimelineAlign, TimelineOrientation, DEFAULT_INDICATOR_SIZE,
    };

    #[test]
    fn completion_follows_active_step_unless_overridden() {
        assert!(item_completed(1, 2, None));
        assert!(item_completed(2, 2, None));
        assert!(!item_completed(3, 2, None));
        assert!(item_completed(9, 1, Some(true)));
        assert!(!item_completed(1, 4, Some(false)));
    }

    #[test]
    fn separator_is_primary_only_when_the_next_step_is_completed() {
        assert!(separator_completed(1, 2));
        assert!(!separator_completed(2, 2));
        assert!(!separator_completed(3, 2));
        assert!(!separator_completed(1, 1));
        assert!(!separator_completed(1, 0));
    }

    #[test]
    fn align_places_content_left_right_or_alternating() {
        assert_eq!(content_side(TimelineAlign::Right, 1), ContentSide::End);
        assert_eq!(content_side(TimelineAlign::Left, 1), ContentSide::Start);
        assert_eq!(content_side(TimelineAlign::Alternate, 1), ContentSide::End);
        assert_eq!(
            content_side(TimelineAlign::Alternate, 2),
            ContentSide::Start
        );
        assert_eq!(content_side(TimelineAlign::Alternate, 3), ContentSide::End);
        assert!(text_align_end(
            TimelineOrientation::Vertical,
            ContentSide::Start
        ));
        assert!(!text_align_end(
            TimelineOrientation::Vertical,
            ContentSide::End
        ));
        assert!(!text_align_end(
            TimelineOrientation::Horizontal,
            ContentSide::Start
        ));
    }

    #[test]
    fn indicator_icon_scales_inside_the_marker() {
        assert_eq!(indicator_icon_size(DEFAULT_INDICATOR_SIZE), 10.0);
        assert_eq!(indicator_icon_size(32.0), 20.0);
        assert_eq!(indicator_icon_size(8.0), 8.0);
    }

    #[test]
    fn alternate_centers_the_axis() {
        assert!(axis_is_centered(TimelineAlign::Alternate));
        assert!(!axis_is_centered(TimelineAlign::Right));
        assert!(!axis_is_centered(TimelineAlign::Left));
    }

    #[test]
    fn vertical_connector_runs_from_indicator_center_to_item_end() {
        assert_eq!(vertical_rail_extent(0.0, 16.0), 16.0);
        assert_eq!(vertical_rail_extent(80.0, 16.0), 80.0);
        assert_eq!(connector_from_center(80.0, 16.0, false), 72.0);
        assert_eq!(connector_from_center(80.0, 16.0, true), 0.0);
        assert_eq!(connector_from_center(8.0, 16.0, false), 0.0);
    }

    #[test]
    fn shared_cross_extent_is_the_tallest_item_body() {
        assert_eq!(max_item_extent(&[]), 0.0);
        assert_eq!(max_item_extent(&[(1, 40.0), (2, 72.0), (3, 55.0)]), 72.0);
    }

    #[test]
    fn horizontal_min_width_fixes_item_size_and_enables_scroll() {
        assert_eq!(
            item_main_axis(TimelineOrientation::Horizontal, Some(132.0)),
            ItemMainAxis::Fixed(132.0)
        );
        assert_eq!(
            item_main_axis(TimelineOrientation::Horizontal, None),
            ItemMainAxis::Flex
        );
        assert_eq!(
            item_main_axis(TimelineOrientation::Vertical, Some(132.0)),
            ItemMainAxis::Flex
        );
        assert!(timeline_scrolls_horizontally(
            TimelineOrientation::Horizontal,
            Some(132.0)
        ));
        assert!(!timeline_scrolls_horizontally(
            TimelineOrientation::Horizontal,
            None
        ));
        assert!(!timeline_scrolls_horizontally(
            TimelineOrientation::Vertical,
            Some(132.0)
        ));
        assert!(!timeline_scrolls_horizontally(
            TimelineOrientation::Horizontal,
            Some(0.0)
        ));
    }
}
