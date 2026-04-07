use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::component::{dispose_node_handle, mount_element, MountedElement};
use crate::ohos_arkui_binding::common::error::ArkUIResult;
use crate::ohos_arkui_binding::common::node::ArkUINode;
use crate::ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use crate::ohos_arkui_binding::component::built_in_component::Column;
use crate::owner::{with_child_owner, with_owner, Owner};
use crate::runtime::schedule_after_mount_effects;
use crate::view::element::{Element, ViewNode};

// ── Show ────────────────────────────────────────────────────────────────────

/// Conditionally render an element. When the condition becomes true the child
/// is mounted; when it becomes false the child is unmounted and cleaned up.
///
/// ```ignore
/// show(
///     move || visible.get(),
///     move || text("Hello!").into(),
/// )
/// ```
pub fn show(
    condition: impl Fn() -> bool + 'static,
    child: impl Fn() -> Element + 'static,
) -> Element {
    ShowElement {
        condition: Rc::new(condition),
        child: Rc::new(child),
    }
    .into()
}

struct ShowElement {
    condition: Rc<dyn Fn() -> bool>,
    child: Rc<dyn Fn() -> Element>,
}

struct ShowState {
    container: RefCell<Column>,
    mounted_child: RefCell<Option<MountedElement>>,
    child_owner: RefCell<Option<Rc<Owner>>>,
    condition: RefCell<Rc<dyn Fn() -> bool>>,
    child: RefCell<Rc<dyn Fn() -> Element>>,
    visible: Cell<bool>,
}

impl ShowState {
    fn mount_child(&self, child_fn: &dyn Fn() -> Element) -> ArkUIResult<()> {
        let (element, child_owner) = with_child_owner(|| child_fn());
        let (child_node, child_meta) = with_owner(child_owner.clone(), || mount_element(element))?;
        self.container.borrow_mut().add_child(child_node)?;
        self.mounted_child.replace(Some(child_meta));
        self.child_owner.replace(Some(child_owner));
        schedule_after_mount_effects();
        Ok(())
    }

    fn unmount_child(&self) -> ArkUIResult<()> {
        if let Some(removed) = self.container.borrow_mut().remove_child(0)? {
            dispose_node_handle(removed)?;
        }
        let meta = self.mounted_child.borrow_mut().take();
        if let Some(meta) = meta {
            meta.cleanup_recursive();
        }
        let owner = self.child_owner.borrow_mut().take();
        if let Some(owner) = owner {
            owner.dispose();
        }
        Ok(())
    }

    fn cleanup(&self) {
        let meta = self.mounted_child.borrow_mut().take();
        if let Some(meta) = meta {
            meta.cleanup_recursive();
        }
        let owner = self.child_owner.borrow_mut().take();
        if let Some(owner) = owner {
            owner.dispose();
        }
    }

    fn has_child(&self) -> bool {
        self.mounted_child.borrow().is_some()
    }

    fn replace_child(&self, child_fn: &dyn Fn() -> Element) -> ArkUIResult<()> {
        if self.has_child() {
            self.unmount_child()?;
        }
        self.mount_child(child_fn)
    }

    fn sync_current(&self) -> ArkUIResult<()> {
        let next = {
            let condition = self.condition.borrow().clone();
            condition()
        };
        let child = self.child.borrow().clone();
        match (next, self.has_child()) {
            (true, true) => self.replace_child(child.as_ref())?,
            (true, false) => self.mount_child(child.as_ref())?,
            (false, true) => self.unmount_child()?,
            (false, false) => {}
        }
        self.visible.set(next);
        Ok(())
    }
}

impl ViewNode for ShowElement {
    fn kind(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<&str> {
        None
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let Self { condition, child } = *self;
        let mut container = Column::new()?;
        let container_node = container.borrow_mut().clone();

        let state = Rc::new(ShowState {
            container: RefCell::new(container),
            mounted_child: RefCell::new(None),
            child_owner: RefCell::new(None),
            condition: RefCell::new(condition.clone()),
            child: RefCell::new(child.clone()),
            visible: Cell::new(false),
        });

        // Initial render
        state.sync_current()?;

        let cleanup_state = state.clone();
        let mut mounted = MountedElement::new(
            TypeId::of::<ShowElement>(),
            std::any::type_name::<ShowElement>(),
            None,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        mounted.set_state(Box::new(state));

        Ok((container_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        _node: &mut ArkUINode,
        mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        let Self { condition, child } = *self;
        let state = mounted
            .state_mut::<Rc<ShowState>>()
            .expect("show patch missing ShowState");
        state.condition.replace(condition);
        state.child.replace(child);
        state.sync_current()
    }
}

// ── For ─────────────────────────────────────────────────────────────────────

/// Render a keyed list of items. On patch, items are added/removed/reordered
/// with minimal DOM mutations based on the current `each()` result.
///
/// ```ignore
/// for_each(
///     move || items.get(),
///     |item| text(item.name.clone()).into(),
///     |item| item.id.to_string(),
/// )
/// ```
pub fn for_each<T, K>(
    each: impl Fn() -> Vec<T> + 'static,
    child: impl Fn(&T) -> Element + 'static,
    key_fn: impl Fn(&T) -> K + 'static,
) -> Element
where
    T: 'static,
    K: Eq + std::hash::Hash + Clone + std::fmt::Debug + 'static,
{
    ForElement {
        each: Rc::new(each),
        child: Rc::new(child),
        key_fn: Rc::new(key_fn),
    }
    .into()
}

struct ForElement<T, K> {
    each: Rc<dyn Fn() -> Vec<T>>,
    child: Rc<dyn Fn(&T) -> Element>,
    key_fn: Rc<dyn Fn(&T) -> K>,
}

struct ForItemEntry {
    mounted: MountedElement,
    owner: Rc<Owner>,
}

struct ForState<T, K> {
    container: RefCell<Column>,
    entries: RefCell<Vec<ForItemEntry>>,
    each: RefCell<Rc<dyn Fn() -> Vec<T>>>,
    child: RefCell<Rc<dyn Fn(&T) -> Element>>,
    key_fn: RefCell<Rc<dyn Fn(&T) -> K>>,
}

struct PreparedForEntry {
    element: Element,
    owner: Rc<Owner>,
}

impl<T: 'static, K: Eq + std::hash::Hash + Clone + 'static> ForState<T, K> {
    fn cleanup(&self) {
        let entries = std::mem::take(&mut *self.entries.borrow_mut());
        for entry in entries {
            entry.mounted.cleanup_recursive();
            entry.owner.dispose();
        }
    }

    fn rebuild(&self, next_entries: Vec<PreparedForEntry>) -> ArkUIResult<()> {
        let old_count = self.container.borrow().raw().children().len();
        for i in (0..old_count).rev() {
            if let Ok(Some(removed)) = self.container.borrow_mut().remove_child(i) {
                let _ = dispose_node_handle(removed);
            }
        }

        let old_entries = std::mem::take(&mut *self.entries.borrow_mut());
        for entry in old_entries {
            entry.mounted.cleanup_recursive();
            entry.owner.dispose();
        }

        let mut mounted_entries = Vec::with_capacity(next_entries.len());
        for entry in next_entries {
            let PreparedForEntry { element, owner } = entry;
            let (child_node, child_meta) = with_owner(owner.clone(), || mount_element(element))?;
            self.container.borrow_mut().add_child(child_node.clone())?;
            mounted_entries.push(ForItemEntry {
                mounted: child_meta,
                owner,
            });
        }

        self.entries.replace(mounted_entries);
        schedule_after_mount_effects();
        Ok(())
    }
}

fn prepare_for_entries<T: 'static, K: Eq + std::hash::Hash + Clone + 'static>(
    state: &ForState<T, K>,
) -> Vec<PreparedForEntry> {
    let items = (state.each.borrow().clone())();
    let child = state.child.borrow().clone();
    let key_fn = state.key_fn.borrow().clone();

    items
        .into_iter()
        .map(|item| {
            let _key = key_fn(&item);
            let (element, owner) = with_child_owner(|| child(&item));
            PreparedForEntry { element, owner }
        })
        .collect()
}

impl<T: 'static, K: Eq + std::hash::Hash + Clone + 'static> ViewNode for ForElement<T, K> {
    fn kind(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<&str> {
        None
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let Self {
            each,
            child,
            key_fn,
        } = *self;

        let mut container = Column::new()?;
        let container_node: ArkUINode = container.borrow_mut().clone();

        let state = Rc::new(ForState::<T, K> {
            container: RefCell::new(container),
            entries: RefCell::new(Vec::new()),
            each: RefCell::new(each.clone()),
            child: RefCell::new(child.clone()),
            key_fn: RefCell::new(key_fn.clone()),
        });

        let initial_entries = prepare_for_entries(state.as_ref());
        state.rebuild(initial_entries)?;

        let cleanup_state = state.clone();
        let mut mounted = MountedElement::new(
            TypeId::of::<ForElement<T, K>>(),
            "ForElement",
            None,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        mounted.set_state(Box::new(state));

        Ok((container_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        _node: &mut ArkUINode,
        mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        let Self {
            each,
            child,
            key_fn,
        } = *self;
        let state = mounted
            .state_mut::<Rc<ForState<T, K>>>()
            .expect("for_each patch missing ForState");
        state.each.replace(each);
        state.child.replace(child);
        state.key_fn.replace(key_fn);
        let next_entries = prepare_for_entries(state.as_ref());
        state.rebuild(next_entries)
    }
}

// ── Dynamic ─────────────────────────────────────────────────────────────────

/// Render a dynamically-chosen element. When the rendered element changes
/// (different kind/key), the old one is unmounted and a new one is mounted.
///
/// ```ignore
/// dynamic(move || {
///     if dark.get() { dark_theme().into() } else { light_theme().into() }
/// })
/// ```
pub fn dynamic(render: impl Fn() -> Element + 'static) -> Element {
    DynamicElement {
        render: Rc::new(render),
    }
    .into()
}

struct DynamicElement {
    render: Rc<dyn Fn() -> Element>,
}

struct DynamicState {
    container: RefCell<Column>,
    mounted_child: RefCell<Option<MountedElement>>,
    child_owner: RefCell<Option<Rc<Owner>>>,
    render: RefCell<Rc<dyn Fn() -> Element>>,
}

impl DynamicState {
    fn update(&self, element: Element) -> ArkUIResult<()> {
        let mut mounted_child = self.mounted_child.borrow_mut();

        // Try patch in place if same kind/key
        if let Some(existing) = mounted_child.as_mut() {
            if existing.kind == element.kind() && existing.key.as_deref() == element.key() {
                let container = self.container.borrow();
                let children = container.raw().children();
                if let Some(handle) = children.first() {
                    let mut child_node: std::cell::RefMut<'_, ArkUINode> = handle.borrow_mut();
                    return element.patch(&mut child_node, existing);
                }
            }
        }

        // Replace: unmount old, mount new
        {
            let mut container = self.container.borrow_mut();
            if let Ok(Some(removed)) = container.remove_child(0) {
                let _ = dispose_node_handle(removed);
            }
        }
        if let Some(old_meta) = mounted_child.take() {
            old_meta.cleanup_recursive();
        }
        let owner = self.child_owner.borrow_mut().take();
        if let Some(owner) = owner {
            owner.dispose();
        }

        let (new_element, new_owner) = with_child_owner(|| element);
        let (child_node, child_meta) =
            with_owner(new_owner.clone(), || mount_element(new_element))?;
        self.container.borrow_mut().add_child(child_node)?;
        *mounted_child = Some(child_meta);
        self.child_owner.replace(Some(new_owner));
        schedule_after_mount_effects();
        Ok(())
    }

    fn cleanup(&self) {
        let meta = self.mounted_child.borrow_mut().take();
        if let Some(meta) = meta {
            meta.cleanup_recursive();
        }
        let owner = self.child_owner.borrow_mut().take();
        if let Some(owner) = owner {
            owner.dispose();
        }
    }

    fn rerender_current(&self) -> ArkUIResult<()> {
        let next = {
            let render = self.render.borrow().clone();
            render()
        };
        self.update(next)
    }
}

impl ViewNode for DynamicElement {
    fn kind(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<&str> {
        None
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let render = self.render;
        let mut container = Column::new()?;
        let container_node: ArkUINode = container.borrow_mut().clone();

        let state = Rc::new(DynamicState {
            container: RefCell::new(container),
            mounted_child: RefCell::new(None),
            child_owner: RefCell::new(None),
            render: RefCell::new(render.clone()),
        });

        // Initial render
        state.rerender_current()?;

        let cleanup_state = state.clone();
        let mut mounted = MountedElement::new(
            TypeId::of::<DynamicElement>(),
            "DynamicElement",
            None,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        mounted.set_state(Box::new(state));

        Ok((container_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        _node: &mut ArkUINode,
        mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        let Self { render } = *self;
        let state = mounted
            .state_mut::<Rc<DynamicState>>()
            .expect("dynamic patch missing DynamicState");
        state.render.replace(render);
        state.rerender_current()
    }
}
