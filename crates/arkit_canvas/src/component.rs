use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;

use arkit_hooks::{use_mounted_node, use_native_element_ref};
use arkit_prelude::*;
use dioxus_core::use_drop;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUIEvent};
use ohos_arkui_binding::types::advanced::NodeDirtyFlag;
use ohos_drawing_binding::{BlendMode, Canvas as NativeCanvas, ClipOperation, Rect as NativeRect};

use crate::context::CanvasSurface;
use crate::{CanvasImage, CanvasRenderingContext2D, CanvasRenderingContext2DSettings};

type CanvasDrawCallback = dyn for<'frame> Fn(&mut CanvasRenderingContext2D<'frame>);

/// A stable drawing callback accepted by [`Canvas`].
#[derive(Clone)]
pub struct CanvasRenderer(Rc<CanvasDrawCallback>);

impl CanvasRenderer {
    pub fn new(
        callback: impl for<'frame> Fn(&mut CanvasRenderingContext2D<'frame>) + 'static,
    ) -> Self {
        Self(Rc::new(callback))
    }

    fn draw(&self, context: &mut CanvasRenderingContext2D<'_>) {
        (self.0)(context);
    }
}

impl fmt::Debug for CanvasRenderer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CanvasRenderer(..)")
    }
}

impl PartialEq for CanvasRenderer {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

type CanvasInvalidator = Rc<dyn Fn()>;
type CanvasSizeReader = Rc<dyn Fn() -> [f32; 2]>;
type CanvasSnapshotReader = Rc<dyn Fn() -> Option<CanvasImage>>;

struct CanvasControllerBinding {
    id: u64,
    invalidate: CanvasInvalidator,
    size: CanvasSizeReader,
    snapshot: CanvasSnapshotReader,
}

#[derive(Default)]
struct CanvasControllerState {
    next_binding: u64,
    binding: Option<CanvasControllerBinding>,
    pending_redraw: bool,
}

/// Imperative handle for redraw requests and mounted logical size queries.
#[derive(Clone, Default)]
pub struct CanvasController {
    inner: Rc<RefCell<CanvasControllerState>>,
}

impl CanvasController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_redraw(&self) {
        let invalidate = self
            .inner
            .borrow()
            .binding
            .as_ref()
            .map(|binding| binding.invalidate.clone());
        if let Some(invalidate) = invalidate {
            invalidate();
        } else {
            self.inner.borrow_mut().pending_redraw = true;
        }
    }

    pub fn get_size(&self) -> Option<[f32; 2]> {
        let reader = self
            .inner
            .borrow()
            .binding
            .as_ref()
            .map(|binding| binding.size.clone())?;
        Some(reader())
    }

    pub fn get_width(&self) -> Option<f32> {
        self.get_size().map(|size| size[0])
    }

    pub fn get_height(&self) -> Option<f32> {
        self.get_size().map(|size| size[1])
    }

    /// Copy the mounted backing store into an immutable image source.
    ///
    /// The returned image can be passed to another context's `draw_image*`
    /// methods or used to create a pattern. Before the first draw, after
    /// unmount, or while the surface is already mutably borrowed, this returns
    /// `None` instead of panicking.
    pub fn snapshot(&self) -> Option<CanvasImage> {
        let reader = self
            .inner
            .borrow()
            .binding
            .as_ref()
            .map(|binding| binding.snapshot.clone())?;
        reader()
    }

    pub fn is_mounted(&self) -> bool {
        self.inner.borrow().binding.is_some()
    }

    fn bind(
        &self,
        invalidate: CanvasInvalidator,
        size: CanvasSizeReader,
        snapshot: CanvasSnapshotReader,
    ) -> u64 {
        let (id, pending) = {
            let mut state = self.inner.borrow_mut();
            state.next_binding = state
                .next_binding
                .checked_add(1)
                .expect("arkit_canvas: controller binding id space exhausted");
            let id = state.next_binding;
            state.binding = Some(CanvasControllerBinding {
                id,
                invalidate: invalidate.clone(),
                size,
                snapshot,
            });
            (id, std::mem::take(&mut state.pending_redraw))
        };
        if pending {
            invalidate();
        }
        id
    }

    fn unbind(&self, id: u64) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|binding| binding.id == id)
        {
            state.binding = None;
        }
    }
}

impl fmt::Debug for CanvasController {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanvasController")
            .field("mounted", &self.is_mounted())
            .finish_non_exhaustive()
    }
}

impl PartialEq for CanvasController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

/// Layout and drawing properties for [`Canvas`].
#[derive(Clone, Props, PartialEq)]
pub struct CanvasProps {
    /// Synchronous Canvas 2D renderer called from ArkUI's custom draw frame.
    pub draw: CanvasRenderer,
    /// CSS width (`"100%"`, `"320"`). Defaults to `"100%"`.
    #[props(default = "100%".to_string())]
    pub width: String,
    /// CSS height (`"300"`, `"100%"`). Defaults to `"300"` when unset.
    #[props(default)]
    pub height: Option<String>,
    /// Clear the backing bitmap before invoking `draw`. The clear value is
    /// transparent black for alpha contexts and opaque black otherwise.
    #[props(default = false)]
    pub clear_before_draw: bool,
    /// Canvas 2D context creation attributes. Unsupported native backing
    /// formats are reflected as their actual fallback by
    /// `get_context_attributes()`.
    #[props(default)]
    pub settings: CanvasRenderingContext2DSettings,
    /// Optional imperative redraw handle.
    #[props(default)]
    pub controller: Option<CanvasController>,
}

struct CustomEventNode<'a>(&'a mut ArkUINode);

impl ArkUIAttributeBasic for CustomEventNode<'_> {
    fn raw(&self) -> &ArkUINode {
        self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        self.0
    }
}

impl ArkUIEvent for CustomEventNode<'_> {}

/// Native Canvas 2D component backed by an ArkUI custom-draw node.
#[component]
pub fn Canvas(props: CanvasProps) -> Element {
    let node_ref = use_native_element_ref();
    let renderer = use_hook(|| Rc::new(RefCell::new(props.draw.clone())));
    renderer.replace(props.draw.clone());
    let clear_before_draw = use_hook(|| Rc::new(Cell::new(props.clear_before_draw)));
    clear_before_draw.set(props.clear_before_draw);
    let settings = use_hook(|| Rc::new(Cell::new(props.settings)));
    settings.set(props.settings);
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<u64>)));
    let surface = use_hook(|| Rc::new(RefCell::new(None::<CanvasSurface>)));

    let controller_binding = use_hook(|| Rc::new(RefCell::new(None::<(CanvasController, u64)>)));
    let controller_changed = {
        let binding = controller_binding.borrow();
        match (binding.as_ref(), props.controller.as_ref()) {
            (Some((current, _)), Some(next)) => current != next,
            (None, None) => false,
            _ => true,
        }
    };
    if controller_changed {
        if let Some((controller, binding)) = controller_binding.borrow_mut().take() {
            controller.unbind(binding);
        }
        if let Some(controller) = props.controller.clone() {
            let invalidate_node = node_ref.clone();
            let size_node = node_ref.clone();
            let snapshot_surface = surface.clone();
            let binding = controller.bind(
                Rc::new(move || {
                    if let Some(node) = invalidate_node.current() {
                        // SAFETY: dirty marking is a renderer-compatible
                        // operation and the native borrow does not escape.
                        let _ = unsafe {
                            node.with_native(|node| node.mark_dirty(NodeDirtyFlag::NeedRender))
                        };
                    }
                }),
                Rc::new(move || {
                    let ratio = CanvasSurface::display_pixel_ratio();
                    size_node
                        .current()
                        .and_then(|node| {
                            // SAFETY: layout is read synchronously from the
                            // generation-checked node and is not retained.
                            unsafe { node.with_native(|node| node.layout_size().ok()) }.flatten()
                        })
                        .map_or([0.0, 0.0], |size| {
                            [size.width as f32 / ratio, size.height as f32 / ratio]
                        })
                }),
                Rc::new(move || {
                    snapshot_surface
                        .try_borrow()
                        .ok()
                        .and_then(|surface| surface.as_ref().and_then(CanvasSurface::snapshot))
                }),
            );
            controller_binding
                .borrow_mut()
                .replace((controller, binding));
        }
    }
    let drop_binding = controller_binding.clone();
    use_drop(move || {
        if let Some((controller, binding)) = drop_binding.borrow_mut().take() {
            controller.unbind(binding);
        }
    });

    let effect_renderer = renderer.clone();
    let effect_clear = clear_before_draw.clone();
    let effect_settings = settings.clone();
    let effect_registered = registered_node.clone();
    let effect_surface = surface.clone();
    use_mounted_node(node_ref.clone(), move |node| {
        let Some(node) = node else {
            effect_registered.set(None);
            effect_surface.borrow_mut().take();
            return;
        };
        let native_key = node.epoch();
        if effect_registered.get() != Some(native_key) {
            let renderer = effect_renderer.clone();
            let clear_before_draw = effect_clear.clone();
            let settings = effect_settings.clone();
            let surface = effect_surface.clone();
            // SAFETY: custom-draw is independent of the renderer's normal
            // node-event route. The callback is owned by the mounted custom
            // node and does not retain this native borrow.
            let _ = unsafe {
                node.with_native_mut(|node| {
                    CustomEventNode(node).on_custom_draw(move |event| {
                        let Some(draw_context) = event.draw_context_in_draw() else {
                            return;
                        };
                        let Some(raw_canvas) = draw_context.canvas() else {
                            return;
                        };
                        // SAFETY: ArkUI owns the canvas for exactly this
                        // synchronous callback. No wrapper escapes.
                        let native = NativeCanvas::from_raw_borrowed(raw_canvas.cast());
                        let size = draw_context.size();
                        let ratio = CanvasSurface::display_pixel_ratio();
                        let width = size.width as f32 / ratio;
                        let height = size.height as f32 / ratio;
                        let pixel_width = size.width.max(1) as u32;
                        let pixel_height = size.height.max(1) as u32;
                        let context_settings = settings.get();
                        let mut surface = surface.borrow_mut();
                        if surface.as_ref().is_none_or(|surface| {
                            !surface.matches(pixel_width, pixel_height, ratio, context_settings)
                        }) {
                            surface.replace(CanvasSurface::new(
                                width,
                                height,
                                pixel_width,
                                pixel_height,
                                ratio,
                                context_settings,
                            ));
                        }
                        let surface = surface
                            .as_mut()
                            .expect("arkit_canvas: surface initialized above");
                        if clear_before_draw.get() {
                            surface.clear_pixels();
                        }
                        {
                            let mut context = surface.context();
                            renderer.borrow().draw(&mut context);
                        }
                        // Clear through the node clip so a shared parent render
                        // target cannot erase sibling content.
                        native.save();
                        let bounds = NativeRect::new(
                            0.0,
                            0.0,
                            size.width.max(1) as f32,
                            size.height.max(1) as f32,
                        );
                        native.clip_rect(&bounds, ClipOperation::Intersect, false);
                        let _ = native.draw_color(0x0000_0000, BlendMode::Clear);
                        surface.draw_to(&native);
                        native.restore();
                    });
                })
            };
            effect_registered.set(Some(native_key));
        }
        // SAFETY: dirty marking neither changes ownership nor event routing.
        let _ = unsafe { node.with_native(|node| node.mark_dirty(NodeDirtyFlag::NeedRender)) };
    });

    let height = props.height.clone().unwrap_or_else(|| "300".into());
    rsx! {
        custom {
            native_ref: node_ref,
            width: props.width.clone(),
            height: height,
            hit_test_behavior: "default",
        }
    }
}
