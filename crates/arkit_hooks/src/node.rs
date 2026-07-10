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
use std::collections::HashMap;
use std::rc::Rc;

use arkit_prelude::*;
use ohos_arkui_binding::common::node::ArkUINode;

use crate::layout::LayoutFrame;

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
    pending: HashMap<ScopeId, Signal<Option<HostNode>>>,
    /// The active overlay content, rendered as a full-screen stack subtree at
    /// the app root by [`use_ark_host_provider`]. `None` = no overlay open.
    /// Driven by [`crate::use_overlay`].
    overlay_content: Option<Signal<Option<Element>>>,
    /// Measured frame of the app-level overlay root. Trigger layout frames are
    /// window-relative, while overlay children are laid out inside this root, so
    /// floating placement must subtract this origin before converting to vp.
    overlay_frame: Option<Signal<LayoutFrame>>,
}

impl Default for ArkHost {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(ArkHostInner {
                pending: HashMap::new(),
                overlay_content: None,
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
    pub(crate) fn register_scope(&self, scope: ScopeId, slot: Signal<Option<HostNode>>) {
        self.inner.borrow_mut().pending.entry(scope).or_insert(slot);
    }

    /// Snapshot of all scopes awaiting node resolution (called by the runtime
    /// after each render).
    pub fn pending_scopes(&self) -> Vec<ScopeId> {
        self.inner.borrow().pending.keys().copied().collect()
    }

    /// Resolve a scope's backing node. Called by the renderer/runtime
    /// integration after the scope's root element is mounted. Writes the node
    /// into the signal slot, notifying any reactive readers (e.g. the layout
    /// observer hooks).
    pub fn resolve_scope(&self, scope: ScopeId, node: HostNode) {
        // Signal is Copy; clone it out of the borrow so we can take `&mut`.
        let slot = self.inner.borrow().pending.get(&scope).copied();
        if let Some(mut slot) = slot {
            slot.set(Some(node));
        }
    }

    /// Clear a scope's slot (called automatically on unmount via `use_drop`).
    pub(crate) fn revoke_scope(&self, scope: ScopeId) {
        let slot = self.inner.borrow_mut().pending.remove(&scope);
        if let Some(mut slot) = slot {
            slot.set(None);
        }
    }

    /// Allocate (if needed) and return the overlay-content signal. The signal
    /// holds the currently-open overlay's `Element`; `use_ark_host_provider`
    /// renders it as a full-screen stack subtree at the app root.
    pub(crate) fn overlay_content(&self) -> Signal<Option<Element>> {
        let mut inner = self.inner.borrow_mut();
        if let Some(sig) = inner.overlay_content {
            return sig;
        }
        let sig = Signal::new(None);
        inner.overlay_content = Some(sig);
        sig
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
    let host = use_context_provider(ArkHost::new);
    // Register the host with the runtime so post-render passes can resolve
    // `use_ark_node` lookups and install the overlay portal root.
    arkit_runtime::set_scope_resolver(Some(std::rc::Rc::new(host.clone())));
    use_drop(|| arkit_runtime::set_scope_resolver(None));
    host
}

/// Render the active overlay content (driven by [`use_overlay`]) as a
/// full-screen stack subtree on top of the app. Must be rendered somewhere
/// inside a subtree that called [`use_ark_host_provider`].
#[allow(non_snake_case)]
pub fn OverlayRoot() -> Element {
    let host = use_context::<ArkHost>();
    let frame_host = host.clone();
    crate::layout::use_layout_frame(move |frame| {
        frame_host.set_overlay_frame(frame);
    });
    let content = host.overlay_content();
    let current = content();
    match current {
        Some(node) => rsx! {
            stack {
                percent_width: 1.0,
                percent_height: 1.0,
                alignment: 0,
                hit_test_behavior: 2_i32,
                {node}
            }
        },
        None => rsx! {
            stack {
                percent_width: 1.0,
                percent_height: 1.0,
                alignment: 0,
                hit_test_behavior: 2_i32,
            }
        },
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
        (self.signal)()
    }

    /// Read the node without subscribing.
    pub fn peek(&self) -> Option<HostNode> {
        self.signal.peek().clone()
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
    let signal = use_hook(|| {
        let slot: Signal<Option<HostNode>> = Signal::new(None);
        host.register_scope(scope, slot);
        slot
    });

    // Re-register on every render in case the host was reset (cheap no-op when
    // the slot is unchanged). `use_hook` only runs the initializer once, so we
    // ensure the mapping is current here.
    host.register_scope(scope, signal);

    // Revoke on unmount.
    let host_for_drop = host;
    let scope_for_drop = scope;
    dioxus_core::use_drop(move || {
        host_for_drop.revoke_scope(scope_for_drop);
    });

    ArkNodeRef { signal }
}
