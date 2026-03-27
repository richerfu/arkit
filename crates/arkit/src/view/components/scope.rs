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
    render: Rc<dyn Fn() -> Element>,
    key: Option<String>,
}

pub fn scope(render: impl Fn() -> Element + 'static) -> Element {
    ScopeElement {
        render: Rc::new(render),
        key: None,
    }
    .into()
}

pub fn keyed_scope(key: impl Into<String>, render: impl Fn() -> Element + 'static) -> Element {
    ScopeElement {
        render: Rc::new(render),
        key: Some(key.into()),
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

        // Render once inside a child owner scope
        let (child_element, child_owner) = with_child_owner(|| (render)());
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
