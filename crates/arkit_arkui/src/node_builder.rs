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
    /// This is the escape hatch for component-specific events. The borrowed
    /// [`NativeNodeEvent`] is valid only for the duration of the callback and
    /// must not be retained.
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

    /// Build the node.
    pub fn build(mut self) -> OwnedNativeNode {
        self.node
            .take()
            .expect("NodeBuilder ownership was already transferred")
    }
}
