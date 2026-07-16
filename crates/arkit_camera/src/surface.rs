use std::sync::mpsc::Sender;

use ohos_arkui_binding::common::node::ArkUINode;
use ohos_camera_binding::{CameraXComponentAttachment, CameraXComponentEvent};
use ohos_xcomponent_binding::{NativeXComponent, XComponentRaw};
use ohos_xcomponent_sys::OH_NativeXComponent_GetNativeXComponent;

use crate::CameraResult;

/// arkit-side owner for the binding crate's optional XComponent adapter.
pub(crate) struct SurfaceRegistration {
    _attachment: CameraXComponentAttachment,
}

impl SurfaceRegistration {
    pub(crate) fn attach(
        node: &ArkUINode,
        sender: Sender<CameraXComponentEvent>,
    ) -> CameraResult<Self> {
        // SAFETY: the renderer owns a mounted ArkUI XComponent node for the
        // lifetime of this registration.
        let raw = unsafe { OH_NativeXComponent_GetNativeXComponent(node.raw_handle().cast()) };
        let component = NativeXComponent::new(XComponentRaw(raw));
        let attachment = CameraXComponentAttachment::attach(component, sender)
            .map_err(crate::CameraError::from)?;
        Ok(Self {
            _attachment: attachment,
        })
    }
}
