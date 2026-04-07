use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::rc::Rc;

use crate::ohos_arkui_binding::common::error::ArkUIResult;
use crate::ohos_arkui_binding::common::node::ArkUINode;
use crate::view::Element;

pub(crate) type Cleanup = Box<dyn FnOnce()>;

pub(crate) struct MountedElement {
    pub(crate) kind: TypeId,
    pub(crate) name: &'static str,
    pub(crate) key: Option<String>,
    pub(crate) cleanups: Vec<Cleanup>,
    pub(crate) children: Vec<MountedElement>,
    pub(crate) state: Option<Box<dyn Any>>,
}

impl MountedElement {
    pub(crate) fn new(
        kind: TypeId,
        name: &'static str,
        key: Option<String>,
        cleanups: Vec<Cleanup>,
        children: Vec<MountedElement>,
    ) -> Self {
        Self {
            kind,
            name,
            key,
            cleanups,
            children,
            state: None,
        }
    }

    pub(crate) fn set_state(&mut self, state: Box<dyn Any>) {
        self.state = Some(state);
    }

    pub(crate) fn state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.state.as_mut()?.downcast_mut::<T>()
    }

    pub(crate) fn replace_cleanups(&mut self, cleanups: Vec<Cleanup>) {
        run_cleanups(std::mem::replace(&mut self.cleanups, cleanups));
    }

    pub(crate) fn cleanup_recursive(self) {
        for child in self.children {
            child.cleanup_recursive();
        }
        run_cleanups(self.cleanups);
    }
}

pub(crate) fn run_cleanups(mut cleanups: Vec<Cleanup>) {
    while let Some(cleanup) = cleanups.pop() {
        cleanup();
    }
}

pub(crate) fn mount_element(element: Element) -> ArkUIResult<(ArkUINode, MountedElement)> {
    element.mount()
}

pub(crate) fn patch_element(
    element: Element,
    node: &mut ArkUINode,
    mounted: &mut MountedElement,
) -> ArkUIResult<()> {
    element.patch(node, mounted)
}

pub(crate) fn dispose_node_handle(handle: Rc<RefCell<ArkUINode>>) -> ArkUIResult<()> {
    let mut node = handle.borrow().clone();
    node.dispose()
}
