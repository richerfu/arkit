use std::sync::mpsc::Sender;

use arkit_arkui::MountedNodeLease;
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
        node: &MountedNodeLease,
        sender: Sender<CameraXComponentEvent>,
    ) -> CameraResult<Self> {
        // SAFETY: context lookup is synchronous inside the generation-checked
        // borrow. The returned XComponent is retained by the registration,
        // whose owner is tied to this lease's native teardown.
        let raw = unsafe {
            node.with_native(|node| {
                OH_NativeXComponent_GetNativeXComponent(node.raw_handle().cast())
            })
        }
        .ok_or_else(|| {
            crate::CameraError::invalid_state(
                "SurfaceRegistration::attach",
                "XComponent is no longer mounted",
            )
        })?;
        let component = NativeXComponent::new(XComponentRaw(raw));
        let attachment = CameraXComponentAttachment::attach(component, sender)
            .map_err(crate::CameraError::from)?;
        Ok(Self {
            _attachment: attachment,
        })
    }
}
