//! A small ergonomic builder for imperatively constructing owned ArkUI nodes.
//!
//! Used by virtual List/Grid/WaterFlow `render_item` callbacks, which run
//! outside the dioxus render cycle and return an [`OwnedNativeNode`].

use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUIEvent,
};
pub use ohos_arkui_binding::event::inner_event::Event as NativeNodeEvent;
pub use ohos_arkui_binding::r#type::drag::PreDragStatus;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
pub use ohos_arkui_binding::types::event::NodeEventType;

use crate::OwnedNativeNode;

struct EventNode<'a>(&'a mut ArkUINode);

impl ArkUIAttributeBasic for EventNode<'_> {
    fn raw(&self) -> &ArkUINode {
        self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        self.0
    }
}

impl ArkUIEvent for EventNode<'_> {}

/// A chainable builder over an owned [`ArkUINode`]. Consumes itself to produce
/// an [`OwnedNativeNode`] via [`build`](Self::build).
pub struct NodeBuilder {
    node: Option<OwnedNativeNode>,
}

impl NodeBuilder {
    /// Create a builder for a node of the given tag (e.g. `"row"`, `"text"`,
    /// `"column"`).
    pub fn new(tag: &str) -> ArkUIResult<Self> {
        Ok(Self {
            node: Some(OwnedNativeNode::from_raw(
                crate::native::create_node_by_tag(tag)?,
            )),
        })
    }

    pub(crate) fn from_raw(node: ArkUINode) -> Self {
        Self {
            node: Some(OwnedNativeNode::from_raw(node)),
        }
    }

    /// Set an attribute by its canonical [`ArkUINodeAttributeType`] with any
    /// value convertible to [`ArkUINodeAttributeItem`].
    pub fn attr(
        self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> ArkUIResult<Self> {
        self.node
            .as_ref()
            .expect("NodeBuilder owns a node until build")
            .as_raw()
            .set_attribute(attr, value.into())?;
        Ok(self)
    }

    /// Append a child node.
    pub fn child(mut self, child: OwnedNativeNode) -> ArkUIResult<Self> {
        let child = child.into_shared();
        if let Err(error) = self
            .node
            .as_mut()
            .expect("NodeBuilder owns a node until build")
            .as_raw_mut()
            .add_child(child.clone())
        {
            let _ = child.borrow_mut().dispose();
            return Err(error);
        }
        Ok(self)
    }

    /// Convenience: percent width (0.0–1.0).
    pub fn percent_width(self, v: f32) -> ArkUIResult<Self> {
        self.attr(ArkUINodeAttributeType::WidthPercent, v)
    }

    /// Convenience: percent height (0.0–1.0).
    pub fn percent_height(self, v: f32) -> ArkUIResult<Self> {
        self.attr(ArkUINodeAttributeType::HeightPercent, v)
    }

    /// Convenience: fixed height (vp).
    pub fn height(self, v: f32) -> ArkUIResult<Self> {
        self.attr(ArkUINodeAttributeType::Height, v)
    }

    /// Convenience: fixed width (vp).
    pub fn width(self, v: f32) -> ArkUIResult<Self> {
        self.attr(ArkUINodeAttributeType::Width, v)
    }

    /// Convenience: background color (hex string or ARGB int).
    pub fn background_color(self, color: impl Into<String>) -> ArkUIResult<Self> {
        let c = color.into();
        let argb = crate::parse_color(&c).map_err(|_| {
            ohos_arkui_binding::common::error::ArkUIError::new(
                ohos_arkui_binding::arkui_input_binding::ArkUIErrorCode::ParamInvalid,
                format!("invalid color: {c}"),
            )
        })?;
        self.attr(ArkUINodeAttributeType::BackgroundColor, argb)
    }

    /// Convenience: font size (vp).
    pub fn font_size(self, v: f32) -> ArkUIResult<Self> {
        self.attr(ArkUINodeAttributeType::FontSize, v)
    }

    /// Convenience: font color (hex string or ARGB int).
    pub fn font_color(self, color: impl Into<String>) -> ArkUIResult<Self> {
        let c = color.into();
        let argb = crate::parse_color(&c).map_err(|_| {
            ohos_arkui_binding::common::error::ArkUIError::new(
                ohos_arkui_binding::arkui_input_binding::ArkUIErrorCode::ParamInvalid,
                format!("invalid color: {c}"),
            )
        })?;
        self.attr(ArkUINodeAttributeType::FontColor, argb)
    }

    /// Convenience: text content.
    pub fn text_content(self, text: impl Into<String>) -> ArkUIResult<Self> {
        self.attr(ArkUINodeAttributeType::TextContent, text.into())
    }

    /// Convenience: padding [top, right, bottom, left].
    pub fn padding(self, v: [f32; 4]) -> ArkUIResult<Self> {
        self.attr(
            ArkUINodeAttributeType::Padding,
            vec![v[0], v[1], v[2], v[3]],
        )
    }

    /// Convenience: margin [top, right, bottom, left].
    pub fn margin(self, v: [f32; 4]) -> ArkUIResult<Self> {
        self.attr(ArkUINodeAttributeType::Margin, vec![v[0], v[1], v[2], v[3]])
    }

    /// Register any ArkUI node event on an imperatively-built node.
    ///
    /// This is the escape hatch for component-specific events. Prefer the
    /// typed convenience methods below for common input and lifecycle events.
    /// The borrowed [`NativeNodeEvent`] is valid only for the duration of the
    /// callback and must not be retained.
    pub fn on_event(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn(&NativeNodeEvent) + 'static,
    ) -> ArkUIResult<Self> {
        let node = self
            .node
            .as_mut()
            .expect("NodeBuilder owns a node until build")
            .as_raw_mut();
        EventNode(node).on_event(event_type, callback);
        Ok(self)
    }

    /// Register a payload-free ArkUI node event.
    pub fn on_event_no_param(
        self,
        event_type: NodeEventType,
        callback: impl Fn() + 'static,
    ) -> ArkUIResult<Self> {
        self.on_event(event_type, move |_| callback())
    }

    /// Register a click callback.
    ///
    /// Arkit targets API 20 and deliberately uses the API 18+
    /// `OnClickEvent`, whose payload is an `ArkUI_UIInputEvent`. Registering
    /// the legacy and modern variants together causes duplicate delivery on
    /// some runtimes and unreliable delivery inside `NodeAdapter` items.
    pub fn on_click(self, callback: impl Fn() + 'static) -> ArkUIResult<Self> {
        self.on_event_no_param(NodeEventType::OnClickEvent, callback)
    }

    /// Register raw touch input (down, move, up, and cancel).
    pub fn on_touch(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::TouchEvent, callback)
    }

    /// Intercept raw touch input. Return `true` to consume it.
    pub fn on_touch_intercept(
        self,
        callback: impl Fn(&NativeNodeEvent) -> bool + 'static,
    ) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnTouchIntercept, move |event| {
            let _ = event.set_return_bool(callback(event));
        })
    }

    /// Register hover enter/leave.
    ///
    /// The component-event variant is intentional. The API 17 input-event
    /// variant is not a touch event, while the current upstream Rust binding
    /// unconditionally parses every input action as `UI_TOUCH_EVENT_ACTION`.
    /// That turns a valid hover into a panic before `is_hovered` can be read.
    pub fn on_hover(self, callback: impl Fn(bool) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnHover, move |event| {
            callback(event.i32_value(0).unwrap_or_default() != 0);
        })
    }

    /// Register continuous pointer/stylus hover movement.
    pub fn on_hover_move(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnHoverMove, callback)
    }

    /// Register mouse button or movement input.
    pub fn on_mouse(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnMouse, callback)
    }

    /// Register native focus acquisition.
    pub fn on_focus(self, callback: impl Fn() + 'static) -> ArkUIResult<Self> {
        self.on_event_no_param(NodeEventType::OnFocus, callback)
    }

    /// Register native focus loss.
    pub fn on_blur(self, callback: impl Fn() + 'static) -> ArkUIResult<Self> {
        self.on_event_no_param(NodeEventType::OnBlur, callback)
    }

    /// Register when the node becomes mounted and visible.
    pub fn on_appear(self, callback: impl Fn() + 'static) -> ArkUIResult<Self> {
        self.on_event_no_param(NodeEventType::EventOnAppear, callback)
    }

    /// Register when the node becomes unmounted or hidden.
    pub fn on_disappear(self, callback: impl Fn() + 'static) -> ArkUIResult<Self> {
        self.on_event_no_param(NodeEventType::EventOnDisappear, callback)
    }

    /// Register when the node is attached to the native tree.
    pub fn on_attach(self, callback: impl Fn() + 'static) -> ArkUIResult<Self> {
        self.on_event_no_param(NodeEventType::EventOnAttach, callback)
    }

    /// Register when the node is detached from the native tree.
    pub fn on_detach(self, callback: impl Fn() + 'static) -> ArkUIResult<Self> {
        self.on_event_no_param(NodeEventType::EventOnDetach, callback)
    }

    /// Register element area changes.
    pub fn on_area_change(
        self,
        callback: impl Fn(&NativeNodeEvent) + 'static,
    ) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::EventOnAreaChange, callback)
    }

    /// Register exact visible-area threshold changes.
    pub fn on_visible_area_change(
        self,
        callback: impl Fn(bool, f32) + 'static,
    ) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::EventOnVisibleAreaChange, move |event| {
            callback(
                event.i32_value(0).unwrap_or_default() != 0,
                event.f32_value(1).unwrap_or_default(),
            );
        })
    }

    /// Register throttled visible-area changes.
    ///
    /// Configure `VisibleAreaApproximateChangeRatio` on the node before using
    /// this callback; ArkUI otherwise has no threshold or interval to observe.
    pub fn on_visible_area_approximate_change(
        self,
        callback: impl Fn(bool, f32) + 'static,
    ) -> ArkUIResult<Self> {
        self.on_event(
            NodeEventType::VisibleAreaApproximateChangeEvent,
            move |event| {
                callback(
                    event.i32_value(0).unwrap_or_default() != 0,
                    event.f32_value(1).unwrap_or_default(),
                );
            },
        )
    }

    /// Register an accessibility action.
    pub fn on_accessibility_action(self, callback: impl Fn(u32) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnAccessibilityActions, move |event| {
            callback(event.u32_value(0).unwrap_or_default());
        })
    }

    /// Register the pre-drag state transition.
    pub fn on_pre_drag(
        self,
        callback: impl Fn(Option<PreDragStatus>) + 'static,
    ) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnPreDrag, move |event| {
            callback(event.pre_drag_status());
        })
    }

    /// Register drag start.
    pub fn on_drag_start(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnDragStart, callback)
    }

    /// Register drag enter.
    pub fn on_drag_enter(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnDragEnter, callback)
    }

    /// Register drag movement.
    pub fn on_drag_move(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnDragMove, callback)
    }

    /// Register drag leave.
    pub fn on_drag_leave(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnDragLeave, callback)
    }

    /// Register drop.
    pub fn on_drop(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnDrop, callback)
    }

    /// Register drag end.
    pub fn on_drag_end(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnDragEnd, callback)
    }

    /// Register a focused node's key event.
    pub fn on_key(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnKeyEvent, callback)
    }

    /// Register a key event before IME handling. Return `true` to consume it.
    pub fn on_key_pre_ime(
        self,
        callback: impl Fn(&NativeNodeEvent) -> bool + 'static,
    ) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnKeyPreIme, move |event| {
            let _ = event.set_return_bool(callback(event));
        })
    }

    /// Intercept key dispatch to child nodes. Return `true` to consume it.
    pub fn on_dispatch_key(
        self,
        callback: impl Fn(&NativeNodeEvent) -> bool + 'static,
    ) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::DispatchKeyEvent, move |event| {
            let _ = event.set_return_bool(callback(event));
        })
    }

    /// Register joystick/focus-axis input.
    pub fn on_focus_axis(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnFocusAxis, callback)
    }

    /// Register mouse-wheel, touchpad, or other axis input.
    pub fn on_axis(self, callback: impl Fn(&NativeNodeEvent) + 'static) -> ArkUIResult<Self> {
        self.on_event(NodeEventType::OnAxis, callback)
    }

    /// Build the node.
    pub fn build(mut self) -> OwnedNativeNode {
        self.node
            .take()
            .expect("NodeBuilder ownership was already transferred")
    }
}
