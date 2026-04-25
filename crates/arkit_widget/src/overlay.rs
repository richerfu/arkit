use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::advanced;
use crate::internal::{current_runtime, queue_ui_loop};
use crate::render_impl::{
    observe_layout_frame as observe_layout_frame_impl, read_layout_frame, read_layout_size,
};
use crate::{
    column_component, row_component, stack_component, Alignment, ArkUINodeAttributeItem,
    ArkUINodeAttributeType, Direction, Element, HitTestBehavior, NodeEventType,
};
use ohos_arkui_binding::arkui_input_binding::{HitTest, UIInputAction};
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use ohos_display_binding::default_display_virtual_pixel_ratio;

const FLOATING_LAYOUT_EPSILON: f32 = 0.5;
const FLOATING_HIDDEN_POSITION_VP: f32 = -10_000.0;
// wrapContent is constrained by the full-screen portal stack; floating panels
// need their unconstrained ideal size so overlong content does not become viewport-width.
const FIX_AT_IDEAL_SIZE_POLICY: i32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutSize {
    pub fn is_measured(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

impl LayoutFrame {
    pub fn is_measured(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatingSide {
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatingAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayDismissMode {
    None,
    Backdrop,
    PassThrough,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayStrategy {
    Portal,
    Native,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatingOverlaySpec {
    pub open: bool,
    pub side: FloatingSide,
    pub align: FloatingAlign,
    pub offset_vp: f32,
    pub match_trigger_width: bool,
    pub dismiss_mode: OverlayDismissMode,
    pub strategy: OverlayStrategy,
}

impl Default for FloatingOverlaySpec {
    fn default() -> Self {
        Self {
            open: false,
            side: FloatingSide::Bottom,
            align: FloatingAlign::Center,
            offset_vp: 4.0,
            match_trigger_width: false,
            dismiss_mode: OverlayDismissMode::None,
            strategy: OverlayStrategy::Portal,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModalPresentation {
    CenteredDialog,
    RightSheet,
    BottomDrawer,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModalOverlaySpec {
    pub open: bool,
    pub presentation: ModalPresentation,
    pub dismiss_on_backdrop: bool,
    pub backdrop_color: u32,
    pub viewport_inset: f32,
}

impl Default for ModalOverlaySpec {
    fn default() -> Self {
        Self {
            open: false,
            presentation: ModalPresentation::CenteredDialog,
            dismiss_on_backdrop: true,
            backdrop_color: 0x80000000,
            viewport_inset: 16.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeOverlayPlacement {
    pub alignment: Alignment,
    pub offset_x: f32,
    pub offset_y: f32,
    pub direction: Direction,
}

impl NativeOverlayPlacement {
    pub fn new(alignment: Alignment) -> Self {
        Self {
            alignment,
            offset_x: 0.0,
            offset_y: 0.0,
            direction: Direction::Auto,
        }
    }

    pub fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
}

impl Default for NativeOverlayPlacement {
    fn default() -> Self {
        Self::new(Alignment::TopStart)
    }
}

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

#[derive(Clone)]
pub struct FloatingSurfaceRegistry {
    next_id: Rc<Cell<usize>>,
    surfaces: Rc<RefCell<Vec<(usize, Shared<LayoutFrame>)>>>,
}

impl FloatingSurfaceRegistry {
    pub fn new() -> Self {
        Self {
            next_id: Rc::new(Cell::new(1)),
            surfaces: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn same_instance(&self, other: &Self) -> bool {
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

impl Default for FloatingSurfaceRegistry {
    fn default() -> Self {
        Self::new()
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
    spec: Shared<FloatingOverlaySpec>,
    dismiss_registry: Shared<Option<FloatingSurfaceRegistry>>,
    nodes: FloatingNodes,
    surface_handles: Rc<Vec<FloatingSurfaceHandle>>,
}

impl FloatingState {
    fn new(register_surfaces: Vec<FloatingSurfaceRegistry>) -> Self {
        Self {
            trigger_frame: Shared::new(LayoutFrame::default()),
            panel_size: Shared::new(LayoutSize::default()),
            container_offset: Shared::new(LayoutFrame::default()),
            spec: Shared::new(FloatingOverlaySpec::default()),
            dismiss_registry: Shared::new(None),
            nodes: FloatingNodes {
                position_column: Rc::new(RefCell::new(None)),
                backdrop_stack: Rc::new(RefCell::new(None)),
            },
            surface_handles: Rc::new(
                register_surfaces
                    .into_iter()
                    .map(|registry| registry.register_surface())
                    .collect(),
            ),
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

struct FloatingOverlayWidgetState {
    floating: Rc<FloatingState>,
}

impl FloatingOverlayWidgetState {
    fn new(register_surfaces: Vec<FloatingSurfaceRegistry>) -> Self {
        Self {
            floating: Rc::new(FloatingState::new(register_surfaces)),
        }
    }
}

impl Drop for FloatingOverlayWidgetState {
    fn drop(&mut self) {
        self.floating.cleanup();
    }
}

enum FloatingPanelContent<Message, AppTheme = crate::Theme> {
    Static(RefCell<Option<Element<Message, AppTheme>>>),
    Builder(Rc<dyn Fn(Option<f32>) -> Element<Message, AppTheme>>),
}

impl<Message, AppTheme> FloatingPanelContent<Message, AppTheme> {
    fn has_content(&self) -> bool {
        match self {
            Self::Static(panel) => panel.borrow().is_some(),
            Self::Builder(_) => true,
        }
    }

    fn take(&self, trigger_width: Option<f32>) -> Option<Element<Message, AppTheme>> {
        match self {
            Self::Static(panel) => panel.borrow_mut().take(),
            Self::Builder(builder) => Some(builder(trigger_width)),
        }
    }
}

struct FloatingOverlayWidget<Message, AppTheme = crate::Theme> {
    trigger: RefCell<Option<Element<Message, AppTheme>>>,
    panel: FloatingPanelContent<Message, AppTheme>,
    spec: FloatingOverlaySpec,
    on_dismiss: Option<Rc<dyn Fn()>>,
    register_surfaces: Vec<FloatingSurfaceRegistry>,
    dismiss_registry: Option<FloatingSurfaceRegistry>,
}

impl<Message, AppTheme> FloatingOverlayWidget<Message, AppTheme> {
    fn ensure_children(tree: &mut advanced::widget::Tree) {
        let mut children = std::mem::take(tree.children_mut());
        children.truncate(2);
        while children.len() < 2 {
            children.push(advanced::widget::Tree::empty());
        }
        tree.replace_children(children);
    }

    fn state_from_tree(tree: &mut advanced::widget::Tree) -> Rc<FloatingState> {
        tree.state()
            .downcast_mut::<FloatingOverlayWidgetState>()
            .expect("floating overlay widget state type mismatch")
            .floating
            .clone()
    }
}

impl<Message: 'static, AppTheme: 'static> advanced::Widget<Message, AppTheme, crate::Renderer>
    for FloatingOverlayWidget<Message, AppTheme>
{
    fn state(&self) -> advanced::widget::State {
        advanced::widget::State::new(Box::new(FloatingOverlayWidgetState::new(
            self.register_surfaces.clone(),
        )))
    }

    fn children(&self) -> Vec<advanced::widget::Tree> {
        vec![
            advanced::widget::Tree::empty(),
            advanced::widget::Tree::empty(),
        ]
    }

    fn diff(&self, tree: &mut advanced::widget::Tree)
    where
        Self: 'static,
    {
        tree.set_tag(self.tag());
        Self::ensure_children(tree);
    }

    fn body(
        &self,
        tree: &mut advanced::widget::Tree,
        _renderer: &crate::Renderer,
    ) -> Option<Element<Message, AppTheme>> {
        let state = Self::state_from_tree(tree);
        let mut spec = self.spec;
        spec.open = spec.open && self.panel.has_content();
        if matches!(spec.strategy, OverlayStrategy::Native) {
            spec.strategy = OverlayStrategy::Portal;
        }
        state.spec.set(spec);
        state.dismiss_registry.set(self.dismiss_registry.clone());
        schedule_floating_sync(state.clone());

        let should_refresh_width = spec.match_trigger_width && spec.open;
        let trigger = self
            .trigger
            .borrow_mut()
            .take()
            .expect("floating overlay trigger was already consumed");

        Some(observe_layout_frame_impl(trigger, true, {
            let state = state.clone();
            move |frame| {
                let changed = state.trigger_frame.set_if_changed(frame);
                apply_floating_position(&state);
                if changed && should_refresh_width {
                    request_runtime_rerender();
                }
            }
        }))
    }

    fn overlay(
        &self,
        tree: &mut advanced::widget::Tree,
        _renderer: &crate::Renderer,
    ) -> Option<Element<Message, AppTheme>> {
        let state = Self::state_from_tree(tree);
        let spec = state.spec.get();
        if !spec.open {
            return None;
        }

        let trigger_width = if spec.match_trigger_width {
            let trigger_frame = state.trigger_frame.get();
            (trigger_frame.width > FLOATING_LAYOUT_EPSILON).then(|| px_to_vp(trigger_frame.width))
        } else {
            None
        };

        self.panel
            .take(trigger_width)
            .map(|panel| floating_portal_layer(panel, &state, self.on_dismiss.clone()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

struct ModalOverlayWidget<Message, AppTheme = crate::Theme> {
    panel: RefCell<Option<Element<Message, AppTheme>>>,
    spec: ModalOverlaySpec,
    on_dismiss: Option<Rc<dyn Fn()>>,
}

impl<Message: 'static, AppTheme: 'static> advanced::Widget<Message, AppTheme, crate::Renderer>
    for ModalOverlayWidget<Message, AppTheme>
{
    fn children(&self) -> Vec<advanced::widget::Tree> {
        vec![
            advanced::widget::Tree::empty(),
            advanced::widget::Tree::empty(),
        ]
    }

    fn diff(&self, tree: &mut advanced::widget::Tree)
    where
        Self: 'static,
    {
        tree.set_tag(self.tag());
        let mut children = std::mem::take(tree.children_mut());
        children.truncate(2);
        while children.len() < 2 {
            children.push(advanced::widget::Tree::empty());
        }
        tree.replace_children(children);
    }

    fn body(
        &self,
        _tree: &mut advanced::widget::Tree,
        _renderer: &crate::Renderer,
    ) -> Option<Element<Message, AppTheme>> {
        Some(overlay_placeholder())
    }

    fn overlay(
        &self,
        _tree: &mut advanced::widget::Tree,
        _renderer: &crate::Renderer,
    ) -> Option<Element<Message, AppTheme>> {
        if !self.spec.open {
            return None;
        }
        self.panel
            .borrow_mut()
            .take()
            .map(|panel| modal_layer(panel, self.spec, self.on_dismiss.clone()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

fn request_runtime_rerender() {
    queue_ui_loop(|| {
        if let Some(runtime) = current_runtime() {
            let _ = runtime.request_rerender();
        }
    });
}

fn schedule_floating_sync(state: Rc<FloatingState>) {
    queue_ui_loop(move || apply_floating_position(&state));
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

fn floating_position(
    trigger: LayoutFrame,
    panel: LayoutSize,
    container: LayoutFrame,
    side: FloatingSide,
    align: FloatingAlign,
    offset_vp: f32,
) -> [f32; 2] {
    let side_offset = vp_to_px(offset_vp);
    let aligned_x = match align {
        FloatingAlign::Start => trigger.x,
        FloatingAlign::Center => trigger.x + ((trigger.width - panel.width) / 2.0),
        FloatingAlign::End => trigger.x + trigger.width - panel.width,
    };
    let [x, y] = match side {
        FloatingSide::Right => [trigger.x + trigger.width + side_offset, trigger.y],
        FloatingSide::Top => [aligned_x, trigger.y - panel.height - side_offset],
        FloatingSide::Bottom => [aligned_x, trigger.y + trigger.height + side_offset],
    };

    [
        clamp_floating_axis(x, container.x, container.x + container.width - panel.width),
        clamp_floating_axis(
            y,
            container.y,
            container.y + container.height - panel.height,
        ),
    ]
}

fn clamp_floating_axis(value: f32, min: f32, max: f32) -> f32 {
    if max < min {
        min
    } else {
        value.clamp(min, max)
    }
}

fn apply_floating_position(state: &FloatingState) {
    let spec = state.spec.get();
    let trigger = state.trigger_frame.get();
    let panel_size = state.panel_size.get();
    let container = state.container_offset.get();
    let ready =
        spec.open && trigger.is_measured() && panel_size.is_measured() && container.is_measured();
    let mut surface_frame = LayoutFrame::default();

    if let Some(column) = state.nodes.position_column.borrow().as_ref() {
        let position: ArkUINodeAttributeItem = if ready {
            let [px_x, px_y] = floating_position(
                trigger,
                panel_size,
                container,
                spec.side,
                spec.align,
                spec.offset_vp,
            );
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
        let _ = column.set_attribute(ArkUINodeAttributeType::Position, position);
        let _ = column.opacity(if ready { 1.0 } else { 0.0 });
        let _ = column.set_hit_test_behavior(i32::from(if ready {
            HitTestBehavior::Default
        } else {
            HitTestBehavior::Transparent
        }));
    }

    if let Some(stack) = state.nodes.backdrop_stack.borrow().as_ref() {
        let backdrop_active = ready && matches!(spec.dismiss_mode, OverlayDismissMode::Backdrop);
        let _ = stack.set_hit_test_behavior(i32::from(if backdrop_active {
            HitTestBehavior::Default
        } else {
            HitTestBehavior::Transparent
        }));
    }

    for handle in state.surface_handles.iter() {
        handle.set(surface_frame);
    }
}

fn overlay_placeholder<Message, AppTheme>() -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    row_component::<Message, AppTheme>()
        .width(0.0)
        .height(0.0)
        .hit_test_behavior(HitTestBehavior::Transparent)
        .attr(ArkUINodeAttributeType::Opacity, 0.0_f32)
        .into()
}

fn floating_portal_layer<Message, AppTheme>(
    panel: Element<Message, AppTheme>,
    state: &Rc<FloatingState>,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let position_node = state.nodes.position_column.clone();
    let backdrop_node = state.nodes.backdrop_stack.clone();
    let sync = Rc::new({
        let state = state.clone();
        move || apply_floating_position(&state)
    });

    let mut children = Vec::new();

    if matches!(state.spec.get().dismiss_mode, OverlayDismissMode::Backdrop) {
        let backdrop_sync = sync.clone();
        let click_backdrop = backdrop_node.clone();
        let click_position = position_node.clone();
        let click_state = state.clone();
        children.push(
            row_component::<Message, AppTheme>()
                .percent_width(1.0)
                .percent_height(1.0)
                .background_color(0x01000000)
                .hit_test_behavior(HitTestBehavior::Default)
                .with_patch(move |node| {
                    backdrop_node.replace(Some(node.borrow_mut().clone()));
                    backdrop_sync();
                    Ok(())
                })
                .on_event(crate::NodeEventType::TouchEvent, move |event| {
                    if click_state.spec.get().open {
                        let Some(input_event) = event.input_event() else {
                            return;
                        };
                        let _ = input_event.pointer_set_stop_propagation(true);
                        let _ = input_event.pointer_set_intercept_hit_test_mode(HitTest::Default);
                        if !matches!(
                            input_event.action,
                            UIInputAction::Up | UIInputAction::Cancel
                        ) {
                            return;
                        }
                        if click_state
                            .dismiss_registry
                            .get()
                            .map(|registry| {
                                registry.contains_point(
                                    input_event.pointer_window_x(),
                                    input_event.pointer_window_y(),
                                )
                            })
                            .unwrap_or(false)
                        {
                            return;
                        }
                        if let Some(dismiss) = on_dismiss.as_ref() {
                            dismiss();
                        }
                        if let Some(node) = click_backdrop.borrow().as_ref() {
                            let _ =
                                node.set_hit_test_behavior(i32::from(HitTestBehavior::Transparent));
                        }
                        if let Some(node) = click_position.borrow().as_ref() {
                            let _ =
                                node.set_hit_test_behavior(i32::from(HitTestBehavior::Transparent));
                            let _ = node.opacity(0.0);
                        }
                    }
                })
                .into(),
        );
    }

    // Inline observation logic instead of using observe_layout_size_impl /
    // observe_layout_frame_impl here.  Those helpers call into_node() which
    // pre-compiles the element tree and discards any Widget overlays collected
    // during compilation.  The panel may contain MenuSubmenuWidget (or other
    // composite widgets that produce overlays), so keeping the panel as an
    // uncompiled Element child lets compile_node preserve those overlays.
    let panel_size_node_ref = Rc::new(RefCell::new(None::<ArkUINode>));
    let panel_size_on_change: Rc<dyn Fn(LayoutSize)> = Rc::new({
        let state = state.clone();
        move |size| {
            state.panel_size.set(size);
            apply_floating_position(&state);
        }
    });

    children.push(
        column_component::<Message, AppTheme>()
            .attr(
                ArkUINodeAttributeType::WidthLayoutpolicy,
                FIX_AT_IDEAL_SIZE_POLICY,
            )
            .attr(
                ArkUINodeAttributeType::HeightLayoutpolicy,
                FIX_AT_IDEAL_SIZE_POLICY,
            )
            .attr(
                ArkUINodeAttributeType::Position,
                vec![FLOATING_HIDDEN_POSITION_VP, FLOATING_HIDDEN_POSITION_VP],
            )
            .attr(ArkUINodeAttributeType::Opacity, 0.0_f32)
            .hit_test_behavior(HitTestBehavior::Default)
            .attr(ArkUINodeAttributeType::ZIndex, 1_i32)
            .with_patch({
                let state = state.clone();
                move |node| {
                    position_node.replace(Some(node.borrow_mut().clone()));
                    apply_floating_position(&state);
                    Ok(())
                }
            })
            .children(vec![stack_component::<Message, AppTheme>()
                .attr(
                    ArkUINodeAttributeType::WidthLayoutpolicy,
                    FIX_AT_IDEAL_SIZE_POLICY,
                )
                .attr(
                    ArkUINodeAttributeType::HeightLayoutpolicy,
                    FIX_AT_IDEAL_SIZE_POLICY,
                )
                .attr(ArkUINodeAttributeType::Clip, false)
                .hit_test_behavior(HitTestBehavior::Default)
                .with_patch({
                    let node_ref = panel_size_node_ref.clone();
                    let on_change = panel_size_on_change.clone();
                    move |node| {
                        let runtime = node.borrow_mut().clone();
                        if let Some(size) = read_layout_size(&runtime) {
                            on_change(size);
                        }
                        node_ref.replace(Some(runtime));
                        Ok(())
                    }
                })
                .on_event_no_param(NodeEventType::EventOnAreaChange, {
                    let node_ref = panel_size_node_ref;
                    let on_change = panel_size_on_change;
                    move || {
                        if let Some(node) = node_ref.borrow().as_ref() {
                            if let Some(size) = read_layout_size(node) {
                                on_change(size);
                            }
                        }
                    }
                })
                .on_event(NodeEventType::TouchEvent, move |event| {
                    if let Some(input_event) = event.input_event() {
                        let _ = input_event.pointer_set_stop_propagation(true);
                        let _ = input_event.pointer_set_intercept_hit_test_mode(HitTest::Default);
                    }
                })
                .child(panel)
                .into()])
            .into(),
    );

    let container_node_ref = Rc::new(RefCell::new(None::<ArkUINode>));
    let container_on_change: Rc<dyn Fn(LayoutFrame)> = Rc::new({
        let state = state.clone();
        move |frame| {
            let changed = state.container_offset.set_if_changed(frame);
            apply_floating_position(&state);
            if changed && state.spec.get().open {
                request_runtime_rerender();
            }
        }
    });

    stack_component::<Message, AppTheme>()
        .percent_width(1.0)
        .percent_height(1.0)
        .attr(ArkUINodeAttributeType::Clip, false)
        .hit_test_behavior(
            if matches!(state.spec.get().dismiss_mode, OverlayDismissMode::Backdrop) {
                HitTestBehavior::BlockHierarchy
            } else {
                HitTestBehavior::Transparent
            },
        )
        .attr(
            ArkUINodeAttributeType::Alignment,
            i32::from(Alignment::TopStart),
        )
        .with_patch({
            let node_ref = container_node_ref.clone();
            let on_change = container_on_change.clone();
            move |node| {
                let runtime = node.borrow_mut().clone();
                if let Some(frame) = read_layout_frame(&runtime) {
                    on_change(frame);
                }
                node_ref.replace(Some(runtime));
                Ok(())
            }
        })
        .on_event_no_param(NodeEventType::EventOnAreaChange, {
            let node_ref = container_node_ref;
            let on_change = container_on_change;
            move || {
                if let Some(node) = node_ref.borrow().as_ref() {
                    if let Some(frame) = read_layout_frame(node) {
                        on_change(frame);
                    }
                }
            }
        })
        .children(children)
        .into()
}

pub fn floating_overlay<Message, AppTheme>(
    trigger: Element<Message, AppTheme>,
    panel: Option<Element<Message, AppTheme>>,
    spec: FloatingOverlaySpec,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    Element::new(FloatingOverlayWidget {
        trigger: RefCell::new(Some(trigger)),
        panel: FloatingPanelContent::Static(RefCell::new(panel)),
        spec,
        on_dismiss,
        register_surfaces: Vec::new(),
        dismiss_registry: None,
    })
}

pub fn floating_overlay_with_builder<Message, AppTheme>(
    trigger: Element<Message, AppTheme>,
    spec: FloatingOverlaySpec,
    panel_builder: Rc<dyn Fn(Option<f32>) -> Element<Message, AppTheme>>,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    Element::new(FloatingOverlayWidget {
        trigger: RefCell::new(Some(trigger)),
        panel: FloatingPanelContent::Builder(panel_builder),
        spec,
        on_dismiss,
        register_surfaces: Vec::new(),
        dismiss_registry: None,
    })
}

pub fn floating_overlay_with_surfaces<Message, AppTheme>(
    trigger: Element<Message, AppTheme>,
    panel: Option<Element<Message, AppTheme>>,
    spec: FloatingOverlaySpec,
    on_dismiss: Option<Rc<dyn Fn()>>,
    register_surfaces: Vec<FloatingSurfaceRegistry>,
    dismiss_registry: Option<FloatingSurfaceRegistry>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    Element::new(FloatingOverlayWidget {
        trigger: RefCell::new(Some(trigger)),
        panel: FloatingPanelContent::Static(RefCell::new(panel)),
        spec,
        on_dismiss,
        register_surfaces,
        dismiss_registry,
    })
}

pub fn floating_overlay_with_builder_and_surfaces<Message, AppTheme>(
    trigger: Element<Message, AppTheme>,
    spec: FloatingOverlaySpec,
    panel_builder: Rc<dyn Fn(Option<f32>) -> Element<Message, AppTheme>>,
    on_dismiss: Option<Rc<dyn Fn()>>,
    register_surfaces: Vec<FloatingSurfaceRegistry>,
    dismiss_registry: Option<FloatingSurfaceRegistry>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    Element::new(FloatingOverlayWidget {
        trigger: RefCell::new(Some(trigger)),
        panel: FloatingPanelContent::Builder(panel_builder),
        spec,
        on_dismiss,
        register_surfaces,
        dismiss_registry,
    })
}

fn modal_layer<Message, AppTheme>(
    panel: Element<Message, AppTheme>,
    spec: ModalOverlaySpec,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let mut backdrop = row_component::<Message, AppTheme>()
        .percent_width(1.0)
        .percent_height(1.0)
        .background_color(spec.backdrop_color);

    let backdrop_dismiss = on_dismiss.clone();
    backdrop = backdrop.on_event(NodeEventType::TouchEvent, move |event| {
        let Some(input_event) = event.input_event() else {
            return;
        };
        let _ = input_event.pointer_set_stop_propagation(true);
        if !matches!(
            input_event.action,
            UIInputAction::Up | UIInputAction::Cancel
        ) {
            return;
        }
        if spec.dismiss_on_backdrop {
            if let Some(dismiss) = backdrop_dismiss.as_ref() {
                dismiss();
            }
        }
    });

    let panel = stack_component::<Message, AppTheme>()
        .attr(ArkUINodeAttributeType::Clip, false)
        .on_event(NodeEventType::TouchEvent, move |event| {
            if let Some(input_event) = event.input_event() {
                let _ = input_event.pointer_set_stop_propagation(true);
            }
        })
        .child(panel)
        .into();

    let overlay_panel = match spec.presentation {
        ModalPresentation::CenteredDialog => {
            let outside_dismiss = on_dismiss.clone();
            column_component::<Message, AppTheme>()
                .percent_width(1.0)
                .percent_height(1.0)
                .justify_content_center()
                .align_items_center()
                .on_event(NodeEventType::TouchEvent, move |event| {
                    let Some(input_event) = event.input_event() else {
                        return;
                    };
                    let _ = input_event.pointer_set_stop_propagation(true);
                    if !matches!(
                        input_event.action,
                        UIInputAction::Up | UIInputAction::Cancel
                    ) {
                        return;
                    }
                    if spec.dismiss_on_backdrop {
                        if let Some(dismiss) = outside_dismiss.as_ref() {
                            dismiss();
                        }
                    }
                })
                .attr(
                    ArkUINodeAttributeType::Padding,
                    vec![
                        spec.viewport_inset,
                        spec.viewport_inset,
                        spec.viewport_inset,
                        spec.viewport_inset,
                    ],
                )
                .children(vec![panel])
                .into()
        }
        ModalPresentation::RightSheet => {
            let outside_dismiss = on_dismiss.clone();
            row_component::<Message, AppTheme>()
                .percent_width(1.0)
                .percent_height(1.0)
                .justify_content_end()
                .on_event(NodeEventType::TouchEvent, move |event| {
                    let Some(input_event) = event.input_event() else {
                        return;
                    };
                    let _ = input_event.pointer_set_stop_propagation(true);
                    if !matches!(
                        input_event.action,
                        UIInputAction::Up | UIInputAction::Cancel
                    ) {
                        return;
                    }
                    if spec.dismiss_on_backdrop {
                        if let Some(dismiss) = outside_dismiss.as_ref() {
                            dismiss();
                        }
                    }
                })
                .attr(
                    ArkUINodeAttributeType::Padding,
                    vec![
                        spec.viewport_inset,
                        spec.viewport_inset,
                        spec.viewport_inset,
                        spec.viewport_inset,
                    ],
                )
                .children(vec![column_component::<Message, AppTheme>()
                    .percent_height(1.0)
                    .children(vec![panel])
                    .into()])
                .into()
        }
        ModalPresentation::BottomDrawer => {
            let outside_dismiss = on_dismiss.clone();
            column_component::<Message, AppTheme>()
                .percent_width(1.0)
                .percent_height(1.0)
                .justify_content_end()
                .on_event(NodeEventType::TouchEvent, move |event| {
                    let Some(input_event) = event.input_event() else {
                        return;
                    };
                    let _ = input_event.pointer_set_stop_propagation(true);
                    if !matches!(
                        input_event.action,
                        UIInputAction::Up | UIInputAction::Cancel
                    ) {
                        return;
                    }
                    if spec.dismiss_on_backdrop {
                        if let Some(dismiss) = outside_dismiss.as_ref() {
                            dismiss();
                        }
                    }
                })
                .attr(
                    ArkUINodeAttributeType::Padding,
                    vec![
                        spec.viewport_inset,
                        spec.viewport_inset,
                        spec.viewport_inset,
                        spec.viewport_inset,
                    ],
                )
                .children(vec![panel])
                .into()
        }
    };

    stack_component::<Message, AppTheme>()
        .percent_width(1.0)
        .percent_height(1.0)
        .attr(ArkUINodeAttributeType::Clip, false)
        .attr(
            ArkUINodeAttributeType::Alignment,
            i32::from(Alignment::TopStart),
        )
        .children(vec![backdrop.into(), overlay_panel])
        .into()
}

pub fn modal_overlay<Message, AppTheme>(
    panel: Option<Element<Message, AppTheme>>,
    spec: ModalOverlaySpec,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    Element::new(ModalOverlayWidget {
        panel: RefCell::new(panel),
        spec,
        on_dismiss,
    })
}

pub fn anchored_overlay<Message: 'static, AppTheme: 'static>(
    trigger: Element<Message, AppTheme>,
    panel: Option<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    floating_overlay(
        trigger,
        panel,
        FloatingOverlaySpec {
            open: true,
            ..FloatingOverlaySpec::default()
        },
        None,
    )
}

pub fn native_overlay<Message: 'static, AppTheme: 'static>(
    trigger: Element<Message, AppTheme>,
    panel: Option<Element<Message, AppTheme>>,
    _placement: NativeOverlayPlacement,
) -> Element<Message, AppTheme> {
    anchored_overlay(trigger, panel)
}
