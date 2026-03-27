use super::*;
use arkit::ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute,
};
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit::{
    anchored_overlay, component, create_signal, native_overlay, observe_layout_frame,
    observe_layout_frame_enabled, observe_layout_size, on_cleanup, LayoutFrame, LayoutSize,
    NativeOverlayPlacement,
};
use ohos_display_binding::default_display_virtual_pixel_ratio;
use std::cell::RefCell;
use std::rc::Rc;

const FLOATING_SIDE_OFFSET_VP: f32 = spacing::XXS;
const FLOATING_LAYOUT_EPSILON: f32 = 0.5;
const FLOATING_HIDDEN_POSITION_VP: f32 = -10_000.0;
const HIT_TEST_TRANSPARENT: i32 = 2;
const WRAP_CONTENT_POLICY: i32 = 1;

fn px_to_vp(value: f32) -> f32 {
    let ratio = default_display_virtual_pixel_ratio();
    if ratio.is_finite() && ratio > 0.0 {
        value / ratio
    } else {
        value
    }
}

fn vp_to_px(value: f32) -> f32 {
    let ratio = default_display_virtual_pixel_ratio();
    if ratio.is_finite() && ratio > 0.0 {
        value * ratio
    } else {
        value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FloatingSide {
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FloatingAlign {
    Start,
    Center,
}

fn floating_position(
    trigger: LayoutFrame,
    panel: LayoutSize,
    side: FloatingSide,
    align: FloatingAlign,
) -> [f32; 2] {
    let side_offset = vp_to_px(FLOATING_SIDE_OFFSET_VP);
    let x = match align {
        FloatingAlign::Start => trigger.x,
        FloatingAlign::Center => trigger.x + ((trigger.width - panel.width) / 2.0),
    };
    let y = match side {
        FloatingSide::Top => trigger.y - panel.height - side_offset,
        FloatingSide::Bottom => trigger.y + trigger.height + side_offset,
    };

    [x, y]
}

// ---------------------------------------------------------------------------
// Shared imperative state for floating position updates.
//
// Layout‑observer signals are created with `arkit::signal` (standalone) so
// that writing to them does NOT schedule a parent‑scope re‑render.  Position
// and visibility are updated imperatively via direct ArkUI‑node manipulation
// triggered by signal subscriptions.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct FloatingNodes {
    position_column: Rc<RefCell<Option<ArkUINode>>>,
    backdrop_stack: Rc<RefCell<Option<ArkUINode>>>,
}

#[derive(Clone)]
struct FloatingSignals {
    trigger_frame: arkit::Signal<LayoutFrame>,
    panel_size: arkit::Signal<LayoutSize>,
    container_offset: arkit::Signal<LayoutFrame>,
    nodes: FloatingNodes,
}

impl FloatingSignals {
    fn new() -> Self {
        Self {
            trigger_frame: arkit::signal(LayoutFrame::default()),
            panel_size: arkit::signal(LayoutSize::default()),
            container_offset: arkit::signal(LayoutFrame::default()),
            nodes: FloatingNodes {
                position_column: Rc::new(RefCell::new(None)),
                backdrop_stack: Rc::new(RefCell::new(None)),
            },
        }
    }
}

fn apply_floating_position(
    fs: &FloatingSignals,
    side: FloatingSide,
    align: FloatingAlign,
    has_dismiss: bool,
) {
    let trigger = fs.trigger_frame.get();
    let panel_size = fs.panel_size.get();
    let container = fs.container_offset.get();
    let ready =
        trigger.is_measured() && panel_size.is_measured() && container.is_measured();

    if let Some(col) = fs.nodes.position_column.borrow().as_ref() {
        let position: ArkUINodeAttributeItem = if ready {
            let [px_x, px_y] = floating_position(trigger, panel_size, side, align);
            vec![px_to_vp(px_x - container.x), px_to_vp(px_y - container.y)].into()
        } else {
            vec![FLOATING_HIDDEN_POSITION_VP, FLOATING_HIDDEN_POSITION_VP].into()
        };
        let _ = col.set_attribute(ArkUINodeAttributeType::Position, position);
        let _ = col.opacity(if ready { 1.0 } else { 0.0 });
    }

    if let Some(stack) = fs.nodes.backdrop_stack.borrow().as_ref() {
        let backdrop_active = ready && has_dismiss;
        let hit = if backdrop_active {
            0_i32
        } else {
            HIT_TEST_TRANSPARENT
        };
        let _ = stack.set_hit_test_behavior(hit);
    }
}

fn setup_floating_subscriptions(
    fs: &FloatingSignals,
    side: FloatingSide,
    align: FloatingAlign,
    has_dismiss: bool,
) -> [usize; 3] {
    let updater = {
        let fs = fs.clone();
        Rc::new(move || apply_floating_position(&fs, side, align, has_dismiss))
    };
    let id1 = fs.trigger_frame.subscribe({
        let u = updater.clone();
        move || u()
    });
    let id2 = fs.panel_size.subscribe({
        let u = updater.clone();
        move || u()
    });
    let id3 = fs.container_offset.subscribe({
        let u = updater.clone();
        move || u()
    });
    [id1, id2, id3]
}

// ---------------------------------------------------------------------------
// Portal layer element (created once, position updated imperatively)
// ---------------------------------------------------------------------------

fn floating_portal_layer(
    panel: Element,
    fs: &FloatingSignals,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element {
    let position_node = fs.nodes.position_column.clone();
    let backdrop_node = fs.nodes.backdrop_stack.clone();

    let mut layer_stack = arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .style(ArkUINodeAttributeType::Clip, false)
        .style(ArkUINodeAttributeType::HitTestBehavior, HIT_TEST_TRANSPARENT)
        .native(move |node| {
            backdrop_node.replace(Some(node.borrow_mut().clone()));
            node.set_stack_align_content(i32::from(Alignment::TopStart))
        });

    if let Some(dismiss) = on_dismiss {
        layer_stack = layer_stack.on_click(move || dismiss());
    }

    let layer = layer_stack
        .children(vec![arkit::column_component()
            .style(
                ArkUINodeAttributeType::WidthLayoutpolicy,
                WRAP_CONTENT_POLICY,
            )
            .style(
                ArkUINodeAttributeType::HeightLayoutpolicy,
                WRAP_CONTENT_POLICY,
            )
            .align_items_start()
            .style(
                ArkUINodeAttributeType::Position,
                vec![FLOATING_HIDDEN_POSITION_VP, FLOATING_HIDDEN_POSITION_VP],
            )
            .style(ArkUINodeAttributeType::Opacity, 0.0_f32)
            .style(ArkUINodeAttributeType::ZIndex, 1_i32)
            .native(move |node| {
                position_node.replace(Some(node.borrow_mut().clone()));
                Ok(())
            })
            .children(vec![
                observe_layout_size(panel, fs.panel_size.clone()).into(),
            ])
            .into()])
        .into();

    observe_layout_frame(layer, fs.container_offset.clone())
}

fn floating_trigger_anchor(trigger: Element) -> Element {
    arkit::row_component()
        .align_items_top()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_START)
        .children(vec![trigger])
        .into()
}

#[allow(dead_code)]
fn native_floating_placement(side: FloatingSide, align: FloatingAlign) -> NativeOverlayPlacement {
    let alignment = match (side, align) {
        (FloatingSide::Top, FloatingAlign::Start) => Alignment::TopStart,
        (FloatingSide::Top, FloatingAlign::Center) => Alignment::Top,
        (FloatingSide::Bottom, FloatingAlign::Start) => Alignment::BottomStart,
        (FloatingSide::Bottom, FloatingAlign::Center) => Alignment::Bottom,
    };
    let offset_y = match side {
        FloatingSide::Top => -FLOATING_SIDE_OFFSET_VP,
        FloatingSide::Bottom => FLOATING_SIDE_OFFSET_VP,
    };

    NativeOverlayPlacement::new(alignment).with_offset(0.0, offset_y)
}

#[component]
pub(crate) fn floating_panel_aligned(
    trigger: Element,
    panel: Element,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element {
    // Standalone signals: writing to them does NOT re‑render the parent scope.
    let fs_holder = create_signal(FloatingSignals::new());
    let fs = fs_holder.get();
    let has_dismiss = on_dismiss.is_some();

    // Imperative position subscription (runs once on mount).
    {
        let fs = fs.clone();
        setup_floating_subscriptions(&fs, side, align, has_dismiss);
    }
    on_cleanup({
        let fs = fs.clone();
        move || {
            fs.nodes.position_column.replace(None);
            fs.nodes.backdrop_stack.replace(None);
        }
    });

    let observed_trigger = observe_layout_frame_enabled(
        floating_trigger_anchor(trigger),
        fs.trigger_frame.clone(),
        true,
    );

    anchored_overlay(
        observed_trigger,
        if open {
            // Reset portal-layer signals so that fresh layout measurements are
            // guaranteed to differ from defaults and trigger the subscription
            // callbacks (handles the close → reopen-at-same-position edge case).
            fs.panel_size.set(LayoutSize::default());
            fs.container_offset.set(LayoutFrame::default());
            Some(floating_portal_layer(panel, &fs, on_dismiss))
        } else {
            // Clear stale node refs when the portal is removed.
            fs.nodes.position_column.replace(None);
            fs.nodes.backdrop_stack.replace(None);
            None
        },
    )
}

#[allow(dead_code)]
pub(crate) fn floating_panel(
    trigger: Element,
    panel: Element,
    open: bool,
    side: FloatingSide,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element {
    floating_panel_aligned(trigger, panel, open, side, FloatingAlign::Center, on_dismiss)
}

#[allow(dead_code)]
pub(crate) fn native_floating_panel(
    trigger: Element,
    panel: Element,
    open: bool,
    side: FloatingSide,
) -> Element {
    native_floating_panel_aligned(trigger, panel, open, side, FloatingAlign::Center)
}

#[allow(dead_code)]
pub(crate) fn native_floating_panel_aligned(
    trigger: Element,
    panel: Element,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
) -> Element {
    native_overlay(
        trigger,
        if open { Some(panel) } else { None },
        native_floating_placement(side, align),
    )
}

#[component]
pub(crate) fn floating_panel_with_builder(
    trigger: Element,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    panel_builder: Rc<dyn Fn(Option<f32>) -> Element>,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element {
    let fs_holder = create_signal(FloatingSignals::new());
    let fs = fs_holder.get();
    let has_dismiss = on_dismiss.is_some();

    {
        let fs = fs.clone();
        setup_floating_subscriptions(&fs, side, align, has_dismiss);
    }
    on_cleanup({
        let fs = fs.clone();
        move || {
            fs.nodes.position_column.replace(None);
            fs.nodes.backdrop_stack.replace(None);
        }
    });

    let observed_trigger = observe_layout_frame_enabled(
        floating_trigger_anchor(trigger),
        fs.trigger_frame.clone(),
        true,
    );
    let trigger_width = {
        let tf = fs.trigger_frame.get();
        if tf.width > FLOATING_LAYOUT_EPSILON {
            Some(px_to_vp(tf.width))
        } else {
            None
        }
    };

    anchored_overlay(
        observed_trigger,
        if open {
            fs.panel_size.set(LayoutSize::default());
            fs.container_offset.set(LayoutFrame::default());
            Some(floating_portal_layer(
                panel_builder(trigger_width),
                &fs,
                on_dismiss,
            ))
        } else {
            fs.nodes.position_column.replace(None);
            fs.nodes.backdrop_stack.replace(None);
            None
        },
    )
}
