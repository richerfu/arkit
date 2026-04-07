use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::component::{dispose_node_handle, mount_element, patch_element, MountedElement};
use crate::logging;
use crate::ohos_arkui_binding::arkui_input_binding::ArkUIErrorCode;
use crate::ohos_arkui_binding::common::error::{ArkUIError, ArkUIResult};
use crate::ohos_arkui_binding::common::node::ArkUINode;
use crate::ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use crate::ohos_arkui_binding::component::built_in_component::{Column, Stack};
use crate::ohos_arkui_binding::types::alignment::Alignment;
use crate::owner::{with_child_owner, Owner};
use crate::runtime::schedule_after_mount_effects;
use crate::view::{Element, ViewNode};

const HIT_TEST_TRANSPARENT: i32 = 2;
const PORTAL_Z_INDEX: i32 = 10_000;

thread_local! {
    static CURRENT_PORTAL_HOST: RefCell<Option<PortalHostHandle>> = RefCell::new(None);
}

#[derive(Clone)]
pub(crate) struct PortalHostHandle {
    inner: Rc<PortalHostInner>,
}

struct PortalHostInner {
    container: RefCell<ArkUINode>,
    entries: RefCell<Vec<Rc<RefCell<PortalEntry>>>>,
    next_id: Cell<usize>,
}

struct PortalEntry {
    id: usize,
    handle: Rc<RefCell<ArkUINode>>,
    mounted: Option<MountedElement>,
}

impl PortalHostHandle {
    fn new(container: ArkUINode) -> Self {
        Self {
            inner: Rc::new(PortalHostInner {
                container: RefCell::new(container),
                entries: RefCell::new(Vec::new()),
                next_id: Cell::new(1),
            }),
        }
    }

    pub(crate) fn next_id(&self) -> usize {
        let id = self.inner.next_id.get();
        self.inner.next_id.set(id + 1);
        id
    }

    pub(crate) fn update(&self, id: usize, next: Option<Element>) -> ArkUIResult<()> {
        match next {
            Some(next) => self.upsert(id, next),
            None => self.remove(id),
        }
    }

    pub(crate) fn remove(&self, id: usize) -> ArkUIResult<()> {
        let Some(index) = self.entry_index(id) else {
            return Ok(());
        };
        self.remove_at(index)
    }

    pub(crate) fn clear(&self) -> ArkUIResult<()> {
        let ids = self
            .inner
            .entries
            .borrow()
            .iter()
            .map(|entry| entry.borrow().id)
            .collect::<Vec<_>>();

        for id in ids {
            self.remove(id)?;
        }

        Ok(())
    }

    fn upsert(&self, id: usize, next: Element) -> ArkUIResult<()> {
        if let Some(entry) = self.entry(id) {
            let patchable = {
                let entry = entry.borrow();
                let mounted = entry
                    .mounted
                    .as_ref()
                    .expect("portal entry should be mounted while patching");
                mounted.kind == next.kind() && mounted.key.as_deref() == next.key()
            };

            if patchable {
                let handle = {
                    let entry = entry.borrow();
                    entry.handle.clone()
                };
                let mut entry = entry.borrow_mut();
                let mounted = entry
                    .mounted
                    .as_mut()
                    .expect("portal entry should be mounted while patching");
                let mut node = handle.borrow_mut();
                return patch_element(next, &mut node, mounted);
            }

            let (next_node, next_mounted) = mount_element(next)?;
            let Some(index) = self.entry_index(id) else {
                let mut cleanup_node = next_node.clone();
                let _ = cleanup_node.dispose();
                next_mounted.cleanup_recursive();
                return Err(ArkUIError::new(
                    ArkUIErrorCode::ParamInvalid,
                    "portal entry disappeared during replace",
                ));
            };

            self.remove_at(index)?;
            self.insert_at(index, id, next_node, next_mounted)
        } else {
            let (next_node, next_mounted) = mount_element(next)?;
            let index = self.inner.entries.borrow().len();
            self.insert_at(index, id, next_node, next_mounted)
        }
    }

    fn insert_at(
        &self,
        index: usize,
        id: usize,
        node: ArkUINode,
        mounted: MountedElement,
    ) -> ArkUIResult<()> {
        let mut cleanup_node = node.clone();
        {
            let mut container = self.inner.container.borrow_mut();
            let attach_result = if index >= container.children().len() {
                container.add_child(node)
            } else {
                container.insert_child(node, index)
            };
            if let Err(error) = attach_result {
                let _ = cleanup_node.dispose();
                mounted.cleanup_recursive();
                return Err(error);
            }
        }

        let handle = {
            let container = self.inner.container.borrow();
            container.children().get(index).cloned().ok_or_else(|| {
                ArkUIError::new(
                    ArkUIErrorCode::ParamInvalid,
                    "portal child handle missing after attach",
                )
            })?
        };

        self.inner.entries.borrow_mut().insert(
            index,
            Rc::new(RefCell::new(PortalEntry {
                id,
                handle,
                mounted: Some(mounted),
            })),
        );
        Ok(())
    }

    fn remove_at(&self, index: usize) -> ArkUIResult<()> {
        let entry = self.inner.entries.borrow_mut().remove(index);
        let removed = {
            let mut container = self.inner.container.borrow_mut();
            container.remove_child(index)?
        };

        if let Some(removed) = removed {
            dispose_node_handle(removed)?;
        }

        let mounted = { entry.borrow_mut().mounted.take() };
        if let Some(mounted) = mounted {
            mounted.cleanup_recursive();
        }
        Ok(())
    }

    fn entry(&self, id: usize) -> Option<Rc<RefCell<PortalEntry>>> {
        self.inner
            .entries
            .borrow()
            .iter()
            .find(|entry| entry.borrow().id == id)
            .cloned()
    }

    fn entry_index(&self, id: usize) -> Option<usize> {
        self.inner
            .entries
            .borrow()
            .iter()
            .position(|entry| entry.borrow().id == id)
    }
}

pub(crate) fn build_portal_host() -> ArkUIResult<(PortalHostHandle, ArkUINode)> {
    let mut container = Stack::new()?;
    container.percent_width(1.0)?;
    container.percent_height(1.0)?;
    container.set_alignment(i32::from(Alignment::TopStart))?;
    container.set_clip(false)?;
    container.set_hit_test_behavior(HIT_TEST_TRANSPARENT)?;
    container.z_index(PORTAL_Z_INDEX)?;
    let container_node = container.borrow_mut().clone();
    let host = PortalHostHandle::new(container_node.clone());
    Ok((host, container_node))
}

pub(crate) fn current_portal_host() -> ArkUIResult<PortalHostHandle> {
    CURRENT_PORTAL_HOST.with(|state| {
        state.borrow().clone().ok_or_else(|| {
            ArkUIError::new(
                ArkUIErrorCode::ParamInvalid,
                "portal host is not installed for the current runtime",
            )
        })
    })
}

pub(crate) fn set_current_portal_host(host: Option<PortalHostHandle>) {
    CURRENT_PORTAL_HOST.with(|state| {
        state.replace(host);
    });
}

pub(crate) fn with_current_portal_host<R>(host: PortalHostHandle, f: impl FnOnce() -> R) -> R {
    CURRENT_PORTAL_HOST.with(|state| {
        let previous = state.replace(Some(host));
        let result = f();
        state.replace(previous);
        result
    })
}

/// State for a portal scope.
/// The render function mounts a child owner once and later updates are driven by
/// explicit runtime rerenders that patch the subtree.
struct PortalScopeState {
    host: PortalHostHandle,
    entry_id: usize,
    child_owner: RefCell<Option<Rc<Owner>>>,
}

impl PortalScopeState {
    fn cleanup(&self) {
        let owner = self.child_owner.borrow_mut().take();
        if let Some(owner) = owner {
            owner.dispose();
        }
        if let Err(error) = self.host.remove(self.entry_id) {
            logging::error(format!(
                "portal error: failed to remove portal entry: {error}"
            ));
        }
    }
}

pub struct PortalScopeElement {
    render: Option<Box<dyn FnOnce() -> Option<Element>>>,
    key: Option<String>,
}

pub fn portal_scope(render: impl FnOnce() -> Option<Element> + 'static) -> Element {
    PortalScopeElement {
        render: Some(Box::new(render)),
        key: None,
    }
    .into()
}

impl ViewNode for PortalScopeElement {
    fn kind(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let Self { render, key } = *self;
        let host = current_portal_host()?;
        let entry_id = host.next_id();

        let render_fn = render.expect("portal scope render should exist during mount");

        // Render once inside a child owner scope
        let (portal_element, child_owner) = with_child_owner(|| render_fn());

        if let Some(element) = portal_element {
            host.update(entry_id, Some(element))?;
        }

        let state = Rc::new(PortalScopeState {
            host,
            entry_id,
            child_owner: RefCell::new(Some(child_owner)),
        });

        schedule_after_mount_effects();

        let mut placeholder = Column::new()?;
        placeholder.width(0.0)?;
        placeholder.height(0.0)?;
        placeholder.set_visibility(2_i32)?;
        let placeholder_node = placeholder.borrow_mut().clone();

        let cleanup_state = state.clone();
        let mut mounted = MountedElement::new(
            TypeId::of::<Self>(),
            std::any::type_name::<Self>(),
            key,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        mounted.set_state(Box::new(state));
        Ok((placeholder_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        _node: &mut ArkUINode,
        _mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        // Portal content is patched through normal runtime rerenders after the
        // initial mount, so this placeholder node has no direct patch work.
        Ok(())
    }
}
