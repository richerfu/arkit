use super::*;
use arkit::ohos_arkui_binding::arkui_input_binding::UIInputAction;
use arkit::ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUIEvent,
};
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit::{
    anchored_overlay, component, create_effect, create_signal, native_overlay,
    observe_layout_frame, observe_layout_frame_enabled, observe_layout_size, on_cleanup,
    LayoutFrame, LayoutSize, NativeOverlayPlacement, Signal,
};
use ohos_display_binding::default_display_virtual_pixel_ratio;
use std::cell::RefCell;
use std::rc::Rc;

const FLOATING_SIDE_OFFSET_VP: f32 = spacing::XXS;
const FLOATING_LAYOUT_EPSILON: f32 = 0.5;
const FLOATING_HIDDEN_POSITION_VP: f32 = -10_000.0;
const TRANSPARENT: u32 = 0x00000000;
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
// Layout-observer signals are created with `arkit::signal` (standalone) so
// that writing to them does not schedule a parent-scope re-render. Position
// and hit testing are updated imperatively via direct ArkUI-node mutation, but
// the dependency tracking still stays inside the framework effect system.
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
    open: &Signal<bool>,
    side: FloatingSide,
    align: FloatingAlign,
    has_dismiss: bool,
) {
    let trigger = fs.trigger_frame.get();
    let panel_size = fs.panel_size.get();
    let container = fs.container_offset.get();
    let ready =
        open.get() && trigger.is_measured() && panel_size.is_measured() && container.is_measured();

    if let Some(col) = fs.nodes.position_column.borrow().as_ref() {
        let position: ArkUINodeAttributeItem = if ready {
            let [px_x, px_y] = floating_position(trigger, panel_size, side, align);
            vec![px_to_vp(px_x - container.x), px_to_vp(px_y - container.y)].into()
        } else {
            vec![FLOATING_HIDDEN_POSITION_VP, FLOATING_HIDDEN_POSITION_VP].into()
        };
        let _ = col.set_attribute(ArkUINodeAttributeType::Position, position);
        let _ = col.opacity(if ready { 1.0 } else { 0.0 });
        let _ = col.set_hit_test_behavior(if ready { 0_i32 } else { HIT_TEST_TRANSPARENT });
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

fn install_floating_effect(
    fs: &FloatingSignals,
    open: Signal<bool>,
    side: FloatingSide,
    align: FloatingAlign,
    has_dismiss: bool,
) {
    create_effect({
        let fs = fs.clone();
        let open = open.clone();
        move || apply_floating_position(&fs, &open, side, align, has_dismiss)
    });
}

// ---------------------------------------------------------------------------
// Portal layer element (created once, position updated imperatively)
// ---------------------------------------------------------------------------

fn floating_portal_layer(
    panel: Element,
    fs: &FloatingSignals,
    open: Signal<bool>,
    side: FloatingSide,
    align: FloatingAlign,
    on_dismiss: Option<Rc<dyn Fn()>>,
    pass_through_dismiss: bool,
) -> Element {
    let position_node = fs.nodes.position_column.clone();
    let backdrop_node = fs.nodes.backdrop_stack.clone();
    let has_dismiss = on_dismiss.is_some() && !pass_through_dismiss;
    let sync_backdrop = Rc::new({
        let fs = fs.clone();
        let open = open.clone();
        move || apply_floating_position(&fs, &open, side, align, has_dismiss)
    });

    let layer_sync_backdrop = sync_backdrop.clone();
    let mut layer_stack = arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .style(ArkUINodeAttributeType::Clip, false)
        .style(
            ArkUINodeAttributeType::HitTestBehavior,
            HIT_TEST_TRANSPARENT,
        )
        .native(move |node| {
            node.set_stack_align_content(i32::from(Alignment::TopStart))?;
            layer_sync_backdrop();
            Ok(())
        });

    // Split dismiss ownership based on mode:
    // - Pass-through: touch intercept handles dismiss, no backdrop needed
    // - Backdrop: backdrop click handles dismiss, no touch intercept needed
    let mut children = Vec::new();

    if pass_through_dismiss {
        if let Some(dismiss) = on_dismiss {
            let dismiss_open = open.clone();
            let dismiss_fs = fs.clone();
            layer_stack = layer_stack.native(move |node| {
                let dismiss = dismiss.clone();
                let open = dismiss_open.clone();
                let fs = dismiss_fs.clone();
                node.on_touch_intercept(move |event| {
                    if !open.get_untracked() {
                        return Some(false);
                    }
                    let Some(input) = event.input_event() else {
                        return Some(false);
                    };
                    if input.action != UIInputAction::Down {
                        return Some(false);
                    }

                    let touch_x = input.pointer_window_x();
                    let touch_y = input.pointer_window_y();

                    let trigger = fs.trigger_frame.get_untracked();
                    let panel_size = fs.panel_size.get_untracked();
                    let container = fs.container_offset.get_untracked();

                    if !trigger.is_measured()
                        || !panel_size.is_measured()
                        || !container.is_measured()
                    {
                        return Some(false);
                    }

                    let [panel_x, panel_y] =
                        floating_position(trigger, panel_size, side, align);
                    let inside = touch_x >= panel_x
                        && touch_x <= panel_x + panel_size.width
                        && touch_y >= panel_y
                        && touch_y <= panel_y + panel_size.height;

                    if !inside {
                        dismiss();
                    }
                    Some(false) // never intercept — let event propagate
                });
                Ok(())
            });
        }
    } else if let Some(dismiss) = on_dismiss {
        let dismiss_open = open.clone();
        let click_backdrop = backdrop_node.clone();
        let click_position = position_node.clone();
        children.push(
            arkit::row_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .background_color(TRANSPARENT)
                .style(
                    ArkUINodeAttributeType::HitTestBehavior,
                    HIT_TEST_TRANSPARENT,
                )
                .native(move |node| {
                    backdrop_node.replace(Some(node.borrow_mut().clone()));
                    sync_backdrop();
                    Ok(())
                })
                .on_click(move || {
                    if dismiss_open.get() {
                        dismiss();
                        if let Some(node) = click_backdrop.borrow().as_ref() {
                            let _ = node.set_hit_test_behavior(HIT_TEST_TRANSPARENT);
                        }
                        if let Some(node) = click_position.borrow().as_ref() {
                            let _ = node.set_hit_test_behavior(HIT_TEST_TRANSPARENT);
                            let _ = node.opacity(0.0);
                        }
                    }
                })
                .into(),
        );
    }

    children.push(
        arkit::column_component()
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
            .style(
                ArkUINodeAttributeType::HitTestBehavior,
                HIT_TEST_TRANSPARENT,
            )
            .style(ArkUINodeAttributeType::ZIndex, 1_i32)
            .native({
                let fs = fs.clone();
                let open = open.clone();
                move |node| {
                    position_node.replace(Some(node.borrow_mut().clone()));
                    apply_floating_position(&fs, &open, side, align, has_dismiss);
                    Ok(())
                }
            })
            .children(vec![
                observe_layout_size(panel, fs.panel_size.clone()).into()
            ])
            .into(),
    );

    let layer = layer_stack.children(children).into();

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
    open: Signal<bool>,
    side: FloatingSide,
    align: FloatingAlign,
    on_dismiss: Option<Rc<dyn Fn()>>,
    pass_through_dismiss: bool,
) -> Element {
    // Standalone signals: writing to them does NOT re‑render the parent scope.
    let fs_holder = create_signal(FloatingSignals::new());
    let fs = fs_holder.get();
    let has_backdrop = on_dismiss.is_some() && !pass_through_dismiss;
    install_floating_effect(&fs, open.clone(), side, align, has_backdrop);
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
        Some(floating_portal_layer(
            panel,
            &fs,
            open.clone(),
            side,
            align,
            on_dismiss,
            pass_through_dismiss,
        )),
    )
}

#[allow(dead_code)]
pub(crate) fn floating_panel(
    trigger: Element,
    panel: Element,
    open: Signal<bool>,
    side: FloatingSide,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element {
    floating_panel_aligned(
        trigger,
        panel,
        open,
        side,
        FloatingAlign::Center,
        on_dismiss,
        false,
    )
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
    open: Signal<bool>,
    side: FloatingSide,
    align: FloatingAlign,
    panel_builder: Rc<dyn Fn(Option<f32>) -> Element>,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element {
    let fs_holder = create_signal(FloatingSignals::new());
    let fs = fs_holder.get();
    install_floating_effect(&fs, open.clone(), side, align, on_dismiss.is_some());
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
    let panel = arkit::dynamic({
        let fs = fs.clone();
        move || {
            let tf = fs.trigger_frame.get();
            let trigger_width = if tf.width > FLOATING_LAYOUT_EPSILON {
                Some(px_to_vp(tf.width))
            } else {
                None
            };
            panel_builder(trigger_width)
        }
    });

    anchored_overlay(
        observed_trigger,
        Some(floating_portal_layer(
            panel,
            &fs,
            open.clone(),
            side,
            align,
            on_dismiss,
            false,
        )),
    )
}
