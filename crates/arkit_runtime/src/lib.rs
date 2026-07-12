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
use std::sync::{Arc, Once};
use std::task::{Context, Poll, Wake, Waker};

use arkit_arkui::{ArkUIRenderer, EventSink};
use dioxus_core::{DynamicNode, ElementId, Runtime as DioxusRuntime, VNode};
use napi_ohos::{Error, Result};
use ohos_arkui_binding::common::handle::ArkUIHandle;
use openharmony_ability::{Event as AbilityEvent, OpenHarmonyApp, OpenHarmonyWaker};

mod webview;
mod window;

pub use webview::{EmbeddedWebViewController, EmbeddedWebViewInit, WebViewFrame, WebViewStyle};
pub use window::{
    EdgeInsets, PhysicalRect, SafeAreaPolicy, WindowMetrics, WindowMetricsHandle,
    WindowMetricsSubscription,
};

pub use dioxus_core::VirtualDom;

// ---------------------------------------------------------------------------
// UI loop machinery
// ---------------------------------------------------------------------------

thread_local! {
    static UI_LOOP_EFFECTS: RefCell<Vec<Box<dyn FnOnce()>>> = RefCell::new(Vec::new());
    static UI_WAKER: RefCell<Option<Rc<dyn Fn()>>> = RefCell::new(None);
}

/// A lazily-initialized, process-wide multi-threaded tokio runtime backing
/// dioxus async hooks (`use_resource`/`use_future`/`use_coroutine`).
///
/// dioxus drives task futures on its own scheduler, but a future that awaits
/// tokio I/O / timers (e.g. `tokio::time::sleep`) needs a tokio runtime
/// context. [`tokio_handle`] returns the handle of the runtime started here;
/// spawn blocking/timer work onto it from inside `use_resource` closures.
static TOKIO_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

fn tokio_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("arkit_runtime: failed to build tokio runtime")
    })
}

/// The handle of the framework's tokio runtime. Use it to await timers / I/O
/// inside dioxus async hooks, e.g.:
///
/// ```ignore
/// let handle = arkit_runtime::tokio_handle();
/// let _ = use_resource(move || async move {
///     handle.spawn(async move { tokio::time::sleep(Duration::from_millis(800)).await }).await.unwrap();
///     "done".to_string()
/// });
/// ```
pub fn tokio_handle() -> tokio::runtime::Handle {
    tokio_runtime().handle().clone()
}

thread_local! {
    /// Global back-press handler registered by `arkit_router::use_back_handler`
    /// (or any component). Consumed by the OHOS back-button interceptor wired
    /// in `ArkRuntime::new`. Returns `true` to consume the back press.
    static BACK_PRESS_HANDLER: RefCell<Option<Rc<dyn Fn() -> bool>>> = RefCell::new(None);
}

/// Register (or clear) the global OHOS back-press handler. Called by
/// `arkit_router::use_back_handler` on mount. The handler returns `true` to
/// consume the back press (e.g. navigate back in the router history).
pub fn set_back_press_handler(handler: Option<Rc<dyn Fn() -> bool>>) {
    BACK_PRESS_HANDLER.with(|state| {
        state.replace(handler);
    });
}

fn set_ui_waker(waker: Option<Rc<dyn Fn()>>) {
    UI_WAKER.with(|state| {
        state.replace(waker);
    })
}

/// Queue a closure to run on the next UI loop tick, and wake the UI waker.
pub fn queue_ui_loop(effect: impl FnOnce() + 'static) {
    UI_LOOP_EFFECTS.with(|state| {
        state.borrow_mut().push(Box::new(effect));
    });
    wake_ui_loop();
}

fn wake_ui_loop() {
    UI_WAKER.with(|state| {
        if let Some(waker) = state.borrow().as_ref() {
            waker();
        }
    });
}

fn run_ui_loop_effects() {
    let effects = UI_LOOP_EFFECTS.with(|state| state.replace(Vec::new()));
    for effect in effects {
        if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(effect)) {
            log_panic_payload("ui_loop_effect", payload.as_ref());
            std::process::abort();
        }
    }
}

fn clear_ui_loop_effects() {
    UI_LOOP_EFFECTS.with(|state| {
        state.borrow_mut().clear();
    });
}

// ---------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------

/// Bridge resolving `use_ark_node` / overlay portal requests after each render.
///
/// dioxus 0.7 does not expose a `ScopeId → ElementId` mapping to component
/// bodies, so `arkit_hooks::use_ark_node` registers pending scope lookups on an
/// [`ArkHost`](arkit_hooks) context. The runtime owns the `VirtualDom` (which
/// *can* map `ScopeId → root ElementId` via `mounted_root`), so after each
/// render it asks the resolver for pending scopes, resolves each to its
/// mounted node, and writes it back.
///
/// Implemented by `arkit_hooks::ArkHost` (which lives in a crate that depends
/// on `arkit_runtime`, so the runtime holds it behind this trait to avoid a
/// circular dependency).
pub trait ScopeNodeResolver {
    /// Snapshot of scopes awaiting node resolution.
    fn pending_scopes(&self) -> Vec<dioxus_core::ScopeId>;
    /// Deliver the mounted native node (shared `Rc` handle — the same one
    /// mounted in the ArkUI tree and used as the event-dispatch user-data
    /// target) for a scope, writing it into the hook's signal slot.
    fn resolve_scope(
        &self,
        scope: dioxus_core::ScopeId,
        node: std::rc::Rc<std::cell::RefCell<ohos_arkui_binding::common::node::ArkUINode>>,
    );
}

thread_local! {
    static SCOPE_RESOLVER: RefCell<Option<Rc<dyn ScopeNodeResolver>>> = RefCell::new(None);
}

/// Install (or clear) the scope-node resolver. Called by
/// `arkit_hooks::use_ark_host_provider` once the `ArkHost` context exists.
pub fn set_scope_resolver(resolver: Option<Rc<dyn ScopeNodeResolver>>) {
    SCOPE_RESOLVER.with(|state| {
        state.replace(resolver);
    });
}

/// Resolve all pending `use_ark_node` lookups against the freshly-rendered
/// VirtualDom. Called after each `render_immediate` / `rebuild`.
fn resolve_pending(dom: &VirtualDom, renderer: &ArkUIRenderer) {
    let Some(resolver) = SCOPE_RESOLVER.with(|state| state.borrow().clone()) else {
        return;
    };

    for scope in resolver.pending_scopes() {
        let node_id_opt = dom
            .get_scope(scope)
            .and_then(|s| first_mounted_element(s.root_node(), dom));
        if let Some(node) = node_id_opt.and_then(|id| renderer.node_for_element(id)) {
            resolver.resolve_scope(scope, node);
        }
    }
}

fn first_mounted_element(vnode: &VNode, dom: &VirtualDom) -> Option<ElementId> {
    (0..vnode.template.roots.len()).find_map(|root_idx| vnode_root_element(vnode, root_idx, dom))
}

fn vnode_root_element(vnode: &VNode, root_idx: usize, dom: &VirtualDom) -> Option<ElementId> {
    let dynamic_idx = vnode.template.roots.get(root_idx)?.dynamic_id();
    match dynamic_idx {
        Some(dynamic_idx) => dynamic_node_element(vnode, dynamic_idx, dom),
        None => vnode.mounted_root(root_idx, dom),
    }
}

fn dynamic_node_element(vnode: &VNode, dynamic_idx: usize, dom: &VirtualDom) -> Option<ElementId> {
    match vnode.dynamic_nodes.get(dynamic_idx)? {
        DynamicNode::Text(_) | DynamicNode::Placeholder(_) => {
            vnode.mounted_dynamic_node(dynamic_idx, dom)
        }
        DynamicNode::Fragment(children) => children
            .iter()
            .find_map(|child| first_mounted_element(child, dom)),
        DynamicNode::Component(component) => component
            .mounted_scope(dynamic_idx, vnode, dom)
            .and_then(|scope| first_mounted_element(scope.root_node(), dom)),
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
    let mut borrowed = inner.borrow_mut();
    let RuntimeInner { dom, renderer } = &mut *borrowed;
    dom.render_immediate(renderer);
    resolve_pending(dom, renderer);
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

/// Owns the dioxus VirtualDom and ArkUI renderer for one entry point.
pub struct ArkRuntime {
    inner: Rc<RefCell<RuntimeInner>>,
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
#[derive(Default)]
struct RuntimeEventSink {
    pending: RefCell<VecDeque<PendingNativeEvent>>,
}

impl RuntimeEventSink {
    fn dispatch_pending(&self, runtime: &Rc<DioxusRuntime>) {
        let pending = self.pending.replace(VecDeque::new());
        for PendingNativeEvent {
            name,
            element,
            payload,
        } in pending
        {
            let data: Rc<dyn Any> =
                Rc::new(dioxus_elements::event::ArkEventData::with_payload(payload));
            let event = dioxus_core::Event::new(data, event_bubbles(name));
            runtime.handle_event(name, event, element);
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
            self.pending.borrow_mut().push_back(PendingNativeEvent {
                name,
                element,
                payload,
            });
            wake_ui_loop();
        })) {
            log_panic_payload("event_dispatch", payload.as_ref());
            std::process::abort();
        }
    }
}

fn event_bubbles(name: &str) -> bool {
    matches!(
        name,
        "click" | "press" | "longpress" | "long_press" | "_long_press" | "touch"
    )
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
        mut dom: VirtualDom,
        safe_area_policy: SafeAreaPolicy,
    ) -> Result<Self> {
        install_panic_hook();

        let mut renderer = ArkUIRenderer::new(slot).map_err(map_arkui_error)?;

        // Install the queueing event sink before rebuild so every listener
        // captures the same phase-isolated native event boundary.
        let sink = Rc::new(RuntimeEventSink::default());
        renderer.set_sink(sink.clone());

        // Window state is owned by the native runtime and provided before the
        // first rebuild so the framework root and every business component see
        // the same initial snapshot.
        let window_metrics = WindowMetricsHandle::new(WindowMetrics::from_app(&app, None));
        #[cfg(debug_assertions)]
        log_window_metrics(window_metrics.get());
        dom.provide_root_context(window_metrics.clone());
        dom.provide_root_context(safe_area_policy);

        // Initial mount: build the real DOM tree onto the slot.
        dom.rebuild(&mut renderer);
        resolve_pending(&dom, &renderer);

        let weak_runtime = Rc::downgrade(&dom.runtime());
        let inner = Rc::new(RefCell::new(RuntimeInner { dom, renderer }));

        // Bridge dioxus' scheduler to OpenHarmony. A pending
        // `VirtualDom::wait_for_work` poll retains this waker, including when a
        // task is later completed by the background tokio runtime.
        let waker = app.create_waker();
        let task_waker = Waker::from(Arc::new(DioxusUiWaker(waker.clone())));
        set_ui_waker(Some(Rc::new({
            let waker = waker.clone();
            move || waker.wake()
        })));

        // Run imperative UI closures and queued native events first, then let
        // dioxus render every piece of scheduler work they made ready.
        let weak_inner = Rc::downgrade(&inner);
        let loop_task_waker = task_waker.clone();
        let loop_sink = sink.clone();
        let loop_metrics = window_metrics.clone();
        let metrics_app = app.clone();
        let mut keyboard_height_px = None;
        app.run_loop(move |event| {
            let is_user_event = matches!(&event, AbilityEvent::UserEvent);
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
                keyboard_height_px = Some(*height);
            }

            if is_user_event || refresh_window_metrics {
                if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(|| {
                    if let Some(inner) = weak_inner.upgrade() {
                        if refresh_window_metrics {
                            let next = WindowMetrics::from_app(&metrics_app, keyboard_height_px);
                            if loop_metrics.update(next) {
                                inner.borrow_mut().dom.mark_all_dirty();
                            }
                        }
                        if is_user_event {
                            run_ui_loop_effects();
                            if let Some(runtime) = weak_runtime.upgrade() {
                                loop_sink.dispatch_pending(&runtime);
                            }
                        }
                        render_ready_work(&inner, &loop_task_waker);
                    }
                })) {
                    log_panic_payload("ui_loop", payload.as_ref());
                    std::process::abort();
                }
            }
        });

        // Wire the OHOS back button: forward to the global back-press handler
        // (registered by `arkit_router::use_back_handler` or any component).
        // Consumes the press when the handler returns `true`.
        app.on_back_press_intercept(move || {
            BACK_PRESS_HANDLER.with(|state| state.borrow().as_ref().map(|h| h()).unwrap_or(false))
        });

        // `rebuild` does not finish a render cycle, so run one immediate pass
        // to publish mount-time effects. Then drain exactly the scheduler work
        // that is ready and leave a pending wait armed for future async work.
        render_dom(&inner);
        render_ready_work(&inner, &task_waker);

        // ArkUI may apply control skins after initial insertion into the native
        // tree. Replaying declarative attrs on the next UI tick keeps first
        // paint consistent with later Dioxus patches without duplicating style
        // logic in components.
        let inner_for_initial_replay = inner.clone();
        queue_ui_loop(move || {
            inner_for_initial_replay
                .borrow_mut()
                .renderer
                .replay_declarative_attrs();
        });

        Ok(Self { inner })
    }

    /// Unmount the renderer root from the NodeContent slot.
    pub fn unmount(&self) -> Result<()> {
        let mut borrowed = self.inner.borrow_mut();
        borrowed.renderer.unmount().map_err(map_arkui_error)
    }
}

impl Drop for ArkRuntime {
    fn drop(&mut self) {
        set_ui_waker(None);
        set_scope_resolver(None);
        set_back_press_handler(None);
        clear_ui_loop_effects();
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
