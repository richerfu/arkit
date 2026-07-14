//! A small ergonomic builder for imperatively constructing [`ArkUINode`]s.
//!
//! Used by virtual-list `render_item` callbacks (which run outside the dioxus
//! render cycle and must return a raw `ArkUINode`), so demos don't touch the
//! binding crate directly.

use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;

/// A chainable builder over an [`ArkUINode`]. Consumes itself to produce the
/// node via [`build`](Self::build).
pub struct NodeBuilder {
    // The binding requires explicit native disposal and `ArkUINode` has no
    // `Drop` implementation. Keeping the in-progress node optional lets the
    // builder clean up every early-error path while transferring ownership
    // exactly once from `build`.
    node: Option<ArkUINode>,
}

impl NodeBuilder {
    /// Create a builder for a node of the given tag (e.g. `"row"`, `"text"`,
    /// `"column"`).
    pub fn new(tag: &str) -> ArkUIResult<Self> {
        Ok(Self {
            node: Some(crate::create_node_by_tag(tag)?),
        })
    }

    /// Wrap an existing node.
    pub fn from_node(node: ArkUINode) -> Self {
        Self { node: Some(node) }
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
            .set_attribute(attr, value.into())?;
        Ok(self)
    }

    /// Append a child node.
    pub fn child(mut self, child: ArkUINode) -> ArkUIResult<Self> {
        self.node
            .as_mut()
            .expect("NodeBuilder owns a node until build")
            .add_child(child)?;
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

    /// Build the node.
    pub fn build(mut self) -> ArkUINode {
        self.node
            .take()
            .expect("NodeBuilder ownership was already transferred")
    }
}

impl Drop for NodeBuilder {
    fn drop(&mut self) {
        if let Some(mut node) = self.node.take() {
            let _ = node.dispose();
        }
    }
}
