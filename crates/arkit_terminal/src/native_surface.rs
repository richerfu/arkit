use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, OnceLock};

use arkit_arkui::MountedNodeLease;
use ohos_native_window_binding::NativeWindow;
use ohos_xcomponent_binding::{NativeXComponent, WindowRaw, XComponentRaw};
use ohos_xcomponent_sys::{
    OH_NativeXComponent, OH_NativeXComponent_GetNativeXComponent,
    OH_NativeXComponent_RegisterOnFrameCallback, OH_NativeXComponent_UnregisterOnFrameCallback,
};

use crate::worker::{schedule_vsync, WorkerMessage};
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

#[derive(Clone)]
struct FrameWake {
    sender: Sender<WorkerMessage>,
    pending: Arc<AtomicBool>,
}

fn frame_wakes() -> &'static Mutex<HashMap<usize, FrameWake>> {
    static FRAME_WAKES: OnceLock<Mutex<HashMap<usize, FrameWake>>> = OnceLock::new();
    FRAME_WAKES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn insert_frame_wake(key: usize, wake: FrameWake) {
    if let Ok(mut wakes) = frame_wakes().lock() {
        wakes.insert(key, wake);
    }
}

fn remove_frame_wake(key: usize) {
    if let Ok(mut wakes) = frame_wakes().lock() {
        wakes.remove(&key);
    }
}

fn frame_wake(key: usize) -> Option<FrameWake> {
    frame_wakes()
        .lock()
        .ok()
        .and_then(|wakes| wakes.get(&key).cloned())
}

// SAFETY: HarmonyOS invokes this from the XComponent vsync path with the
// same native handle registered in `SurfaceRegistration::attach`. The map
// lookup is mutex-protected; a late callback after unregistration is a no-op.
unsafe extern "C" fn on_native_frame(
    component: *mut OH_NativeXComponent,
    timestamp: u64,
    target_timestamp: u64,
) {
    let Some(wake) = frame_wake(component as usize) else {
        return;
    };
    schedule_vsync(&wake.sender, &wake.pending, timestamp, target_timestamp);
}

/// Owns the native callbacks associated with one mounted terminal XComponent.
pub(crate) struct SurfaceRegistration {
    component: NativeXComponent,
    frame_key: usize,
}

impl SurfaceRegistration {
    pub(crate) fn attach(
        node: &MountedNodeLease,
        sender: Sender<WorkerMessage>,
        vsync_pending: Arc<AtomicBool>,
    ) -> TerminalResult<Self> {
        // SAFETY: context lookup is synchronous inside the generation-checked
        // borrow. The returned XComponent is retained by the registration,
        // whose owner is tied to this lease's native teardown.
        let raw = unsafe {
            node.with_native(|node| {
                OH_NativeXComponent_GetNativeXComponent(node.raw_handle().cast())
            })
        }
        .ok_or_else(|| surface_error("XComponent is no longer mounted"))?;
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

        let frame_key = component.raw() as usize;
        insert_frame_wake(
            frame_key,
            FrameWake {
                sender: sender.clone(),
                pending: vsync_pending,
            },
        );
        // Direct C trampoline: the binding stores frame callbacks in
        // thread-local Rc, but HarmonyOS may deliver vsync off the UI
        // thread. A process map keyed by native handle is Send.
        // SAFETY: `component` is a live XComponent; `on_native_frame` only
        // looks up this handle in `FRAME_WAKES` and sends a coalesced wakeup.
        let frame_ret = unsafe {
            OH_NativeXComponent_RegisterOnFrameCallback(component.raw(), Some(on_native_frame))
        };
        if frame_ret != 0 {
            remove_frame_wake(frame_key);
            ohos_hilog_binding::error(
                "arkit_terminal: OH_NativeXComponent_RegisterOnFrameCallback failed",
            );
        } else if let Err(error) = component.set_frame_rate(30, 120, 120) {
            ohos_hilog_binding::error(format!(
                "arkit_terminal: failed to set XComponent frame rate: {error}"
            ));
        }

        let registration = Self {
            component,
            frame_key,
        };
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
        // SAFETY: the handle was registered in `attach` and is still owned
        // by this registration. Unregister before dropping the wake map entry
        // so a concurrent vsync cannot observe a half-removed mapping.
        let _ = unsafe { OH_NativeXComponent_UnregisterOnFrameCallback(self.component.raw()) };
        remove_frame_wake(self.frame_key);
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
