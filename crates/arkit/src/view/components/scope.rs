use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;

use crate::component::{mount_element, MountedElement};
use crate::logging;
use crate::ohos_arkui_binding::common::error::ArkUIResult;
use crate::ohos_arkui_binding::common::node::ArkUINode;
use crate::ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use crate::ohos_arkui_binding::component::built_in_component::Column;
use crate::owner::{with_child_owner, Owner};
use crate::runtime::schedule_after_mount_effects;

use super::super::element::{Element, ViewNode};

/// Internal state for a scoped element in the new reactive model.
/// The render function runs once; signals drive all updates via effects.
#[allow(dead_code)]
struct ScopedState {
    container: RefCell<Column>,
    mounted_child: RefCell<Option<MountedElement>>,
    child_owner: RefCell<Option<Rc<Owner>>>,
}

impl ScopedState {
    fn cleanup(&self) {
        if let Some(meta) = self.mounted_child.borrow_mut().take() {
            meta.cleanup_recursive();
        }
        if let Some(owner) = self.child_owner.borrow_mut().take() {
            owner.dispose();
        }
    }
}

pub struct ScopeElement {
    /// Stored as `FnOnce` so that component parameters can be moved into the
    /// closure without requiring `Clone` or `Copy`. Wrapped in `RefCell<Option>`
    /// because `FnOnce` can only be called once — we take it during mount.
    render: Rc<RefCell<Option<Box<dyn FnOnce() -> Element>>>>,
    key: Option<String>,
}

pub fn scope(render: impl FnOnce() -> Element + 'static) -> Element {
    ScopeElement {
        render: Rc::new(RefCell::new(Some(Box::new(render)))),
        key: None,
    }
    .into()
}

pub fn keyed_scope(key: impl Into<String>, render: impl FnOnce() -> Element + 'static) -> Element {
    ScopeElement {
        render: Rc::new(RefCell::new(Some(Box::new(render)))),
        key: Some(key.into()),
    }
    .into()
}

/// Create a scope from a pre-rendered element and its child owner.
///
/// This is the guard-based companion to [`scope`]: the caller runs the body
/// directly (not inside a closure) between `enter_scope()` and `guard.exit()`,
/// then passes the result here. Used by `#[component]` so that the function
/// body stays unwrapped for LSP analysis.
#[doc(hidden)]
pub fn scope_owned(child_owner: Rc<Owner>, element: Element) -> Element {
    OwnedScopeElement {
        element: Some(element),
        child_owner,
        key: None,
    }
    .into()
}

impl ScopeElement {
    pub fn key(mut self, value: impl Into<String>) -> Self {
        self.key = Some(value.into());
        self
    }
}

impl ViewNode for ScopeElement {
    fn kind(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let Self { render, key } = *self;

        let mut container = Column::new()?;
        let container_node = container.borrow_mut().clone();

        // Take the render function — it runs exactly once.
        let render_fn = render
            .borrow_mut()
            .take()
            .expect("scope render function called more than once");

        // Render once inside a child owner scope
        let (child_element, child_owner) = with_child_owner(|| render_fn());
        let (child_node, child_meta) = mount_element(child_element).map_err(|error| {
            logging::error(format!(
                "scope error: failed to mount scoped root child: {error}"
            ));
            error
        })?;

        container.add_child(child_node).map_err(|error| {
            logging::error(format!(
                "scope error: failed to attach scoped root child: {error}"
            ));
            error
        })?;

        let state = Rc::new(ScopedState {
            container: RefCell::new(container),
            mounted_child: RefCell::new(Some(child_meta)),
            child_owner: RefCell::new(Some(child_owner)),
        });

        schedule_after_mount_effects();

        let cleanup_state = state.clone();
        let mut mounted = MountedElement::new(
            TypeId::of::<Self>(),
            std::any::type_name::<Self>(),
            key,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        mounted.set_state(Box::new(state));
        Ok((container_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        _node: &mut ArkUINode,
        _mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        // In the fine-grained reactive model, the scope renders once and
        // signals drive updates via effects. Patch is a no-op.
        Ok(())
    }
}

// ── OwnedScopeElement (guard-based, for #[component] macro) ────────────────────

/// A scope element where the body has already been executed via `enter_scope`
/// guard. On mount it wraps the pre-rendered element in a Column container.
struct OwnedScopeElement {
    element: Option<Element>,
    child_owner: Rc<Owner>,
    key: Option<String>,
}

impl ViewNode for OwnedScopeElement {
    fn kind(&self) -> TypeId {
        TypeId::of::<ScopeElement>()
    }

    fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    fn mount(mut self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let key = self.key.take();
        let child_element = self
            .element
            .take()
            .expect("OwnedScopeElement mount called twice");
        let child_owner = self.child_owner.clone();

        let mut container = Column::new()?;
        let container_node = container.borrow_mut().clone();

        let (child_node, child_meta) = mount_element(child_element).map_err(|error| {
            logging::error(format!(
                "scope error: failed to mount scoped root child: {error}"
            ));
            error
        })?;

        container.add_child(child_node).map_err(|error| {
            logging::error(format!(
                "scope error: failed to attach scoped root child: {error}"
            ));
            error
        })?;

        let state = Rc::new(ScopedState {
            container: RefCell::new(container),
            mounted_child: RefCell::new(Some(child_meta)),
            child_owner: RefCell::new(Some(child_owner)),
        });

        schedule_after_mount_effects();

        let cleanup_state = state.clone();
        let mut mounted = MountedElement::new(
            TypeId::of::<ScopeElement>(),
            std::any::type_name::<ScopeElement>(),
            key,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        mounted.set_state(Box::new(state));
        Ok((container_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        _node: &mut ArkUINode,
        _mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        Ok(())
    }
}
