use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::component::{mount_element, MountedElement};
use crate::logging;
use crate::ohos_arkui_binding::common::error::ArkUIResult;
use crate::ohos_arkui_binding::common::node::ArkUINode;
use crate::ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use crate::ohos_arkui_binding::component::built_in_component::Column;
use crate::owner::{with_child_owner, Owner};
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
}

impl ShowState {
    fn mount_child(&self, child_fn: &dyn Fn() -> Element) -> ArkUIResult<()> {
        let (element, child_owner) = with_child_owner(|| child_fn());
        let (child_node, child_meta) = mount_element(element)?;
        self.container.borrow_mut().add_child(child_node)?;
        self.mounted_child.replace(Some(child_meta));
        self.child_owner.replace(Some(child_owner));
        schedule_after_mount_effects();
        Ok(())
    }

    fn unmount_child(&self) -> ArkUIResult<()> {
        if let Some(removed) = self.container.borrow_mut().remove_child(0)? {
            removed.borrow_mut().dispose()?;
        }
        if let Some(meta) = self.mounted_child.borrow_mut().take() {
            meta.cleanup_recursive();
        }
        if let Some(owner) = self.child_owner.borrow_mut().take() {
            owner.dispose();
        }
        Ok(())
    }

    fn cleanup(&self) {
        if let Some(meta) = self.mounted_child.borrow_mut().take() {
            meta.cleanup_recursive();
        }
        if let Some(owner) = self.child_owner.borrow_mut().take() {
            owner.dispose();
        }
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
        });

        // Initial render
        if (condition)() {
            state.mount_child(child.as_ref())?;
        }

        // Reactive effect: re-evaluate condition when signals change
        let effect_state = state.clone();
        let prev_value = Rc::new(Cell::new((condition)()));
        crate::effect::create_effect(move || {
            let next = condition();
            if next == prev_value.get() {
                return;
            }
            prev_value.set(next);
            let update_state = effect_state.clone();
            let update_child = child.clone();
            crate::runtime::queue_ui_loop(move || {
                let result = if next {
                    update_state.mount_child(update_child.as_ref())
                } else {
                    update_state.unmount_child()
                };
                if let Err(e) = result {
                    logging::error(format!("show: update failed: {e}"));
                }
            });
        });

        let cleanup_state = state.clone();
        let mounted = MountedElement::new(
            TypeId::of::<ShowElement>(),
            std::any::type_name::<ShowElement>(),
            None,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );

        Ok((container_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        _node: &mut ArkUINode,
        _mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        // Self-managed via reactive effects — patch is a no-op.
        Ok(())
    }
}

// ── For ─────────────────────────────────────────────────────────────────────

/// Render a reactive keyed list of items. When the list signal changes,
/// items are added/removed/reordered with minimal DOM mutations.
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

struct ForItemEntry<K> {
    key: K,
    node: ArkUINode,
    mounted: MountedElement,
    owner: Rc<Owner>,
}

struct ForState<K> {
    container: RefCell<Column>,
    entries: RefCell<Vec<ForItemEntry<K>>>,
}

impl<K: Eq + std::hash::Hash + Clone> ForState<K> {
    fn cleanup(&self) {
        for entry in self.entries.borrow_mut().drain(..) {
            entry.mounted.cleanup_recursive();
            entry.owner.dispose();
        }
    }
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

        let state = Rc::new(ForState::<K> {
            container: RefCell::new(container),
            entries: RefCell::new(Vec::new()),
        });

        // Initial render (untracked so we don't create deps from initial mount)
        {
            let items = crate::effect::untrack(|| each());
            let mut entries = state.entries.borrow_mut();
            let mut container_guard = state.container.borrow_mut();
            for item in &items {
                let (element, item_owner) = with_child_owner(|| child(item));
                let (child_node, child_meta) = mount_element(element)?;
                container_guard.add_child(child_node.clone())?;
                entries.push(ForItemEntry {
                    key: key_fn(item),
                    node: child_node,
                    mounted: child_meta,
                    owner: item_owner,
                });
            }
        }

        // Reactive effect for list changes
        let effect_state = state.clone();
        crate::effect::create_effect(move || {
            let next_items = each();
            let next_keys: Vec<K> = next_items.iter().map(|item| key_fn(item)).collect();

            // Build new elements for items that don't exist yet (must happen in
            // the reactive context so with_child_owner has the right parent).
            let current_keys: std::collections::HashSet<K> = effect_state
                .entries
                .borrow()
                .iter()
                .map(|e| e.key.clone())
                .collect();

            let mut new_item_data: Vec<Option<(Element, Rc<Owner>)>> =
                Vec::with_capacity(next_keys.len());
            for (i, item) in next_items.iter().enumerate() {
                let key = &next_keys[i];
                if current_keys.contains(key) {
                    new_item_data.push(None);
                } else {
                    let (el, owner) = with_child_owner(|| child(item));
                    new_item_data.push(Some((el, owner)));
                }
            }

            let update_state = effect_state.clone();
            crate::runtime::queue_ui_loop(move || {
                if let Err(e) = reconcile_for(&update_state, next_keys, new_item_data) {
                    logging::error(format!("for_each: reconciliation failed: {e}"));
                }
            });
        });

        let cleanup_state = state.clone();
        let mounted = MountedElement::new(
            TypeId::of::<ForElement<T, K>>(),
            "ForElement",
            None,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );

        Ok((container_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        _node: &mut ArkUINode,
        _mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        // Self-managed via effects
        Ok(())
    }
}

fn reconcile_for<K: Eq + std::hash::Hash + Clone>(
    state: &ForState<K>,
    next_keys: Vec<K>,
    new_item_data: Vec<Option<(Element, Rc<Owner>)>>,
) -> ArkUIResult<()> {
    use std::collections::HashMap;

    let mut entries = state.entries.borrow_mut();
    let mut container = state.container.borrow_mut();

    // Build key→index map for current entries
    let mut old_map: HashMap<K, usize> = HashMap::new();
    for (i, entry) in entries.iter().enumerate() {
        old_map.insert(entry.key.clone(), i);
    }

    // Drain all existing entries out so we can move them freely.
    let mut old_entries: Vec<Option<ForItemEntry<K>>> = entries.drain(..).map(Some).collect();
    let mut matched = vec![false; old_entries.len()];

    // Build result list: reuse matched old entries or mount new ones.
    let mut result_entries: Vec<ForItemEntry<K>> = Vec::with_capacity(next_keys.len());

    for (key, data) in next_keys.into_iter().zip(new_item_data.into_iter()) {
        if let Some(&old_idx) = old_map.get(&key) {
            // Reuse existing entry by taking it out of the old vec
            matched[old_idx] = true;
            if let Some(mut entry) = old_entries[old_idx].take() {
                entry.key = key;
                result_entries.push(entry);
            }
        } else if let Some((element, owner)) = data {
            // Mount new item
            let (child_node, child_meta) = mount_element(element)?;
            result_entries.push(ForItemEntry {
                key,
                node: child_node,
                mounted: child_meta,
                owner,
            });
        }
    }

    // Cleanup and dispose unmatched old entries
    for (i, was_matched) in matched.iter().enumerate() {
        if !was_matched {
            if let Some(entry) = old_entries[i].take() {
                entry.mounted.cleanup_recursive();
                entry.owner.dispose();
            }
        }
    }

    // Remove all children from the container (in reverse order)
    let old_count = container.raw().children().len();
    for i in (0..old_count).rev() {
        if let Ok(Some(removed)) = container.remove_child(i) {
            // Don't dispose matched nodes — they're reused
            if !matched.get(i).copied().unwrap_or(false) {
                let _ = removed.borrow_mut().dispose();
            }
        }
    }

    // Add children back in new order
    for entry in &result_entries {
        container.add_child(entry.node.clone())?;
    }

    *entries = result_entries;
    schedule_after_mount_effects();
    Ok(())
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
                let _ = removed.borrow_mut().dispose();
            }
        }
        if let Some(old_meta) = mounted_child.take() {
            old_meta.cleanup_recursive();
        }
        if let Some(owner) = self.child_owner.borrow_mut().take() {
            owner.dispose();
        }

        let (new_element, new_owner) = with_child_owner(|| element);
        let (child_node, child_meta) = mount_element(new_element)?;
        self.container.borrow_mut().add_child(child_node)?;
        *mounted_child = Some(child_meta);
        self.child_owner.replace(Some(new_owner));
        schedule_after_mount_effects();
        Ok(())
    }

    fn cleanup(&self) {
        if let Some(meta) = self.mounted_child.borrow_mut().take() {
            meta.cleanup_recursive();
        }
        if let Some(owner) = self.child_owner.borrow_mut().take() {
            owner.dispose();
        }
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
        });

        // Initial render
        let initial = crate::effect::untrack(|| render());
        state.update(initial)?;

        // Reactive effect
        let effect_state = state.clone();
        crate::effect::create_effect(move || {
            let next = render();
            let update_state = effect_state.clone();
            crate::runtime::queue_ui_loop(move || {
                if let Err(e) = update_state.update(next) {
                    logging::error(format!("dynamic: update failed: {e}"));
                }
            });
        });

        let cleanup_state = state.clone();
        let mounted = MountedElement::new(
            TypeId::of::<DynamicElement>(),
            "DynamicElement",
            None,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );

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
