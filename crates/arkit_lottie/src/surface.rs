use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Instant;

use ohos_arkui_binding::common::node::ArkUINode;
use ohos_native_window_binding::NativeWindow;
use ohos_xcomponent_binding::{NativeXComponent, WindowRaw, XComponentRaw};
use ohos_xcomponent_sys::{
    OH_NativeXComponent_GetNativeXComponent, OH_NativeXComponent_UnregisterOnFrameCallback,
};

use crate::worker::WorkerMessage;
use crate::{LottieError, LottieErrorKind, LottieResult};

/// Owns callback routing between one mounted XComponent and its render worker.
pub(crate) struct SurfaceRegistration {
    component: NativeXComponent,
}

impl SurfaceRegistration {
    pub(crate) fn attach(
        node: &ArkUINode,
        sender: Sender<WorkerMessage>,
        tick_pending: Arc<AtomicBool>,
        frames_per_second: u16,
    ) -> LottieResult<Self> {
        // SAFETY: `node` is the mounted ArkUI XComponent resolved by
        // `use_ark_node`; the renderer keeps it alive for this registration.
        let raw = unsafe { OH_NativeXComponent_GetNativeXComponent(node.raw_handle().cast()) };
        if raw.is_null() {
            return Err(LottieError::new(
                LottieErrorKind::SurfaceUnavailable,
                "OH_NativeXComponent_GetNativeXComponent",
                "ArkUI did not return a native XComponent handle",
            ));
        }
        let component = NativeXComponent::new(XComponentRaw(raw));
        component.id().map_err(|error| {
            LottieError::new(
                LottieErrorKind::SurfaceUnavailable,
                "NativeXComponent::id",
                error.to_string(),
            )
        })?;

        let created_sender = sender.clone();
        component.on_surface_created(move |component, window| {
            send_surface(&created_sender, component, window);
            Ok(())
        });
        let changed_sender = sender.clone();
        component.on_surface_changed(move |component, window| {
            send_surface(&changed_sender, component, window);
            Ok(())
        });
        let destroyed_sender = sender.clone();
        component.on_surface_destroyed(move |_, _| {
            let _ = destroyed_sender.send(WorkerMessage::SurfaceLost);
            Ok(())
        });
        let registration = Self { component };
        registration
            .component
            .register_callback()
            .map_err(|error| {
                LottieError::new(
                    LottieErrorKind::SurfaceUnavailable,
                    "NativeXComponent::register_callback",
                    error.to_string(),
                )
            })?;

        let frame_sender = sender.clone();
        registration
            .component
            .on_frame_callback(move |_, _, _| {
                if tick_pending
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                    && frame_sender
                        .send(WorkerMessage::Tick(Instant::now()))
                        .is_err()
                {
                    tick_pending.store(false, Ordering::Release);
                }
                Ok(())
            })
            .map_err(|error| {
                LottieError::new(
                    LottieErrorKind::SurfaceUnavailable,
                    "NativeXComponent::on_frame_callback",
                    error.to_string(),
                )
            })?;

        registration.set_frame_rate(frames_per_second)?;
        registration.send_current_surface(sender);
        Ok(registration)
    }

    pub(crate) fn set_frame_rate(&self, frames_per_second: u16) -> LottieResult<()> {
        let fps = i32::from(frames_per_second.clamp(1, 120));
        self.component.set_frame_rate(1, fps, fps).map_err(|error| {
            LottieError::new(
                LottieErrorKind::SurfaceUnavailable,
                "NativeXComponent::set_frame_rate",
                error.to_string(),
            )
        })
    }

    fn send_current_surface(&self, sender: Sender<WorkerMessage>) {
        let Some(window) = self.component.native_window() else {
            return;
        };
        let raw = WindowRaw(window.raw());
        // `native_window()` is currently global in the binding. Size lookup
        // validates that the window actually belongs to this XComponent before
        // it is routed to the worker.
        let component = XComponentRaw(self.component.raw());
        if component.size(raw).is_ok() {
            send_surface(&sender, component, raw);
        }
    }
}

impl Drop for SurfaceRegistration {
    fn drop(&mut self) {
        self.component.on_surface_created(|_, _| Ok(()));
        self.component.on_surface_changed(|_, _| Ok(()));
        self.component.on_surface_destroyed(|_, _| Ok(()));
        // Replace the multi-mode callback-map closure so it releases its worker
        // sender, then unregister native frame delivery.
        let _ = self.component.on_frame_callback(|_, _, _| Ok(()));
        // SAFETY: `component.raw()` remains valid while the mounted native node
        // and this registration are alive. Unregister is idempotent for this
        // callback slot and prevents delivery after component teardown.
        unsafe {
            OH_NativeXComponent_UnregisterOnFrameCallback(self.component.raw());
        }
    }
}

fn send_surface(sender: &Sender<WorkerMessage>, component: XComponentRaw, window: WindowRaw) {
    if window.0.is_null() {
        return;
    }
    let Ok(size) = component.size(window) else {
        ohos_hilog_binding::error("arkit_lottie: failed to query XComponent surface size");
        return;
    };
    let (Ok(width), Ok(height)) = (i32::try_from(size.width), i32::try_from(size.height)) else {
        ohos_hilog_binding::error("arkit_lottie: XComponent surface size exceeds i32");
        return;
    };
    if width <= 0 || height <= 0 {
        return;
    }
    let native_window = NativeWindow::clone_from_ptr(window.0);
    // OH_NativeWindow_NativeWindowRequestBuffer requires geometry to be set
    // first. Relying on the XComponent's implicit queue geometry can expose a
    // buffer whose allocation is smaller than its reported stride/height on
    // physical devices, causing ThorVG to write beyond the mapped allocation.
    if let Err(error) = native_window.set_buffer_geometry(width, height) {
        ohos_hilog_binding::error(format!(
            "arkit_lottie: failed to set native window geometry: {error:?}"
        ));
        return;
    }
    let _ = sender.send(WorkerMessage::SurfaceAvailable(native_window));
}
