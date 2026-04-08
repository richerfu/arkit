use super::*;
use arkit::ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit::{LayoutFrame, LayoutSize, NativeOverlayPlacement};
use arkit_widget::{
    anchored_overlay, native_overlay, observe_layout_frame, observe_layout_frame_enabled,
    observe_layout_size, on_cleanup,
};
use ohos_display_binding::default_display_virtual_pixel_ratio;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const FLOATING_SIDE_OFFSET_VP: f32 = spacing::XXS;
const FLOATING_LAYOUT_EPSILON: f32 = 0.5;
const FLOATING_HIDDEN_POSITION_VP: f32 = -10_000.0;
const TRANSPARENT: u32 = 0x00000000;
const HIT_TEST_TRANSPARENT: i32 = 2;
const WRAP_CONTENT_POLICY: i32 = 1;

#[derive(Clone)]
struct Shared<T>(Rc<RefCell<T>>);

impl<T> Shared<T> {
    fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    fn set(&self, value: T) {
        self.0.replace(value);
    }
}

impl<T: Clone> Shared<T> {
    fn get(&self) -> T {
        self.0.borrow().clone()
    }
}

impl<T: Clone + PartialEq> Shared<T> {
    fn set_if_changed(&self, value: T) -> bool {
        let mut current = self.0.borrow_mut();
        if *current == value {
            return false;
        }
        *current = value;
        true
    }
}

fn request_runtime_rerender() {
    arkit_widget::queue_ui_loop(|| {
        if let Some(runtime) = arkit_widget::current_runtime() {
            let _ = runtime.request_rerender();
        }
    });
}

fn schedule_floating_sync(fs: Rc<FloatingState>) {
    arkit_widget::queue_ui_loop(move || apply_floating_position(&fs));
}

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
    Right,
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
    let [x, y] = match side {
        FloatingSide::Right => [trigger.x + trigger.width + side_offset, trigger.y],
        FloatingSide::Top => [x, trigger.y - panel.height - side_offset],
        FloatingSide::Bottom => [x, trigger.y + trigger.height + side_offset],
    };

    [x, y]
}

#[derive(Clone)]
pub(crate) struct FloatingSurfaceRegistry {
    next_id: Rc<Cell<usize>>,
    surfaces: Rc<RefCell<Vec<(usize, Shared<LayoutFrame>)>>>,
}

impl FloatingSurfaceRegistry {
    pub(crate) fn new() -> Self {
        Self {
            next_id: Rc::new(Cell::new(1)),
            surfaces: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(crate) fn same_instance(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.surfaces, &other.surfaces)
    }

    fn register_surface(&self) -> FloatingSurfaceHandle {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        let frame = Shared::new(LayoutFrame::default());
        self.surfaces.borrow_mut().push((id, frame.clone()));
        FloatingSurfaceHandle {
            registry: self.clone(),
            id,
            frame,
        }
    }

    fn unregister(&self, id: usize) {
        self.surfaces
            .borrow_mut()
            .retain(|(current, _)| *current != id);
    }

    fn contains_point(&self, x: f32, y: f32) -> bool {
        self.surfaces.borrow().iter().any(|(_, frame)| {
            let frame = frame.get();
            frame.is_measured()
                && x >= frame.x
                && x <= frame.x + frame.width
                && y >= frame.y
                && y <= frame.y + frame.height
        })
    }
}

#[derive(Clone)]
struct FloatingSurfaceHandle {
    registry: FloatingSurfaceRegistry,
    id: usize,
    frame: Shared<LayoutFrame>,
}

impl FloatingSurfaceHandle {
    fn set(&self, frame: LayoutFrame) {
        self.frame.set(frame);
    }

    fn cleanup(&self) {
        self.frame.set(LayoutFrame::default());
        self.registry.unregister(self.id);
    }
}

// ---------------------------------------------------------------------------
// Shared imperative state for floating position updates.
// Layout observers write into these shared cells. Position and hit testing are
// updated imperatively via direct ArkUI-node mutation.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct FloatingNodes {
    position_column: Rc<RefCell<Option<ArkUINode>>>,
    backdrop_stack: Rc<RefCell<Option<ArkUINode>>>,
}

#[derive(Clone)]
struct FloatingState {
    trigger_frame: Shared<LayoutFrame>,
    panel_size: Shared<LayoutSize>,
    container_offset: Shared<LayoutFrame>,
    props: Shared<FloatingRenderProps>,
    nodes: FloatingNodes,
    surface_handles: Rc<Vec<FloatingSurfaceHandle>>,
}

#[derive(Clone)]
struct FloatingRenderProps {
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    has_dismiss: bool,
    dismiss_registry: Option<FloatingSurfaceRegistry>,
}

impl FloatingState {
    fn new(surface_handles: Vec<FloatingSurfaceHandle>) -> Self {
        Self {
            trigger_frame: Shared::new(LayoutFrame::default()),
            panel_size: Shared::new(LayoutSize::default()),
            container_offset: Shared::new(LayoutFrame::default()),
            props: Shared::new(FloatingRenderProps {
                open: false,
                side: FloatingSide::Bottom,
                align: FloatingAlign::Center,
                has_dismiss: false,
                dismiss_registry: None,
            }),
            nodes: FloatingNodes {
                position_column: Rc::new(RefCell::new(None)),
                backdrop_stack: Rc::new(RefCell::new(None)),
            },
            surface_handles: Rc::new(surface_handles),
        }
    }
    fn cleanup(&self) {
        for handle in self.surface_handles.iter() {
            handle.cleanup();
        }
        self.nodes.position_column.replace(None);
        self.nodes.backdrop_stack.replace(None);
    }
}

#[derive(Clone)]
struct FloatingPanelState(Rc<FloatingState>);

fn use_floating_state(register_surfaces: Vec<FloatingSurfaceRegistry>) -> Rc<FloatingState> {
    if let Some(state) = arkit_widget::use_local_context::<FloatingPanelState>() {
        return state.0;
    }

    let state = Rc::new(FloatingState::new(
        register_surfaces
            .into_iter()
            .map(|registry| registry.register_surface())
            .collect(),
    ));
    arkit_widget::provide_context(FloatingPanelState(state.clone()));
    on_cleanup({
        let state = state.clone();
        move || state.cleanup()
    });
    state
}

fn apply_floating_position(fs: &FloatingState) {
    let props = fs.props.get();
    let trigger = fs.trigger_frame.get();
    let panel_size = fs.panel_size.get();
    let container = fs.container_offset.get();
    let ready =
        props.open && trigger.is_measured() && panel_size.is_measured() && container.is_measured();
    let mut surface_frame = LayoutFrame::default();

    if let Some(col) = fs.nodes.position_column.borrow().as_ref() {
        let position: ArkUINodeAttributeItem = if ready {
            let [px_x, px_y] = floating_position(trigger, panel_size, props.side, props.align);
            surface_frame = LayoutFrame {
                x: px_x,
                y: px_y,
                width: panel_size.width,
                height: panel_size.height,
            };
            vec![px_to_vp(px_x - container.x), px_to_vp(px_y - container.y)].into()
        } else {
            vec![FLOATING_HIDDEN_POSITION_VP, FLOATING_HIDDEN_POSITION_VP].into()
        };
        let _ = col.set_attribute(ArkUINodeAttributeType::Position, position);
        let _ = col.opacity(if ready { 1.0 } else { 0.0 });
        let _ = col.set_hit_test_behavior(if ready { 0_i32 } else { HIT_TEST_TRANSPARENT });
    }

    if let Some(stack) = fs.nodes.backdrop_stack.borrow().as_ref() {
        let backdrop_active = ready && props.has_dismiss;
        let hit = if backdrop_active {
            0_i32
        } else {
            HIT_TEST_TRANSPARENT
        };
        let _ = stack.set_hit_test_behavior(hit);
    }

    for handle in fs.surface_handles.iter() {
        handle.set(surface_frame);
    }
}

// ---------------------------------------------------------------------------
// Portal layer element (created once, position updated imperatively)
// ---------------------------------------------------------------------------

fn floating_portal_layer<Message: 'static>(
    panel: Element<Message>,
    fs: &Rc<FloatingState>,
    _pass_through_dismiss: bool,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message> {
    let position_node = fs.nodes.position_column.clone();
    let backdrop_node = fs.nodes.backdrop_stack.clone();
    let sync_backdrop = Rc::new({
        let fs = fs.clone();
        move || apply_floating_position(&fs)
    });

    let layer_sync_backdrop = sync_backdrop.clone();
    let mut layer_stack = arkit::stack_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .percent_height(1.0)
        .style(ArkUINodeAttributeType::Clip, false)
        .style(
            ArkUINodeAttributeType::HitTestBehavior,
            HIT_TEST_TRANSPARENT,
        )
        .native(move |node| {
            node.set_alignment(i32::from(Alignment::TopStart))?;
            layer_sync_backdrop();
            Ok(())
        });

    // Split dismiss ownership based on mode:
    // - Pass-through: touch intercept handles dismiss, no backdrop needed
    // - Backdrop: backdrop click handles dismiss, no touch intercept needed
    let mut children = Vec::new();

    if let Some(dismiss) = on_dismiss {
        let click_backdrop = backdrop_node.clone();
        let click_position = position_node.clone();
        let click_fs = fs.clone();
        children.push(
            arkit::row_component::<Message, arkit::Theme>()
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
                    if click_fs.props.get().open {
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
        arkit::column_component::<Message, arkit::Theme>()
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
                move |node| {
                    position_node.replace(Some(node.borrow_mut().clone()));
                    apply_floating_position(&fs);
                    Ok(())
                }
            })
            .children(vec![observe_layout_size(panel, {
                let fs = fs.clone();
                move |size| {
                    fs.panel_size.set(size);
                    apply_floating_position(&fs);
                }
            })
            .into()])
            .into(),
    );

    let layer = layer_stack.children(children).into();

    observe_layout_frame(layer, {
        let fs = fs.clone();
        move |frame| {
            fs.container_offset.set(frame);
            apply_floating_position(&fs);
        }
    })
}

#[allow(dead_code)]
fn native_floating_placement(side: FloatingSide, align: FloatingAlign) -> NativeOverlayPlacement {
    let alignment = match (side, align) {
        (FloatingSide::Right, FloatingAlign::Start) => Alignment::End,
        (FloatingSide::Right, FloatingAlign::Center) => Alignment::End,
        (FloatingSide::Top, FloatingAlign::Start) => Alignment::TopStart,
        (FloatingSide::Top, FloatingAlign::Center) => Alignment::Top,
        (FloatingSide::Bottom, FloatingAlign::Start) => Alignment::BottomStart,
        (FloatingSide::Bottom, FloatingAlign::Center) => Alignment::Bottom,
    };
    let (offset_x, offset_y) = match side {
        FloatingSide::Right => (FLOATING_SIDE_OFFSET_VP, 0.0),
        FloatingSide::Top => (0.0, -FLOATING_SIDE_OFFSET_VP),
        FloatingSide::Bottom => (0.0, FLOATING_SIDE_OFFSET_VP),
    };

    NativeOverlayPlacement::new(alignment).with_offset(offset_x, offset_y)
}

fn floating_panel_aligned_scoped<Message: 'static>(
    trigger: Element<Message>,
    panel: Element<Message>,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    on_dismiss: Option<Rc<dyn Fn()>>,
    pass_through_dismiss: bool,
    register_surfaces: Vec<FloatingSurfaceRegistry>,
    dismiss_registry: Option<FloatingSurfaceRegistry>,
) -> Element<Message> {
    let fs = use_floating_state(register_surfaces);
    let has_backdrop = on_dismiss.is_some() && !pass_through_dismiss;
    fs.props.set(FloatingRenderProps {
        open,
        side,
        align,
        has_dismiss: has_backdrop,
        dismiss_registry: dismiss_registry.clone(),
    });
    schedule_floating_sync(fs.clone());

    let observed_trigger = observe_layout_frame_enabled(trigger, true, {
        let fs = fs.clone();
        move |frame| {
            fs.trigger_frame.set(frame);
            apply_floating_position(&fs);
        }
    });

    anchored_overlay(
        observed_trigger,
        Some(floating_portal_layer(
            panel,
            &fs,
            pass_through_dismiss,
            on_dismiss,
        )),
    )
}

pub(crate) fn floating_panel_aligned<Message: 'static>(
    trigger: Element<Message>,
    panel: Element<Message>,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    on_dismiss: Option<Rc<dyn Fn()>>,
    pass_through_dismiss: bool,
    register_surfaces: Vec<FloatingSurfaceRegistry>,
    dismiss_registry: Option<FloatingSurfaceRegistry>,
) -> Element<Message> {
    arkit_widget::scope(move || {
        floating_panel_aligned_scoped(
            trigger,
            panel,
            open,
            side,
            align,
            on_dismiss,
            pass_through_dismiss,
            register_surfaces,
            dismiss_registry,
        )
    })
}

#[allow(dead_code)]
pub(crate) fn floating_panel<Message>(
    trigger: Element<Message>,
    panel: Element<Message>,
    open: bool,
    side: FloatingSide,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message>
where
    Message: 'static,
{
    floating_panel_aligned(
        trigger,
        panel,
        open,
        side,
        FloatingAlign::Center,
        on_dismiss,
        false,
        vec![],
        None,
    )
}

#[allow(dead_code)]
pub(crate) fn native_floating_panel<Message>(
    trigger: Element<Message>,
    panel: Element<Message>,
    open: bool,
    side: FloatingSide,
) -> Element<Message>
where
    Message: 'static,
{
    native_floating_panel_aligned(trigger, panel, open, side, FloatingAlign::Center)
}

#[allow(dead_code)]
pub(crate) fn native_floating_panel_aligned<Message>(
    trigger: Element<Message>,
    panel: Element<Message>,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
) -> Element<Message>
where
    Message: 'static,
{
    native_overlay(
        trigger,
        if open { Some(panel) } else { None },
        native_floating_placement(side, align),
    )
}

fn floating_panel_with_builder_scoped<Message: 'static>(
    trigger: Element<Message>,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    panel_builder: Rc<dyn Fn(Option<f32>) -> Element<Message>>,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message> {
    let fs = use_floating_state(Vec::<FloatingSurfaceRegistry>::new());
    let has_dismiss = on_dismiss.is_some();
    fs.props.set(FloatingRenderProps {
        open,
        side,
        align,
        has_dismiss,
        dismiss_registry: None,
    });
    schedule_floating_sync(fs.clone());

    let observed_trigger = observe_layout_frame_enabled(trigger, true, {
        let fs = fs.clone();
        move |frame| {
            let changed = fs.trigger_frame.set_if_changed(frame);
            apply_floating_position(&fs);
            if changed {
                request_runtime_rerender();
            }
        }
    });
    let tf = fs.trigger_frame.get();
    let trigger_width = if tf.width > FLOATING_LAYOUT_EPSILON {
        Some(px_to_vp(tf.width))
    } else {
        None
    };
    let panel = panel_builder(trigger_width);

    anchored_overlay(
        observed_trigger,
        Some(floating_portal_layer(panel, &fs, false, on_dismiss)),
    )
}

pub(crate) fn floating_panel_with_builder<Message: 'static>(
    trigger: Element<Message>,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    panel_builder: Rc<dyn Fn(Option<f32>) -> Element<Message>>,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message> {
    arkit_widget::scope(move || {
        floating_panel_with_builder_scoped(trigger, open, side, align, panel_builder, on_dismiss)
    })
}
