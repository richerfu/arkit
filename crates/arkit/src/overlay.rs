use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUIEvent,
};
use ohos_arkui_binding::types::alignment::Alignment;
use ohos_arkui_binding::types::direction::Direction;
use ohos_arkui_binding::types::overlay::OverlayOptions;

use crate::component::{mount_element, patch_element, MountedElement};
use crate::portal::{current_portal_host, PortalHostHandle};
use crate::queue_after_mount;
use crate::queue_ui_loop;
use crate::view::{Element, ViewNode};
use crate::Signal;

const FLOATING_LAYOUT_EPSILON: f32 = 0.5;

struct RuntimeNode<'a>(&'a mut ArkUINode);

impl ArkUIAttributeBasic for RuntimeNode<'_> {
    fn raw(&self) -> &ArkUINode {
        self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        self.0
    }
}

impl ArkUICommonAttribute for RuntimeNode<'_> {}
impl ArkUIEvent for RuntimeNode<'_> {}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutSize {
    pub width: f32,
    pub height: f32,
}

impl LayoutSize {
    pub fn is_measured(self) -> bool {
        self.width > FLOATING_LAYOUT_EPSILON && self.height > FLOATING_LAYOUT_EPSILON
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutFrame {
    pub fn is_measured(self) -> bool {
        self.width > FLOATING_LAYOUT_EPSILON && self.height > FLOATING_LAYOUT_EPSILON
    }
}

pub fn anchored_overlay(trigger: Element, panel: Option<Element>) -> Element {
    AnchoredOverlayElement { trigger, panel }.into()
}

pub fn native_overlay(
    trigger: Element,
    panel: Option<Element>,
    placement: NativeOverlayPlacement,
) -> Element {
    NativeOverlayElement {
        trigger,
        panel,
        placement,
    }
    .into()
}

pub fn observe_layout_size(element: Element, size: Signal<LayoutSize>) -> Element {
    LayoutObserverElement {
        element,
        observer: LayoutObserver::Size {
            signal: size,
            enabled: true,
        },
    }
    .into()
}

pub fn observe_layout_frame(element: Element, frame: Signal<LayoutFrame>) -> Element {
    observe_layout_frame_enabled(element, frame, true)
}

pub fn observe_layout_frame_enabled(
    element: Element,
    frame: Signal<LayoutFrame>,
    enabled: bool,
) -> Element {
    LayoutObserverElement {
        element,
        observer: LayoutObserver::Frame {
            signal: frame,
            enabled,
        },
    }
    .into()
}

struct AnchoredOverlayState {
    trigger: RefCell<Option<MountedElement>>,
    host: PortalHostHandle,
    entry_id: usize,
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

struct DetachedOverlayPanel {
    node: ArkUINode,
    mounted: MountedElement,
}

impl DetachedOverlayPanel {
    fn cleanup(self) {
        let mut node = self.node;
        let _ = node.dispose();
        self.mounted.cleanup_recursive();
    }
}

struct NativeOverlayState {
    trigger: RefCell<Option<MountedElement>>,
    trigger_node: RefCell<ArkUINode>,
    panel: RefCell<Option<DetachedOverlayPanel>>,
}

impl AnchoredOverlayState {
    fn cleanup(&self) {
        if let Some(trigger) = self.trigger.borrow_mut().take() {
            trigger.cleanup_recursive();
        }

        let _ = self.host.remove(self.entry_id);
    }
}

impl NativeOverlayState {
    fn cleanup(&self) {
        let _ = clear_native_overlay(&self.trigger_node.borrow());

        if let Some(panel) = self.panel.borrow_mut().take() {
            panel.cleanup();
        }

        if let Some(trigger) = self.trigger.borrow_mut().take() {
            trigger.cleanup_recursive();
        }
    }
}

#[derive(Clone)]
enum LayoutObserver {
    Size {
        signal: Signal<LayoutSize>,
        enabled: bool,
    },
    Frame {
        signal: Signal<LayoutFrame>,
        enabled: bool,
    },
}

struct LayoutObserverState {
    child: RefCell<Option<MountedElement>>,
    active: Rc<Cell<bool>>,
    observer: Rc<RefCell<LayoutObserver>>,
}

impl LayoutObserverState {
    fn cleanup(&self) {
        self.active.set(false);
        if let Some(child) = self.child.borrow_mut().take() {
            child.cleanup_recursive();
        }
    }
}

struct AnchoredOverlayElement {
    trigger: Element,
    panel: Option<Element>,
}

struct NativeOverlayElement {
    trigger: Element,
    panel: Option<Element>,
    placement: NativeOverlayPlacement,
}

impl ViewNode for AnchoredOverlayElement {
    fn kind(&self) -> TypeId {
        self.trigger.kind()
    }

    fn key(&self) -> Option<&str> {
        self.trigger.key()
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let Self { trigger, panel } = *self;
        let kind = trigger.kind();
        let key = trigger.key().map(str::to_owned);
        let (trigger_node, trigger_mounted) = mount_element(trigger)?;
        let host = current_portal_host()?;
        let entry_id = host.next_id();
        host.update(entry_id, panel)?;

        let state = Rc::new(AnchoredOverlayState {
            trigger: RefCell::new(Some(trigger_mounted)),
            host,
            entry_id,
        });
        let cleanup_state = state.clone();
        let mut mounted = MountedElement::new(
            kind,
            std::any::type_name::<Self>(),
            key,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        mounted.set_state(Box::new(state));
        Ok((trigger_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        node: &mut ArkUINode,
        mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        let Self { trigger, panel } = *self;
        let kind = trigger.kind();
        let key = trigger.key().map(str::to_owned);
        let state = mounted
            .state_mut::<Rc<AnchoredOverlayState>>()
            .expect("anchored overlay state should exist")
            .clone();

        {
            let mut trigger_meta = state.trigger.borrow_mut();
            let trigger_meta = trigger_meta
                .as_mut()
                .expect("anchored overlay trigger should be mounted");
            patch_element(trigger, node, trigger_meta)?;
        }

        state.host.update(state.entry_id, panel)?;
        mounted.kind = kind;
        mounted.key = key;
        Ok(())
    }
}

impl ViewNode for NativeOverlayElement {
    fn kind(&self) -> TypeId {
        self.trigger.kind()
    }

    fn key(&self) -> Option<&str> {
        self.trigger.key()
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let Self {
            trigger,
            panel,
            placement,
        } = *self;
        let kind = trigger.kind();
        let key = trigger.key().map(str::to_owned);
        let (trigger_node, trigger_mounted) = mount_element(trigger)?;
        let panel_state = mount_native_overlay_panel(&trigger_node, panel, placement)?;

        let state = Rc::new(NativeOverlayState {
            trigger: RefCell::new(Some(trigger_mounted)),
            trigger_node: RefCell::new(trigger_node.clone()),
            panel: RefCell::new(panel_state),
        });
        let cleanup_state = state.clone();
        let mut mounted = MountedElement::new(
            kind,
            std::any::type_name::<Self>(),
            key,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        mounted.set_state(Box::new(state));
        Ok((trigger_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        node: &mut ArkUINode,
        mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        let Self {
            trigger,
            panel,
            placement,
        } = *self;
        let kind = trigger.kind();
        let key = trigger.key().map(str::to_owned);
        let state = mounted
            .state_mut::<Rc<NativeOverlayState>>()
            .expect("native overlay state should exist")
            .clone();

        {
            let mut trigger_meta = state.trigger.borrow_mut();
            let trigger_meta = trigger_meta
                .as_mut()
                .expect("native overlay trigger should be mounted");
            patch_element(trigger, node, trigger_meta)?;
        }

        state.trigger_node.replace(node.clone());
        update_native_overlay_panel(
            &state.trigger_node.borrow(),
            &mut state.panel.borrow_mut(),
            panel,
            placement,
        )?;

        mounted.kind = kind;
        mounted.key = key;
        Ok(())
    }
}

struct LayoutObserverElement {
    element: Element,
    observer: LayoutObserver,
}

impl ViewNode for LayoutObserverElement {
    fn kind(&self) -> TypeId {
        self.element.kind()
    }

    fn key(&self) -> Option<&str> {
        self.element.key()
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let Self { element, observer } = *self;
        let kind = element.kind();
        let key = element.key().map(str::to_owned);
        let (mut node, child_mounted) = mount_element(element)?;
        let active = Rc::new(Cell::new(true));
        let observer = Rc::new(RefCell::new(observer));
        attach_layout_observer(&mut node, observer.clone(), active.clone());
        let state = Rc::new(LayoutObserverState {
            child: RefCell::new(Some(child_mounted)),
            active,
            observer,
        });
        let cleanup_state = state.clone();
        let mut mounted = MountedElement::new(
            kind,
            std::any::type_name::<Self>(),
            key,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        mounted.set_state(Box::new(state));
        Ok((node, mounted))
    }

    fn patch(
        self: Box<Self>,
        node: &mut ArkUINode,
        mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        let Self { element, observer } = *self;
        let kind = element.kind();
        let key = element.key().map(str::to_owned);
        let state = mounted
            .state_mut::<Rc<LayoutObserverState>>()
            .expect("layout observer state should exist")
            .clone();
        {
            let mut child_meta = state.child.borrow_mut();
            let child_meta = child_meta
                .as_mut()
                .expect("layout observer child should be mounted");
            patch_element(element, node, child_meta)?;
        }

        state.observer.replace(observer);
        update_layout_observer(node, &state.observer.borrow());

        mounted.kind = kind;
        mounted.key = key;
        Ok(())
    }
}

fn attach_layout_observer(
    node: &mut ArkUINode,
    observer: Rc<RefCell<LayoutObserver>>,
    active: Rc<Cell<bool>>,
) {
    let measured_node = Rc::new(RefCell::new(node.clone()));
    let update_measurement: Rc<dyn Fn()> = Rc::new({
        let measured_node = measured_node.clone();
        let observer = observer.clone();
        move || {
            if !active.get() {
                return;
            }

            let measured_node = measured_node.borrow();
            let observer = observer.borrow();
            update_layout_observer(&measured_node, &observer);
        }
    });

    let mut runtime_node = RuntimeNode(node);
    runtime_node.on_area_change({
        let update_measurement = update_measurement.clone();
        move |_| update_measurement()
    });
    runtime_node.on_size_change({
        let update_measurement = update_measurement.clone();
        move |_| update_measurement()
    });

    queue_after_mount(move || {
        queue_ui_loop(move || {
            update_measurement();
        });
    });
}

fn update_layout_observer(node: &ArkUINode, observer: &LayoutObserver) {
    match observer {
        LayoutObserver::Size { signal, enabled } => {
            if !enabled {
                return;
            }
            let next = layout_size(node);
            if !size_approx_equal(signal.get(), next) {
                signal.set(next);
            }
        }
        LayoutObserver::Frame { signal, enabled } => {
            if !enabled {
                return;
            }
            let next = layout_frame(node);
            if !frame_approx_equal(signal.get(), next) {
                signal.set(next);
            }
        }
    }
}

fn frame_approx_equal(a: LayoutFrame, b: LayoutFrame) -> bool {
    (a.x - b.x).abs() < FLOATING_LAYOUT_EPSILON
        && (a.y - b.y).abs() < FLOATING_LAYOUT_EPSILON
        && (a.width - b.width).abs() < FLOATING_LAYOUT_EPSILON
        && (a.height - b.height).abs() < FLOATING_LAYOUT_EPSILON
}

fn size_approx_equal(a: LayoutSize, b: LayoutSize) -> bool {
    (a.width - b.width).abs() < FLOATING_LAYOUT_EPSILON
        && (a.height - b.height).abs() < FLOATING_LAYOUT_EPSILON
}

fn layout_size(node: &ArkUINode) -> LayoutSize {
    node.layout_size()
        .map(|size| LayoutSize {
            width: size.width as f32,
            height: size.height as f32,
        })
        .unwrap_or_default()
}

fn layout_frame(node: &ArkUINode) -> LayoutFrame {
    let size = node.layout_size();
    let position = node.position_with_translate_in_window();
    match (position, size) {
        (Ok(position), Ok(size)) => LayoutFrame {
            x: position.x as f32,
            y: position.y as f32,
            width: size.width as f32,
            height: size.height as f32,
        },
        _ => LayoutFrame::default(),
    }
}

fn mount_native_overlay_panel(
    trigger_node: &ArkUINode,
    panel: Option<Element>,
    placement: NativeOverlayPlacement,
) -> ArkUIResult<Option<DetachedOverlayPanel>> {
    let Some(panel) = panel else {
        clear_native_overlay(trigger_node)?;
        return Ok(None);
    };

    let (panel_node, panel_mounted) = mount_element(panel)?;
    if let Err(error) = apply_native_overlay(trigger_node, &panel_node, placement) {
        let mut cleanup_node = panel_node.clone();
        let _ = cleanup_node.dispose();
        panel_mounted.cleanup_recursive();
        return Err(error);
    }

    Ok(Some(DetachedOverlayPanel {
        node: panel_node,
        mounted: panel_mounted,
    }))
}

fn update_native_overlay_panel(
    trigger_node: &ArkUINode,
    current_panel: &mut Option<DetachedOverlayPanel>,
    next_panel: Option<Element>,
    placement: NativeOverlayPlacement,
) -> ArkUIResult<()> {
    match next_panel {
        Some(next_panel) => {
            if let Some(current_panel) = current_panel.as_mut() {
                let patchable = current_panel.mounted.kind == next_panel.kind()
                    && current_panel.mounted.key.as_deref() == next_panel.key();
                if patchable {
                    patch_element(
                        next_panel,
                        &mut current_panel.node,
                        &mut current_panel.mounted,
                    )?;
                    return apply_native_overlay(trigger_node, &current_panel.node, placement);
                }
            }

            if let Some(current_panel) = current_panel.take() {
                let _ = clear_native_overlay(trigger_node);
                current_panel.cleanup();
            }

            *current_panel = mount_native_overlay_panel(trigger_node, Some(next_panel), placement)?;
            Ok(())
        }
        None => {
            clear_native_overlay(trigger_node)?;
            if let Some(current_panel) = current_panel.take() {
                current_panel.cleanup();
            }
            Ok(())
        }
    }
}

fn apply_native_overlay(
    trigger_node: &ArkUINode,
    panel_node: &ArkUINode,
    placement: NativeOverlayPlacement,
) -> ArkUIResult<()> {
    let mut options = OverlayOptions::new();
    options
        .node(panel_node)
        .alignment(placement.alignment)
        .offset(placement.offset_x, placement.offset_y)
        .direction(placement.direction);
    trigger_node.set_overlay(options)
}

fn clear_native_overlay(trigger_node: &ArkUINode) -> ArkUIResult<()> {
    trigger_node.reset_overlay()
}
