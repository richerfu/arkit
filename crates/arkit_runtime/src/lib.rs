//! Dioxus VirtualDom host for OpenHarmony/ArkUI.
//!
//! [`ArkRuntime`] owns a dioxus [`VirtualDom`] and an [`ArkUIRenderer`], wiring
//! the dioxus wake/render loop into the OpenHarmony UI loop.
//!
//! ## Lifecycle
//! 1. `ArkRuntime::from_virtual_dom(slot, app, dom)` installs the renderer and
//!    event sink, rebuilds the VirtualDom, and wires the OpenHarmony loop.
//! 2. Native events (registered by the renderer) call [`EventSink::dispatch`],
//!    which queues owned event data and wakes the OpenHarmony UI loop.
//! 3. Each UI tick forwards queued events into `Runtime::handle_event`, drains
//!    the resulting dioxus scheduler work with `render_immediate`, and then
//!    re-arms the scheduler wait. Native callbacks never re-enter a render.
//! 4. `unmount` detaches the renderer root from the slot.

use std::any::Any;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Once};
use std::task::{Context, Poll, Wake, Waker};

use arkit_arkui::{ArkUIRenderer, EventSink, NativeElementDelivery};
use dioxus_core::{ElementId, Runtime as DioxusRuntime};
use napi_ohos::{Error, Result};
use ohos_arkui_binding::common::handle::ArkUIHandle;
use ohos_arkui_binding::common::node::ArkUINode;
use openharmony_ability::{Event as AbilityEvent, OpenHarmonyApp, OpenHarmonyWaker};

mod lifecycle;
mod session;
mod webview;
mod window;

/// Runtime-local liveness of native nodes for hook-owned integrations.
///
/// Dioxus hooks that attach native resources to a node (XComponent callbacks,
/// surfaces, animators) must stop touching those resources once the node has
/// been destroyed outside the renderer (for example an embedded virtual-list
/// host that disappeared without removal callbacks). The renderer cannot know
/// about such destruction, so the runtime propagates a per-root liveness flag:
/// integrations read it during their teardown and skip native calls when dead.
#[derive(Clone, Default)]
pub struct NativeLiveness(std::rc::Rc<std::cell::Cell<bool>>);

impl NativeLiveness {
    /// Whether the native subtree backing this runtime is still alive.
    pub fn is_alive(&self) -> bool {
        self.0.get()
    }

    pub(crate) fn kill(&self) {
        self.0.set(false);
    }
}

pub use lifecycle::{
    ApplicationLifecycleEvent, ApplicationLifecycleHandle, ApplicationLifecyclePhase,
    ApplicationLifecycleState, ApplicationLifecycleSubscription,
};
pub use session::{use_runtime_handle, BackPressRegistration, RuntimeHandle, RuntimeId};
pub use webview::{EmbeddedWebViewController, EmbeddedWebViewInit, WebViewFrame, WebViewStyle};
pub use window::{
    EdgeInsets, PhysicalRect, SafeAreaPolicy, WindowMetrics, WindowMetricsHandle,
    WindowMetricsSubscription,
};

pub use dioxus_core::VirtualDom;

static NEXT_RUNTIME_ID: AtomicU64 = AtomicU64::new(1);

fn next_runtime_id() -> RuntimeId {
    RuntimeId::new(NEXT_RUNTIME_ID.fetch_add(1, Ordering::Relaxed))
}

fn run_ui_loop_effects(handle: &RuntimeHandle, runtime: &Rc<DioxusRuntime>) {
    for effect in handle.run_ui_effects() {
        // Effects are component-authored closures (e.g. `queue_ui` callbacks)
        // that touch dioxus state. They run from the NAPI event-loop callback,
        // which has no runtime on the thread-local stack, so install a guard
        // around each one like dioxus' own event dispatch does.
        let _guard = dioxus_core::RuntimeGuard::new(runtime.clone());
        if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(effect)) {
            log_panic_payload("ui_loop_effect", payload.as_ref());
            std::process::abort();
        }
    }
}

static PANIC_HOOK: Once = Once::new();

fn install_panic_hook() {
    PANIC_HOOK.call_once(|| {
        panic::set_hook(Box::new(|info| {
            ohos_hilog_binding::error(format!("arkit_runtime: panic: {info}"));
        }));
    });
}

fn log_panic_payload(context: &str, payload: &(dyn Any + Send)) {
    let message = payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-string panic payload".to_string());
    ohos_hilog_binding::error(format!("arkit_runtime: panic in {context}: {message}"));
}

#[cfg(debug_assertions)]
fn log_window_metrics(metrics: WindowMetrics) {
    ohos_hilog_binding::info(format!(
        "arkit_runtime: window_metrics content={:?} window={:?} scale={} safe={:?} gesture={:?} ime={:?} keyboard_height={}",
        metrics.content_rect,
        metrics.window_rect,
        metrics.scale,
        metrics.safe_area,
        metrics.gesture_area,
        metrics.ime_area,
        metrics.keyboard_height,
    ));
}

struct RuntimeInner {
    dom: VirtualDom,
    renderer: ArkUIRenderer,
}

struct EmbeddedRuntimeInner {
    dom: VirtualDom,
    renderer: ArkUIRenderer,
    sink: Rc<RuntimeEventSink>,
    liveness: NativeLiveness,
}

/// Wakes the OpenHarmony event loop when dioxus' scheduler receives work.
///
/// `VirtualDom::wait_for_work` registers this waker with its scheduler queue.
/// The future itself can then be dropped: the queue retains the waker and calls
/// it when signals, effects, events, or async tasks enqueue work.
struct DioxusUiWaker(OpenHarmonyWaker);

impl Wake for DioxusUiWaker {
    fn wake(self: Arc<Self>) {
        self.0.wake();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.wake();
    }
}

fn render_dom(inner: &Rc<RefCell<RuntimeInner>>) {
    let fault = {
        let mut borrowed = inner.borrow_mut();
        let RuntimeInner { dom, renderer } = &mut *borrowed;
        dom.render_immediate(renderer);
        renderer.finish_mutation_batch();
        renderer.take_fault()
    };
    if let Some(fault) = fault {
        panic!("arkit_runtime: native projection became inconsistent: {fault}");
    }
}

/// Poll dioxus' scheduler once, rendering only when work is ready.
///
/// A pending poll is intentional: it leaves `task_waker` registered with the
/// scheduler so work completed on another thread wakes the OpenHarmony loop.
fn render_ready_work(inner: &Rc<RefCell<RuntimeInner>>, task_waker: &Waker) {
    loop {
        let is_ready = {
            let mut borrowed = inner.borrow_mut();
            let mut wait_for_work = std::pin::pin!(borrowed.dom.wait_for_work());
            let mut context = Context::from_waker(task_waker);
            matches!(wait_for_work.as_mut().poll(&mut context), Poll::Ready(()))
        };

        if !is_ready {
            return;
        }
        render_dom(inner);
    }
}

fn render_embedded(inner: &Rc<RefCell<EmbeddedRuntimeInner>>) {
    let fault = {
        let mut borrowed = inner.borrow_mut();
        let EmbeddedRuntimeInner { dom, renderer, .. } = &mut *borrowed;
        dom.render_immediate(renderer);
        renderer.finish_mutation_batch();
        renderer.take_fault()
    };
    if let Some(fault) = fault {
        panic!("arkit_runtime: embedded native projection became inconsistent: {fault}");
    }
}

fn render_ready_embedded_work(inner: &Rc<RefCell<EmbeddedRuntimeInner>>, task_waker: &Waker) {
    loop {
        let is_ready = {
            let mut borrowed = inner.borrow_mut();
            let mut wait_for_work = std::pin::pin!(borrowed.dom.wait_for_work());
            let mut context = Context::from_waker(task_waker);
            matches!(wait_for_work.as_mut().poll(&mut context), Poll::Ready(()))
        };

        if !is_ready {
            return;
        }
        render_embedded(inner);
    }
}

fn pump_embedded_runtimes(handle: &RuntimeHandle) {
    let runtimes = handle.embedded_runtimes();
    if runtimes.is_empty() {
        return;
    }

    let task_waker = handle.scheduler_waker();
    for inner in runtimes {
        let (sink, runtime) = {
            let borrowed = inner.borrow();
            (borrowed.sink.clone(), borrowed.dom.runtime())
        };
        sink.dispatch_pending(&runtime);
        render_ready_embedded_work(&inner, &task_waker);
    }
}

/// Owns the dioxus VirtualDom and ArkUI renderer for one entry point.
pub struct ArkRuntime {
    inner: Rc<RefCell<RuntimeInner>>,
    handle: RuntimeHandle,
    async_runtime: Option<tokio::runtime::Runtime>,
}

/// Item-local Dioxus runtime projected into an adapter-owned native wrapper.
///
/// This runtime participates in the application's existing UI loop, so item
/// signals, hooks, async work, and event handlers remain live while ArkUI keeps
/// the virtual item mounted.
pub struct EmbeddedArkRuntime {
    registration: Option<session::EmbeddedRuntimeRegistration>,
    inner: Option<Rc<RefCell<EmbeddedRuntimeInner>>>,
}

impl EmbeddedArkRuntime {
    /// Schedule the embedded root for an in-place render.
    ///
    /// Adapter-backed containers use this when a retained item's logical
    /// index changes after an insert, removal, or move. The native wrapper and
    /// item-local hooks stay mounted while the subtree observes its new index.
    pub fn rerender(&self) {
        if let Some(inner) = &self.inner {
            inner.borrow_mut().dom.mark_dirty(dioxus_core::ScopeId::APP);
        }
    }

    /// Stop scheduling this runtime without touching an already-invalid native
    /// root.
    ///
    /// ArkUI normally emits item-removal events while wrappers are still live,
    /// so regular `Drop` performs full listener cleanup. This path is reserved
    /// for a host that disappeared without those callbacks. The runtime is
    /// released immediately instead of leaked: the renderer is switched to
    /// inert mode (no native calls during teardown), the liveness flag is
    /// killed so hook-owned integrations skip native unregistration, and the
    /// Dioxus state is dropped. Native resources that outlive the node
    /// (adapters, animators, N-API references) release themselves through
    /// their own Drop impls.
    pub fn abandon(mut self) {
        self.registration.take();
        if let Some(inner) = self.inner.take() {
            {
                let mut borrowed = inner.borrow_mut();
                borrowed.liveness.kill();
                borrowed.renderer.make_inert();
            }
            // Dropping `inner` tears down the embedded Dioxus tree and renderer
            // state without any native call on the dead host subtree.
        }
    }
}

struct PendingNativeEvent {
    name: &'static str,
    element: ElementId,
    payload: arkit_arkui::ArkEventPayload,
}

/// Native event boundary for one runtime.
///
/// ArkUI may invoke callbacks synchronously while the renderer is attaching or
/// patching native nodes. The callback therefore owns only this queue: touching
/// the `VirtualDom` here would re-enter it while `render_immediate` still has
/// exclusive access to [`RuntimeInner`]. The OpenHarmony UI loop drains the
/// queue at a phase boundary before starting the next render.
struct RuntimeEventSink {
    handle: RuntimeHandle,
    pending: RefCell<VecDeque<PendingNativeEvent>>,
    draining: RefCell<VecDeque<PendingNativeEvent>>,
    pending_native_refs: RefCell<VecDeque<NativeElementDelivery>>,
    draining_native_refs: RefCell<VecDeque<NativeElementDelivery>>,
}

impl RuntimeEventSink {
    fn new(handle: RuntimeHandle) -> Self {
        Self {
            handle,
            pending: RefCell::new(VecDeque::new()),
            draining: RefCell::new(VecDeque::new()),
            pending_native_refs: RefCell::new(VecDeque::new()),
            draining_native_refs: RefCell::new(VecDeque::new()),
        }
    }

    fn enqueue(&self, event: PendingNativeEvent) {
        let mut pending = self.pending.borrow_mut();
        if let Some(pointer) = pointer_payload(&event)
            .filter(|pointer| pointer.action == dioxus_elements::event::PointerAction::Move)
        {
            let mut replace_index = None;
            for (index, queued) in pending.iter().enumerate().rev() {
                let Some(queued_pointer) = pointer_payload(queued) else {
                    continue;
                };
                let same_stream = queued.name == event.name
                    && queued.element == event.element
                    && queued_pointer.pointer_id == pointer.pointer_id;
                if !same_stream {
                    continue;
                }
                if queued_pointer.action == dioxus_elements::event::PointerAction::Move {
                    replace_index = Some(index);
                }
                break;
            }
            if let Some(index) = replace_index {
                // Remove the earlier sample and append the latest one so event
                // ordering relative to other input streams remains correct.
                pending.remove(index);
            }
        }
        pending.push_back(event);
    }

    fn dispatch_pending(&self, runtime: &Rc<DioxusRuntime>) {
        {
            let mut pending = self.pending.borrow_mut();
            let mut draining = self.draining.borrow_mut();
            debug_assert!(draining.is_empty());
            std::mem::swap(&mut *pending, &mut *draining);
        }
        while let Some(PendingNativeEvent {
            name,
            element,
            payload,
        }) = self.draining.borrow_mut().pop_front()
        {
            let data: Rc<dyn Any> =
                Rc::new(dioxus_elements::event::ArkEventData::with_payload(payload));
            let event = dioxus_core::Event::new(data, event_bubbles(name));
            runtime.handle_event(name, event, element);
        }
        {
            let mut pending = self.pending_native_refs.borrow_mut();
            let mut draining = self.draining_native_refs.borrow_mut();
            debug_assert!(draining.is_empty());
            std::mem::swap(&mut *pending, &mut *draining);
        }
        while let Some(delivery) = self.draining_native_refs.borrow_mut().pop_front() {
            delivery.deliver();
        }
    }
}

impl EventSink for RuntimeEventSink {
    fn dispatch(
        &self,
        name: &'static str,
        element: ElementId,
        payload: arkit_arkui::ArkEventPayload,
    ) {
        if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(|| {
            self.enqueue(PendingNativeEvent {
                name,
                element,
                payload,
            });
            self.handle.wake();
        })) {
            log_panic_payload("event_dispatch", payload.as_ref());
            std::process::abort();
        }
    }

    fn dispatch_native_ref(&self, delivery: NativeElementDelivery) {
        if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(|| {
            self.pending_native_refs.borrow_mut().push_back(delivery);
            self.handle.wake();
        })) {
            log_panic_payload("native_ref_dispatch", payload.as_ref());
            std::process::abort();
        }
    }
}

fn pointer_payload(event: &PendingNativeEvent) -> Option<&dioxus_elements::event::PointerPayload> {
    match &event.payload {
        arkit_arkui::ArkEventPayload::Pointer(pointer) => Some(pointer),
        _ => None,
    }
}

fn event_bubbles(name: &str) -> bool {
    dioxus_elements::event::classify_event_name(name).is_some_and(|kind| kind.bubbles())
}

impl ArkRuntime {
    /// Create and mount a runtime from an already-configured dioxus
    /// [`VirtualDom`].
    ///
    /// This is the native-renderer boundary used by higher-level launchers that
    /// need root props or context wrappers. The runtime owns the VirtualDom
    /// directly; it does not reconstruct or reinterpret the component tree.
    pub fn from_virtual_dom(
        slot: ArkUIHandle,
        app: OpenHarmonyApp,
        dom: VirtualDom,
    ) -> Result<Self> {
        Self::from_virtual_dom_with_policy(slot, app, dom, SafeAreaPolicy::Safe)
    }

    /// Create and mount a runtime with an explicit root safe-area policy.
    pub fn from_virtual_dom_with_policy(
        slot: ArkUIHandle,
        app: OpenHarmonyApp,
        dom: VirtualDom,
        safe_area_policy: SafeAreaPolicy,
    ) -> Result<Self> {
        let async_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|error| {
                Error::from_reason(format!("failed to build async runtime: {error}"))
            })?;
        Self::from_virtual_dom_with_policy_and_runtime(
            slot,
            app,
            dom,
            safe_area_policy,
            async_runtime,
        )
    }

    /// Mount with a caller-configured Tokio runtime. The runtime is owned by
    /// this `ArkRuntime` and shuts down in the background on unmount/drop, so
    /// tasks cannot silently outlive the UI owner.
    pub fn from_virtual_dom_with_policy_and_runtime(
        slot: ArkUIHandle,
        app: OpenHarmonyApp,
        mut dom: VirtualDom,
        safe_area_policy: SafeAreaPolicy,
        async_runtime: tokio::runtime::Runtime,
    ) -> Result<Self> {
        install_panic_hook();
        let runtime_handle = RuntimeHandle::new(next_runtime_id(), async_runtime.handle().clone());

        let mut renderer = ArkUIRenderer::new(slot).map_err(map_arkui_error)?;

        // Install the queueing event sink before rebuild so every listener
        // captures the same phase-isolated native event boundary.
        let sink = Rc::new(RuntimeEventSink::new(runtime_handle.clone()));
        renderer.set_sink(sink.clone());

        // Window state is owned by the native runtime and provided before the
        // first rebuild so the framework root and every business component see
        // the same initial snapshot.
        let window_metrics = WindowMetricsHandle::new(WindowMetrics::from_app(&app, None));
        let application_lifecycle =
            ApplicationLifecycleHandle::new(ApplicationLifecycleState::default());
        #[cfg(debug_assertions)]
        log_window_metrics(window_metrics.get());
        dom.provide_root_context(window_metrics.clone());
        dom.provide_root_context(application_lifecycle.clone());
        dom.provide_root_context(safe_area_policy);
        dom.provide_root_context(runtime_handle.clone());
        // Root-level liveness for hook-owned native integrations. The renderer
        // owns this root's native lifetime, so it stays alive until unmount;
        // embedded runtimes override their own flag when a host disappears.
        dom.provide_root_context(NativeLiveness::default());

        // Initial mount: build the real DOM tree onto the slot.
        dom.rebuild(&mut renderer);
        renderer.finish_mutation_batch();

        let weak_runtime = Rc::downgrade(&dom.runtime());
        let inner = Rc::new(RefCell::new(RuntimeInner { dom, renderer }));

        // One-shot EventOnAppear replays route back into the renderer so
        // declarative attrs are reapplied after ArkUI control skins settle,
        // before the node's first paint (single-frame convergence).
        let replay_inner = Rc::downgrade(&inner);
        inner
            .borrow_mut()
            .renderer
            .set_appear_replay_handler(Rc::new(move |element| {
                if let Some(inner) = replay_inner.upgrade() {
                    if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(|| {
                        inner.borrow_mut().renderer.replay_element_attrs(element);
                    })) {
                        log_panic_payload("appear_replay", payload.as_ref());
                    }
                }
            }));

        // Bridge dioxus' scheduler to OpenHarmony. A pending
        // `VirtualDom::wait_for_work` poll retains this waker, including when a
        // task is later completed by the background tokio runtime.
        let waker = app.create_waker();
        let task_waker = Waker::from(Arc::new(DioxusUiWaker(waker.clone())));
        runtime_handle.set_scheduler_waker(task_waker.clone());
        runtime_handle.set_ui_waker(Rc::new({
            let waker = waker.clone();
            move || waker.wake()
        }));

        // Run imperative UI closures and queued native events first, then let
        // dioxus render every piece of scheduler work they made ready.
        // Process-lifetime closures capture only weak references: once this
        // runtime is retired, the OpenHarmonyApp closures become inert no-ops
        // instead of pinning the session/sink/metrics state forever.
        let weak_inner = Rc::downgrade(&inner);
        let loop_task_waker = task_waker.clone();
        let loop_sink = Rc::downgrade(&sink);
        let loop_metrics = window_metrics.downgrade();
        let loop_lifecycle = application_lifecycle.downgrade();
        let loop_runtime = runtime_handle.downgrade();
        let metrics_app = app.clone();
        let mut keyboard_height_px = None;
        app.run_loop(move |event| {
            let is_user_event = matches!(&event, AbilityEvent::UserEvent);
            let lifecycle_event = ApplicationLifecycleHandle::handles_ability_event(&event);
            let refresh_window_metrics = matches!(
                &event,
                AbilityEvent::WindowCreate
                    | AbilityEvent::SurfaceCreate
                    | AbilityEvent::WindowResize(_)
                    | AbilityEvent::ContentRectChange(_)
                    | AbilityEvent::AvoidAreaChange(_)
                    | AbilityEvent::ConfigChanged(_)
                    | AbilityEvent::KeyboardEvent(_)
            );

            if let AbilityEvent::KeyboardEvent(height) = &event {
                // Debounce keyboard height changes: the keyboard show/hide
                // animation emits many intermediate heights, and each metrics
                // change currently marks the whole tree dirty. Only significant
                // steps (>= 1vp) propagate to avoid a re-render storm that
                // fights the keyboard animation visually.
                let step = height - keyboard_height_px.unwrap_or(*height);
                if step.abs() >= 1 {
                    keyboard_height_px = Some(*height);
                }
            }

            if is_user_event || refresh_window_metrics || lifecycle_event {
                if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(|| {
                    if let Some(inner) = weak_inner.upgrade() {
                        if lifecycle_event {
                            if let Some(lifecycle) = loop_lifecycle.upgrade() {
                                ApplicationLifecycleHandle::from_inner(lifecycle)
                                    .update_from_ability_event(&event);
                            }
                        }
                        if refresh_window_metrics {
                            let next = WindowMetrics::from_app(&metrics_app, keyboard_height_px);
                            if let Some(metrics) = loop_metrics.upgrade() {
                                if WindowMetricsHandle::from_inner(metrics).update(next) {
                                    inner.borrow_mut().dom.mark_all_dirty();
                                }
                            }
                        }
                        if is_user_event {
                            if let Some(handle) = loop_runtime.upgrade() {
                                let handle = RuntimeHandle::from_inner(handle);
                                if let Some(runtime) = weak_runtime.upgrade() {
                                    run_ui_loop_effects(&handle, &runtime);
                                    if let Some(sink) = loop_sink.upgrade() {
                                        sink.dispatch_pending(&runtime);
                                    }
                                }
                                pump_embedded_runtimes(&handle);
                            }
                        }
                        render_ready_work(&inner, &loop_task_waker);
                        if let Some(handle) = loop_runtime.upgrade() {
                            pump_embedded_runtimes(&RuntimeHandle::from_inner(handle));
                        }
                    }
                })) {
                    log_panic_payload("ui_loop", payload.as_ref());
                    std::process::abort();
                }
            }
        });

        // Wire the OHOS back button to this root's handler stack (registered
        // by `arkit_router::use_back_handler` or another component).
        // Walk newest-to-oldest so an inactive overlay can pass the event to
        // the next active overlay or router handler. The intercept closure is
        // process-lifetime; it holds a weak session so a retired runtime
        // becomes a no-op instead of leaking.
        let back_runtime = runtime_handle.downgrade();
        app.on_back_press_intercept(move || {
            back_runtime.upgrade().is_some_and(|state| {
                RuntimeHandle::from_inner(state).dispatch_back_press()
            })
        });

        // `rebuild` does not finish a render cycle, so run one immediate pass
        // to publish mount-time effects. Then drain exactly the scheduler work
        // that is ready and leave a pending wait armed for future async work.
        render_dom(&inner);
        render_ready_work(&inner, &task_waker);

        Ok(Self {
            inner,
            handle: runtime_handle,
            async_runtime: Some(async_runtime),
        })
    }

    /// Consume this runtime and unmount its root from the NodeContent slot.
    pub fn unmount(self) -> Result<()> {
        // An explicitly unmounted root must no longer accept queued effects,
        // async wakeups, back handlers, or embedded-runtime scheduling.
        self.handle.close();
        let mut borrowed = self.inner.borrow_mut();
        borrowed.renderer.unmount().map_err(map_arkui_error)
    }

    pub fn handle(&self) -> RuntimeHandle {
        self.handle.clone()
    }
}

impl Drop for ArkRuntime {
    fn drop(&mut self) {
        self.handle.close();
        if let Some(runtime) = self.async_runtime.take() {
            runtime.shutdown_background();
        }
    }
}

/// Mount a Dioxus subtree directly into an existing native node.
///
/// The caller retains ownership of `root` and must drop the returned runtime
/// before disposing that node.
pub fn mount_embedded_virtual_dom(
    root: Rc<RefCell<ArkUINode>>,
    mut dom: VirtualDom,
    runtime_handle: RuntimeHandle,
) -> EmbeddedArkRuntime {
    let mut renderer = ArkUIRenderer::new_embedded(root);
    let sink = Rc::new(RuntimeEventSink::new(runtime_handle.clone()));
    renderer.set_sink(sink.clone());
    dom.provide_root_context(runtime_handle.clone());
    let liveness = NativeLiveness::default();
    dom.provide_root_context(liveness.clone());

    dom.rebuild(&mut renderer);
    renderer.finish_mutation_batch();

    let inner = Rc::new(RefCell::new(EmbeddedRuntimeInner {
        dom,
        renderer,
        sink,
        liveness,
    }));

    // Same single-frame appear replay as the root runtime: item subtrees also
    // converge on declarative styles before their first paint.
    let replay_inner = Rc::downgrade(&inner);
    inner
        .borrow_mut()
        .renderer
        .set_appear_replay_handler(Rc::new(move |element| {
            if let Some(inner) = replay_inner.upgrade() {
                if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(|| {
                    inner.borrow_mut().renderer.replay_element_attrs(element);
                })) {
                    log_panic_payload("embedded_appear_replay", payload.as_ref());
                }
            }
        }));
    let registration = runtime_handle.register_embedded(Rc::downgrade(&inner));

    render_embedded(&inner);
    let task_waker = runtime_handle.scheduler_waker();
    render_ready_embedded_work(&inner, &task_waker);

    EmbeddedArkRuntime {
        registration: Some(registration),
        inner: Some(inner),
    }
}

/// Mount an already-configured dioxus [`VirtualDom`] into a NodeContent slot.
pub fn mount_virtual_dom(
    slot: ArkUIHandle,
    app: OpenHarmonyApp,
    dom: VirtualDom,
) -> Result<ArkRuntime> {
    ArkRuntime::from_virtual_dom(slot, app, dom)
}

/// Mount a VirtualDom with an explicit root safe-area policy.
pub fn mount_virtual_dom_with_policy(
    slot: ArkUIHandle,
    app: OpenHarmonyApp,
    dom: VirtualDom,
    safe_area_policy: SafeAreaPolicy,
) -> Result<ArkRuntime> {
    ArkRuntime::from_virtual_dom_with_policy(slot, app, dom, safe_area_policy)
}

fn map_arkui_error<E: ToString>(error: E) -> Error {
    Error::from_reason(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn test_runtime_handle() -> (tokio::runtime::Runtime, RuntimeHandle) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("test tokio runtime");
        let handle = RuntimeHandle::new(next_runtime_id(), runtime.handle().clone());
        (runtime, handle)
    }

    #[test]
    fn back_press_falls_through_inactive_newer_handlers() {
        let (_runtime, handle) = test_runtime_handle();
        let older_calls = Rc::new(Cell::new(0));
        let newer_calls = Rc::new(Cell::new(0));
        let older_counter = older_calls.clone();
        let newer_counter = newer_calls.clone();
        let _older = handle.register_back_handler(Rc::new(move || {
            older_counter.set(older_counter.get() + 1);
            true
        }));
        let _newer = handle.register_back_handler(Rc::new(move || {
            newer_counter.set(newer_counter.get() + 1);
            false
        }));

        assert!(handle.dispatch_back_press());
        assert_eq!(newer_calls.get(), 1);
        assert_eq!(older_calls.get(), 1);
    }

    #[test]
    fn back_press_stops_after_the_first_consuming_handler() {
        let (_runtime, handle) = test_runtime_handle();
        let older_calls = Rc::new(Cell::new(0));
        let newer_calls = Rc::new(Cell::new(0));
        let older_counter = older_calls.clone();
        let newer_counter = newer_calls.clone();
        let _older = handle.register_back_handler(Rc::new(move || {
            older_counter.set(older_counter.get() + 1);
            true
        }));
        let _newer = handle.register_back_handler(Rc::new(move || {
            newer_counter.set(newer_counter.get() + 1);
            true
        }));

        assert!(handle.dispatch_back_press());
        assert_eq!(newer_calls.get(), 1);
        assert_eq!(older_calls.get(), 0);
    }

    #[test]
    fn back_handlers_are_isolated_per_root() {
        let (_first_runtime, first) = test_runtime_handle();
        let (_second_runtime, second) = test_runtime_handle();
        let calls = Rc::new(Cell::new(0));
        let callback_calls = calls.clone();
        let _registration = first.register_back_handler(Rc::new(move || {
            callback_calls.set(callback_calls.get() + 1);
            true
        }));

        assert!(!second.dispatch_back_press());
        assert_eq!(calls.get(), 0);
        assert!(first.dispatch_back_press());
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn closed_runtime_rejects_new_back_handlers() {
        let (_runtime, handle) = test_runtime_handle();
        let calls = Rc::new(Cell::new(0));
        handle.close();

        let callback_calls = calls.clone();
        let _registration = handle.register_back_handler(Rc::new(move || {
            callback_calls.set(callback_calls.get() + 1);
            true
        }));

        assert!(!handle.dispatch_back_press());
        assert_eq!(calls.get(), 0);
    }
}
