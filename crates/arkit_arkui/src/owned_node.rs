//! Explicit ownership for imperatively-created ArkUI nodes.

use ohos_arkui_binding::common::node::ArkUINode;

/// Unique owner of an unattached native node.
///
/// Dropping the value disposes the native subtree. Ownership is transferred
/// only at renderer/adapter attachment boundaries.
pub struct OwnedNativeNode {
    node: Option<ArkUINode>,
}

impl OwnedNativeNode {
    pub(crate) fn from_raw(node: ArkUINode) -> Self {
        Self { node: Some(node) }
    }

    pub(crate) fn as_raw(&self) -> &ArkUINode {
        self.node
            .as_ref()
            .expect("owned native node was already transferred")
    }

    pub(crate) fn as_raw_mut(&mut self) -> &mut ArkUINode {
        self.node
            .as_mut()
            .expect("owned native node was already transferred")
    }

    pub(crate) fn into_raw(mut self) -> ArkUINode {
        self.node
            .take()
            .expect("owned native node was already transferred")
    }
}

impl Drop for OwnedNativeNode {
    fn drop(&mut self) {
        if let Some(mut node) = self.node.take() {
            let _ = node.dispose();
        }
    }
}
