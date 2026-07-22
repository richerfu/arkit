//! `use_ark_node` and the `ArkHost` context.
//!
//! ## Node lookup
//!
//! A dioxus hook cannot discover its own `ElementId` — ids are assigned deep
//! in the dioxus diff layer and are not exposed to component bodies in dioxus
//! 0.7. The renderer (`arkit_arkui::ArkUIRenderer`) tracks `ElementId → node`,
//! while the `ElementId ↔ ScopeId` bridge belongs to the `VirtualDom` in
//! `arkit_runtime`. `ArkHost` connects those two owners through a resolver
//! keyed by `ScopeId`:
//!
//! 1. The app root calls [`use_ark_host_provider`] once (provides the
//!    [`ArkHost`] context to the whole tree).
//! 2. [`use_ark_node`] allocates a `Signal<Option<HostNode>>` slot via
//!    `use_hook`, keyed by the current [`dioxus_core::ScopeId`], and registers
//!    it in the host.
//! 3. After each render, `arkit_runtime` resolves pending scopes through
//!    [`ArkUIRenderer::node_for_element`](arkit_arkui::ArkUIRenderer::node_for_element)
//!    and writes the mounted node into the registered signal.
//!
//! Separately, [`ArkHost`] holds the overlay-content signal
//! ([`ArkHost::overlay_content`]) driven by [`crate::use_overlay`]; the app
//! root renders [`OverlayRoot`] to mount that content as a full-screen stack.

use std::cell::RefCell;
use std::rc::Rc;

use arkit_prelude::*;
use ohos_arkui_binding::common::node::ArkUINode;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::layout::LayoutFrame;
use crate::overlay::OverlayLayer;

// ArkUI hit-test modes for documentation and typed API. RSX must pass CSS
// keywords (`"none"`, `"default"`, …) — raw integers are rejected by the encoder.
//
// `None`: the portal root itself is never a touch target; interactive
// descendants still participate. `Transparent` is not equivalent: it still
// makes the full-screen node part of the hit-test result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum HitTestMode {
    Default = 0,
    Block = 1,
    Transparent = 2,
    None = 3,
}

impl HitTestMode {
    /// CSS keyword accepted by the RSX `hit_test_behavior` attribute.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Block => "block",
            Self::Transparent => "transparent",
            Self::None => "none",
        }
    }
}

#[derive(Clone)]
struct OverlayEntry {
    token: u64,
    layer: OverlayLayer,
    element: Element,
}

/// Shared handle to the native ArkUI node backing a dioxus element. This is the
/// same `Rc` mounted in the ArkUI tree (the event-dispatch user-data target),
/// so event registration on it reaches ArkUI's dispatcher.
pub type HostNode = Rc<RefCell<ArkUINode>>;

/// Shared host state provided at the app root via [`use_ark_host_provider`].
///
/// Cloning is cheap (it is an `Rc` handle to the inner registry).
#[derive(Clone)]
pub struct ArkHost {
    inner: Rc<RefCell<ArkHostInner>>,
}

struct ArkHostInner {
    /// ScopeId → the signal slot allocated by `use_ark_node`. The integration
    /// writes the resolved node into the signal via `resolve_scope`.
    bindings: FxHashMap<ScopeId, Signal<Option<HostNode>>>,
    /// Scopes whose component rendered since the last successful resolution.
    /// Keeping this separate from `bindings` avoids traversing every hooked
    /// scope after unrelated renders elsewhere in the tree.
    dirty_scopes: FxHashSet<ScopeId>,
    /// All active overlay entries, rendered together as keyed full-screen
    /// pass-through subtrees at the app root. Modal/floating/transient layers
    /// must coexist: publishing a toast must never hide an active dialog.
    overlay_content: Option<Signal<Vec<OverlayEntry>>>,
    /// Tokenized overlay registry. A stale component may update or remove only
    /// its own entry. Ordering is stable across content refreshes.
    overlay_entries: Vec<OverlayEntry>,
    next_overlay_token: u64,
    /// Measured frame of the app-level overlay root. Trigger layout frames are
    /// window-relative, while overlay children are laid out inside this root, so
    /// floating placement must subtract this origin before converting to vp.
    overlay_frame: Option<Signal<LayoutFrame>>,
}

impl Default for ArkHost {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(ArkHostInner {
                bindings: FxHashMap::default(),
                dirty_scopes: FxHashSet::default(),
                overlay_content: None,
                overlay_entries: Vec::new(),
                next_overlay_token: 0,
                overlay_frame: None,
            })),
        }
    }
}

impl ArkHost {
    /// Create an empty host.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a signal slot for a scope (called by `use_ark_node`).
    ///
    /// **Idempotent**: if a slot already exists for the scope, the existing slot
    /// is returned and the new one is discarded. This makes `use_ark_node` safe
    /// to call multiple times in the same component scope (e.g. a component
    /// calling it directly + a layout hook calling it internally) — all callers
    /// share one resolver slot per scope, so none overwrites another.
    pub(crate) fn register_scope(
        &self,
        scope: ScopeId,
        slot: Signal<Option<HostNode>>,
    ) -> Signal<Option<HostNode>> {
        let mut inner = self.inner.borrow_mut();
        let registered = *inner.bindings.entry(scope).or_insert(slot);
        inner.dirty_scopes.insert(scope);
        registered
    }

    /// Snapshot of all scopes awaiting node resolution (called by the runtime
    /// after each render).
    pub fn pending_scopes(&self) -> Vec<ScopeId> {
        self.inner.borrow().dirty_scopes.iter().copied().collect()
    }

    /// Resolve a scope's backing node. Called by the renderer/runtime
    /// integration after the scope's root element is mounted. Writes the node
    /// into the signal slot, notifying any reactive readers (e.g. the layout
    /// observer hooks).
    pub fn resolve_scope(&self, scope: ScopeId, node: HostNode) {
        // Signal is Copy; clone it out of the borrow so we can take `&mut`.
        let slot = self.inner.borrow().bindings.get(&scope).copied();
        if let Some(mut slot) = slot {
            if let Ok(mut value) = slot.try_write() {
                let unchanged = value
                    .as_ref()
                    .is_some_and(|current| Rc::ptr_eq(current, &node));
                if !unchanged {
                    *value = Some(node);
                }
                self.inner.borrow_mut().dirty_scopes.remove(&scope);
            }
        }
    }

    /// Clear a scope's slot (called automatically on unmount via `use_drop`).
    pub(crate) fn revoke_scope(&self, scope: ScopeId) {
        let slot = {
            let mut inner = self.inner.borrow_mut();
            inner.dirty_scopes.remove(&scope);
            inner.bindings.remove(&scope)
        };
        if let Some(mut slot) = slot {
            if let Ok(mut value) = slot.try_write() {
                *value = None;
            }
        }
    }

    /// Allocate (if needed) and return the overlay-content signal. The signal
    /// holds every currently-open entry; [`OverlayRoot`] renders them in stable
    /// layer/token order.
    fn overlay_content(&self) -> Signal<Vec<OverlayEntry>> {
        let mut inner = self.inner.borrow_mut();
        if let Some(sig) = inner.overlay_content {
            return sig;
        }
        let sig = Signal::new(Vec::new());
        inner.overlay_content = Some(sig);
        sig
    }

    pub(crate) fn allocate_overlay_token(&self) -> u64 {
        let mut inner = self.inner.borrow_mut();
        inner.next_overlay_token = inner
            .next_overlay_token
            .checked_add(1)
            .expect("arkit_hooks: overlay token space exhausted");
        inner.next_overlay_token
    }

    pub(crate) fn set_overlay(&self, token: u64, layer: OverlayLayer, element: Element) {
        let mut signal = self.overlay_content();
        let entries = {
            let mut inner = self.inner.borrow_mut();
            if let Some(entry) = inner
                .overlay_entries
                .iter_mut()
                .find(|entry| entry.token == token)
            {
                entry.layer = layer;
                entry.element = element;
            } else {
                inner.overlay_entries.push(OverlayEntry {
                    token,
                    layer,
                    element,
                });
            }
            inner
                .overlay_entries
                .sort_by_key(|entry| (entry.layer, entry.token));
            inner.overlay_entries.clone()
        };
        signal.set(entries);
    }

    pub(crate) fn dismiss_overlay(&self, token: u64) -> bool {
        let mut signal = self.overlay_content();
        let (removed, entries) = {
            let mut inner = self.inner.borrow_mut();
            let before = inner.overlay_entries.len();
            inner.overlay_entries.retain(|entry| entry.token != token);
            let removed = before != inner.overlay_entries.len();
            (removed, inner.overlay_entries.clone())
        };
        if removed {
            signal.set(entries);
        }
        removed
    }

    pub(crate) fn overlay_frame(&self) -> Signal<LayoutFrame> {
        let mut inner = self.inner.borrow_mut();
        if let Some(sig) = inner.overlay_frame {
            return sig;
        }
        let sig = Signal::new(LayoutFrame::default());
        inner.overlay_frame = Some(sig);
        sig
    }

    pub(crate) fn set_overlay_frame(&self, frame: LayoutFrame) {
        let mut sig = self.overlay_frame();
        if *sig.peek() != frame {
            sig.set(frame);
        }
    }

    pub fn overlay_frame_value(&self) -> LayoutFrame {
        *self.overlay_frame().peek()
    }
}

/// `ArkHost` implements the runtime's resolver trait so the VirtualDom owner
/// (`arkit_runtime`) can resolve pending `use_ark_node` lookups without a
/// circular dependency on `arkit_hooks`.
impl arkit_runtime::ScopeNodeResolver for ArkHost {
    fn pending_scopes(&self) -> Vec<ScopeId> {
        ArkHost::pending_scopes(self)
    }

    fn resolve_scope(&self, scope: ScopeId, node: HostNode) {
        ArkHost::resolve_scope(self, scope, node);
    }
}

/// Provide the [`ArkHost`] context to the component subtree and register it as
/// the runtime's scope-node resolver.
///
/// Call once near the app root, then render [`OverlayRoot`] (which mounts the
/// active overlay content as a full-screen stack on top of the app):
///
/// ```ignore
/// fn app() -> Element {
///     let _host = use_ark_host_provider();
///     rsx! {
///         {app_content()}
///         OverlayRoot {}
///     }
/// }
/// ```
#[track_caller]
pub fn use_ark_host_provider() -> ArkHost {
    let _application_lifecycle = crate::lifecycle::use_application_lifecycle_provider();
    let _window_metrics = crate::safe_area::use_window_metrics_provider();
    let host = use_context_provider(ArkHost::new);
    // Register the host with the runtime so post-render passes can resolve
    // `use_ark_node` lookups and install the overlay portal root.
    let resolver_host = host.clone();
    let _resolver_registration = use_hook(|| {
        Rc::new(arkit_runtime::register_scope_resolver(Rc::new(
            resolver_host,
        )))
    });
    host
}

/// Render the active overlay content (driven by [`use_overlay`]) as a
/// full-screen stack subtree on top of the app. Must be rendered somewhere
/// inside a subtree that called [`use_ark_host_provider`].
#[allow(non_snake_case)]
pub fn OverlayRoot() -> Element {
    let host = use_context::<ArkHost>();
    let frame_host = host.clone();
    let window_metrics = dioxus_core::try_consume_context::<arkit_runtime::WindowMetricsHandle>();
    crate::layout::use_layout_frame(move |frame| {
        frame_host.set_overlay_frame(frame);
        if let Some(metrics) = window_metrics.as_ref() {
            metrics.report_content_rect(arkit_runtime::PhysicalRect {
                top: frame.y.round() as i32,
                left: frame.x.round() as i32,
                width: frame.width.round() as i32,
                height: frame.height.round() as i32,
            });
        }
    });
    let content = host.overlay_content();
    let entries = content();
    rsx! {
        stack {
            width: "100%",
            height: "100%",
            alignment: "top-start",
            // Must be a CSS keyword — raw ArkUI ints are not encoded.
            hit_test_behavior: "none",
            for entry in entries {
                stack {
                    key: "{entry.token}",
                    width: "100%",
                    height: "100%",
                    alignment: "top-start",
                    z_index: entry.layer.z_index(),
                    hit_test_behavior: "none",
                    {entry.element}
                }
            }
        }
    }
}

/// Obtain the shared [`ArkHost`]. Panics if no ancestor called
/// [`use_ark_host_provider`].
#[track_caller]
pub fn use_ark_host() -> ArkHost {
    use_context::<ArkHost>()
}

/// A read-only handle to the native ArkUI node backing the current dioxus
/// element.
///
/// The node is `None` until the renderer resolves the scope (see the module
/// docs). Read it reactively with [`ArkNodeRef::get`] (which subscribes the
/// current scope) or peek without subscribing with [`ArkNodeRef::peek`].
#[derive(Clone, Copy)]
pub struct ArkNodeRef {
    signal: Signal<Option<HostNode>>,
}

impl ArkNodeRef {
    /// Read the node, subscribing the current scope to changes. Returns
    /// `None` until the renderer has resolved the scope.
    pub fn get(&self) -> Option<HostNode> {
        self.signal.try_read().ok().and_then(|value| value.clone())
    }

    /// Read the node without subscribing.
    pub fn peek(&self) -> Option<HostNode> {
        self.signal.try_peek().ok().and_then(|value| value.clone())
    }

    /// Access the underlying signal (crate-internal, for the layout hooks).
    pub(crate) fn signal(self) -> Signal<Option<HostNode>> {
        self.signal
    }
}

/// Allocate a slot for the native ArkUI node backing the current dioxus
/// element and return a read-only handle.
///
/// The slot starts as `None` and is filled by the renderer integration via
/// [`ArkHost::resolve_scope`]. Use [`ArkNodeRef::get`] to read it reactively.
///
/// ```ignore
/// fn my_component() -> Element {
///     let node = use_ark_node();
///     use_effect(move || {
///         if let Some(n) = node.get() {
///             // ... register native callbacks on n
///         }
///     });
///     rsx! { ... }
/// }
/// ```
#[track_caller]
pub fn use_ark_node() -> ArkNodeRef {
    let host = use_ark_host();
    let scope = current_scope_id();
    let signal = use_hook(|| host.register_scope(scope, Signal::new(None)));

    // Re-register on every render in case the host was reset (cheap no-op when
    // the slot is unchanged). `use_hook` only runs the initializer once, so we
    // ensure the mapping is current here.
    let _ = host.register_scope(scope, signal);

    // Revoke on unmount.
    let host_for_drop = host;
    let scope_for_drop = scope;
    dioxus_core::use_drop(move || {
        host_for_drop.revoke_scope(scope_for_drop);
    });

    ArkNodeRef { signal }
}
