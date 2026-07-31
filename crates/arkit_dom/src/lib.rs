//! Platform-independent logical tree used by Arkit renderers.
//!
//! This crate owns Dioxus-facing node identity and tree mutation semantics. It
//! deliberately has no OpenHarmony or ArkUI dependency, so projection behavior
//! can be verified on the host toolchain.

use std::ops::{Deref, DerefMut, Index, IndexMut};

use oxc_index::IndexVec;

oxc_index::define_index_type! {
    /// Dense identity for one logical host node.
    pub struct HostId = u32;
}

/// Renderer-independent identity assigned by the upstream virtual DOM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementKey(usize);

impl ElementKey {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable overlay projection plane.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortalLayer {
    #[default]
    Modal,
    Floating,
    Transient,
}

/// Logical node kind. Native projection state belongs to the renderer payload,
/// not this foundation crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostKind {
    Root,
    Element {
        tag: &'static str,
    },
    Text {
        value: String,
    },
    Placeholder,
    /// A logical node whose native projection is parented to the renderer root
    /// while its component/context ancestry remains at the source location.
    Portal {
        layer: PortalLayer,
    },
}

impl HostKind {
    pub const fn tag(&self) -> &'static str {
        match self {
            Self::Root | Self::Portal { .. } => "stack",
            Self::Element { tag } => tag,
            Self::Text { .. } => "text",
            Self::Placeholder => "stack",
        }
    }

    pub const fn portal_layer(&self) -> Option<PortalLayer> {
        match self {
            Self::Portal { layer } => Some(*layer),
            _ => None,
        }
    }
}

/// One node in the logical host tree.
#[derive(Debug)]
pub struct HostNode<P> {
    pub kind: HostKind,
    pub parent: Option<HostId>,
    pub children: Vec<HostId>,
    pub payload: P,
    bound_elements: Vec<ElementKey>,
}

impl<P> HostNode<P> {
    fn new(kind: HostKind, payload: P) -> Self {
        Self {
            kind,
            parent: None,
            children: Vec::new(),
            payload,
            bound_elements: Vec::new(),
        }
    }

    pub const fn tag(&self) -> &'static str {
        self.kind.tag()
    }
}

impl<P> Deref for HostNode<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl<P> DerefMut for HostNode<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}

/// Dense logical tree plus upstream element bindings and mutation stack.
///
/// Released slots are reused only after every bound element key is cleared.
/// This prevents an upstream id from aliasing a later node that occupies the
/// same dense slot.
pub struct HostTree<P> {
    nodes: IndexVec<HostId, HostNode<P>>,
    free: Vec<HostId>,
    element_to_host: Vec<Option<HostId>>,
    stack: Vec<HostId>,
}

impl<P: Default> HostTree<P> {
    pub fn new(root_payload: P) -> Self {
        let mut nodes = IndexVec::new();
        let root = nodes.push(HostNode::new(HostKind::Root, root_payload));
        debug_assert_eq!(root, HostId::new(0));
        Self {
            nodes,
            free: Vec::new(),
            element_to_host: vec![Some(root)],
            stack: Vec::new(),
        }
    }

    pub const fn root(&self) -> HostId {
        HostId::new(0)
    }

    pub fn is_connected_to_root(&self, host: HostId) -> bool {
        let mut current = host;
        for _ in 0..self.nodes.len() {
            if current == self.root() {
                return true;
            }
            let Some(parent) = self.nodes[current].parent else {
                return false;
            };
            current = parent;
        }
        debug_assert!(false, "host tree contains a parent cycle");
        false
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the backing arena has no nodes.
    ///
    /// A constructed tree always contains its synthetic root, so this remains
    /// false for every valid `HostTree`.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn alloc(&mut self, kind: HostKind) -> HostId {
        if let Some(id) = self.free.pop() {
            self.nodes[id] = HostNode::new(kind, P::default());
            id
        } else {
            self.nodes.push(HostNode::new(kind, P::default()))
        }
    }

    pub fn bind_element(&mut self, element: ElementKey, host: HostId) {
        if element.index() >= self.element_to_host.len() {
            self.element_to_host
                .resize_with(element.index() + 1, || None);
        }
        if let Some(previous) = self.element_to_host[element.index()] {
            self.nodes[previous]
                .bound_elements
                .retain(|candidate| *candidate != element);
        }
        self.element_to_host[element.index()] = Some(host);
        if !self.nodes[host].bound_elements.contains(&element) {
            self.nodes[host].bound_elements.push(element);
        }
    }

    pub fn host_for_element(&self, element: ElementKey) -> Option<HostId> {
        self.element_to_host.get(element.index()).copied().flatten()
    }

    pub fn release(&mut self, host: HostId) {
        assert_ne!(host, self.root(), "the synthetic root cannot be released");
        for element in std::mem::take(&mut self.nodes[host].bound_elements) {
            if self.element_to_host.get(element.index()).copied().flatten() == Some(host) {
                self.element_to_host[element.index()] = None;
            }
        }
        self.nodes[host] = HostNode::new(HostKind::Placeholder, P::default());
        self.free.push(host);
    }

    pub fn append_child(&mut self, parent: HostId, child: HostId) {
        self.nodes[child].parent = Some(parent);
        self.nodes[parent].children.push(child);
    }

    pub fn insert_child(&mut self, parent: HostId, index: usize, child: HostId) {
        self.nodes[child].parent = Some(parent);
        self.nodes[parent].children.insert(index, child);
    }

    pub fn detach_child(&mut self, parent: HostId, child: HostId) -> Option<usize> {
        let index = self.nodes[parent]
            .children
            .iter()
            .position(|candidate| *candidate == child)?;
        self.nodes[parent].children.remove(index);
        self.nodes[child].parent = None;
        Some(index)
    }

    pub fn walk_path(&self, root: HostId, path: &[u8]) -> Option<HostId> {
        let mut current = root;
        for index in path {
            current = *self.nodes[current].children.get(usize::from(*index))?;
        }
        Some(current)
    }

    pub fn push(&mut self, host: HostId) {
        self.stack.push(host);
    }

    pub fn pop(&mut self) -> Option<HostId> {
        self.stack.pop()
    }

    pub fn stack_last(&self) -> Option<HostId> {
        self.stack.last().copied()
    }
}

impl<P> Index<HostId> for HostTree<P> {
    type Output = HostNode<P>;

    fn index(&self, index: HostId) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<P> IndexMut<HostId> for HostTree<P> {
    fn index_mut(&mut self, index: HostId) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn released_slots_cannot_alias_old_element_bindings() {
        let mut tree = HostTree::<()>::new(());
        let first = tree.alloc(HostKind::Element { tag: "row" });
        tree.bind_element(ElementKey::new(4), first);
        tree.release(first);

        let reused = tree.alloc(HostKind::Element { tag: "column" });
        assert_eq!(first, reused);
        assert_eq!(tree.host_for_element(ElementKey::new(4)), None);
    }

    #[test]
    fn structural_operations_preserve_logical_parentage() {
        let mut tree = HostTree::<()>::new(());
        let row = tree.alloc(HostKind::Element { tag: "row" });
        let first = tree.alloc(HostKind::Text {
            value: "first".into(),
        });
        let second = tree.alloc(HostKind::Placeholder);
        tree.append_child(tree.root(), row);
        tree.append_child(row, second);
        tree.insert_child(row, 0, first);

        assert_eq!(tree[row].children, vec![first, second]);
        assert_eq!(tree.walk_path(row, &[1]), Some(second));
        assert_eq!(tree.detach_child(row, first), Some(0));
        assert_eq!(tree[first].parent, None);
    }

    #[test]
    fn portal_keeps_logical_parent_and_declares_projection_layer() {
        let mut tree = HostTree::<()>::new(());
        let source = tree.alloc(HostKind::Element { tag: "column" });
        let portal = tree.alloc(HostKind::Portal {
            layer: PortalLayer::Floating,
        });
        tree.append_child(tree.root(), source);
        tree.append_child(source, portal);

        assert_eq!(tree[portal].parent, Some(source));
        assert_eq!(
            tree[portal].kind.portal_layer(),
            Some(PortalLayer::Floating)
        );
    }

    #[test]
    fn root_connectivity_tracks_detach_and_reparent() {
        let mut tree = HostTree::<()>::new(());
        let parent = tree.alloc(HostKind::Element { tag: "column" });
        let child = tree.alloc(HostKind::Element { tag: "text" });
        tree.append_child(parent, child);

        assert!(!tree.is_connected_to_root(parent));
        assert!(!tree.is_connected_to_root(child));

        tree.append_child(tree.root(), parent);
        assert!(tree.is_connected_to_root(parent));
        assert!(tree.is_connected_to_root(child));

        tree.detach_child(tree.root(), parent);
        assert!(!tree.is_connected_to_root(parent));
        assert!(!tree.is_connected_to_root(child));
    }
}
