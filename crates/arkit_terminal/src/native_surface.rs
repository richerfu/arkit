use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::mpsc::Sender;

use ohos_arkui_binding::common::node::ArkUINode;
use ohos_native_window_binding::NativeWindow;
use ohos_xcomponent_binding::{NativeXComponent, WindowRaw, XComponentRaw};
use ohos_xcomponent_sys::OH_NativeXComponent_GetNativeXComponent;

use crate::worker::WorkerMessage;
use crate::{TerminalError, TerminalErrorKind, TerminalResult};

/// XComponent window reference transferred to the renderer thread.
///
/// The owned `NativeWindow` reference keeps `raw_window` valid until the GPU
/// drawing surface has been destroyed.
pub(crate) struct NativeSurface {
    _window: NativeWindow,
    raw_window: NonNull<c_void>,
    width: i32,
    height: i32,
}

impl NativeSurface {
    fn new(window: NativeWindow, raw_window: *mut c_void, width: i32, height: i32) -> Self {
        Self {
            _window: window,
            raw_window: NonNull::new(raw_window).expect("XComponent window was checked above"),
            width,
            height,
        }
    }

    pub(crate) fn raw_window(&self) -> *mut c_void {
        self.raw_window.as_ptr()
    }

    pub(crate) fn width(&self) -> i32 {
        self.width
    }

    pub(crate) fn height(&self) -> i32 {
        self.height
    }
}

// SAFETY: `OHNativeWindow` is reference-counted by `_window`, and the OHOS
// native-window API permits a retained window to be consumed by a render
// thread. All access after transfer is serialized by that single thread.
unsafe impl Send for NativeSurface {}

/// Owns the native callbacks associated with one mounted terminal XComponent.
pub(crate) struct SurfaceRegistration {
    component: NativeXComponent,
}

impl SurfaceRegistration {
    pub(crate) fn attach(node: &ArkUINode, sender: Sender<WorkerMessage>) -> TerminalResult<Self> {
        // SAFETY: `node` is the mounted XComponent resolved by `use_ark_node`
        // and remains mounted while this registration is retained.
        let raw = unsafe { OH_NativeXComponent_GetNativeXComponent(node.raw_handle().cast()) };
        if raw.is_null() {
            return Err(surface_error(
                "ArkUI did not return a native XComponent handle",
            ));
        }
        let component = NativeXComponent::new(XComponentRaw(raw));
        component
            .id()
            .map_err(|error| surface_error(error.to_string()))?;

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
        component
            .register_callback()
            .map_err(|error| surface_error(error.to_string()))?;

        let registration = Self { component };
        registration.send_current_surface(sender);
        Ok(registration)
    }

    fn send_current_surface(&self, sender: Sender<WorkerMessage>) {
        let Some(window) = self.component.native_window() else {
            return;
        };
        let window = WindowRaw(window.raw());
        let component = XComponentRaw(self.component.raw());
        if component.size(window).is_ok() {
            send_surface(&sender, component, window);
        }
    }
}

impl Drop for SurfaceRegistration {
    fn drop(&mut self) {
        // Replacing the callbacks releases all worker senders before the native
        // component can deliver a late lifecycle event during teardown.
        self.component.on_surface_created(|_, _| Ok(()));
        self.component.on_surface_changed(|_, _| Ok(()));
        self.component.on_surface_destroyed(|_, _| Ok(()));
    }
}

fn send_surface(sender: &Sender<WorkerMessage>, component: XComponentRaw, window: WindowRaw) {
    if window.0.is_null() {
        return;
    }
    let Ok(size) = component.size(window) else {
        ohos_hilog_binding::error("arkit_terminal: failed to query XComponent surface size");
        return;
    };
    let (Ok(width), Ok(height)) = (i32::try_from(size.width), i32::try_from(size.height)) else {
        ohos_hilog_binding::error("arkit_terminal: XComponent surface size exceeds i32");
        return;
    };
    if width <= 0 || height <= 0 {
        return;
    }

    let native_window = NativeWindow::clone_from_ptr(window.0);
    if let Err(error) = native_window.set_buffer_geometry(width, height) {
        ohos_hilog_binding::error(format!(
            "arkit_terminal: failed to set native window geometry: {error:?}"
        ));
        return;
    }
    let surface = NativeSurface::new(native_window, window.0, width, height);
    let _ = sender.send(WorkerMessage::SurfaceAvailable(surface));
}

fn surface_error(message: impl Into<String>) -> TerminalError {
    TerminalError::new(TerminalErrorKind::Io, message)
}
