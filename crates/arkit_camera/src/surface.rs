use std::sync::mpsc::Sender;

use arkit_arkui::MountedNodeLease;
use ohos_camera_binding::{CameraXComponentAttachment, CameraXComponentEvent};

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
        // SAFETY: component lookup is synchronous inside the
        // generation-checked borrow. The returned wrapper is retained by the
        // registration, whose owner is tied to this lease's native teardown.
        let component = unsafe { node.with_native(|node| node.native_xcomponent()) }
            .flatten()
            .ok_or_else(|| {
                crate::CameraError::invalid_state(
                    "SurfaceRegistration::attach",
                    "XComponent is not mounted or has no native surface",
                )
            })?;
        let attachment = CameraXComponentAttachment::attach(component, sender)
            .map_err(crate::CameraError::from)?;
        Ok(Self {
            _attachment: attachment,
        })
    }
}
