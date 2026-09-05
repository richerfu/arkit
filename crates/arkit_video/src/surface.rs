use std::sync::mpsc::Sender;

use arkit_arkui::MountedNodeLease;
use ohos_native_window_binding::NativeWindow;
use ohos_xcomponent_binding::{NativeXComponent, WindowRaw, XComponentRaw};
use ohos_xcomponent_sys::OH_NativeXComponent_GetNativeXComponent;

use crate::worker::WorkerMessage;
use crate::{VideoError, VideoErrorKind, VideoResult};

/// Owns callback routing between one mounted XComponent and its AVPlayer worker.
pub(crate) struct SurfaceRegistration {
    registration_id: u64,
    component: NativeXComponent,
    liveness: Option<arkit_runtime::NativeLiveness>,
}

impl SurfaceRegistration {
    pub(crate) fn attach(
        node: &MountedNodeLease,
        registration_id: u64,
        sender: Sender<WorkerMessage>,
        liveness: Option<arkit_runtime::NativeLiveness>,
    ) -> VideoResult<Self> {
        // SAFETY: the lookup is synchronous inside a generation-checked native
        // borrow. This registration is torn down with the same mounted node.
        let raw = unsafe {
            node.with_native(|node| {
                OH_NativeXComponent_GetNativeXComponent(node.raw_handle().cast())
            })
        }
        .ok_or_else(|| {
            VideoError::new(
                VideoErrorKind::SurfaceUnavailable,
                "SurfaceRegistration::attach",
                "XComponent is no longer mounted",
            )
        })?;
        if raw.is_null() {
            return Err(VideoError::new(
                VideoErrorKind::SurfaceUnavailable,
                "OH_NativeXComponent_GetNativeXComponent",
                "ArkUI did not return a native XComponent handle",
            ));
        }

        let component = NativeXComponent::new(XComponentRaw(raw));
        component.id().map_err(|error| {
            VideoError::new(
                VideoErrorKind::SurfaceUnavailable,
                "NativeXComponent::id",
                error.to_string(),
            )
        })?;

        let created_sender = sender.clone();
        component.on_surface_created(move |component, window| {
            send_surface(&created_sender, registration_id, component, window);
            Ok(())
        });
        let changed_sender = sender.clone();
        component.on_surface_changed(move |component, window| {
            send_surface(&changed_sender, registration_id, component, window);
            Ok(())
        });
        let destroyed_sender = sender.clone();
        component.on_surface_destroyed(move |_, _| {
            let _ = destroyed_sender.send(WorkerMessage::SurfaceLost(registration_id));
            Ok(())
        });

        let registration = Self {
            registration_id,
            component,
            liveness,
        };
        registration
            .component
            .register_callback()
            .map_err(|error| {
                VideoError::new(
                    VideoErrorKind::SurfaceUnavailable,
                    "NativeXComponent::register_callback",
                    error.to_string(),
                )
            })?;
        registration.send_current_surface(registration_id, sender);
        Ok(registration)
    }

    pub(crate) const fn id(&self) -> u64 {
        self.registration_id
    }

    fn send_current_surface(&self, registration: u64, sender: Sender<WorkerMessage>) {
        let Some(window) = self.component.native_window() else {
            return;
        };
        let raw = WindowRaw(window.raw());
        let component = XComponentRaw(self.component.raw());
        if component.size(raw).is_ok() {
            send_surface(&sender, registration, component, raw);
        }
    }
}

impl Drop for SurfaceRegistration {
    fn drop(&mut self) {
        if !self
            .liveness
            .as_ref()
            .is_none_or(|liveness| liveness.is_alive())
        {
            return;
        }
        // Replacing the Rust callback slots releases their worker senders
        // before the native node can deliver any later lifecycle callback.
        self.component.on_surface_created(|_, _| Ok(()));
        self.component.on_surface_changed(|_, _| Ok(()));
        self.component.on_surface_destroyed(|_, _| Ok(()));
    }
}

fn send_surface(
    sender: &Sender<WorkerMessage>,
    registration: u64,
    component: XComponentRaw,
    window: WindowRaw,
) {
    if window.0.is_null() {
        return;
    }
    let Ok(size) = component.size(window) else {
        ohos_hilog_binding::error("arkit_video: failed to query XComponent surface size");
        return;
    };
    if size.width == 0 || size.height == 0 {
        return;
    }
    let native_window = NativeWindow::clone_from_ptr(window.0);
    let _ = sender.send(WorkerMessage::SurfaceAvailable {
        registration,
        surface: native_window,
    });
}
