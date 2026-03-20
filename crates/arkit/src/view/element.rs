use std::any::TypeId;

use crate::component::MountedElement;
use crate::ohos_arkui_binding::common::error::ArkUIResult;
use crate::ohos_arkui_binding::common::node::ArkUINode;

pub(crate) trait ViewNode {
    fn kind(&self) -> TypeId;
    fn key(&self) -> Option<&str>;
    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)>;
    fn patch(
        self: Box<Self>,
        node: &mut ArkUINode,
        mounted: &mut MountedElement,
    ) -> ArkUIResult<()>;
}

pub struct Element {
    inner: Box<dyn ViewNode>,
}

impl Element {
    pub(crate) fn kind(&self) -> TypeId {
        self.inner.kind()
    }

    pub(crate) fn key(&self) -> Option<&str> {
        self.inner.key()
    }

    pub(crate) fn mount(self) -> ArkUIResult<(ArkUINode, MountedElement)> {
        self.inner.mount()
    }

    pub(crate) fn patch(
        self,
        node: &mut ArkUINode,
        mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        self.inner.patch(node, mounted)
    }
}

impl<T> From<T> for Element
where
    T: ViewNode + 'static,
{
    fn from(value: T) -> Self {
        Self {
            inner: Box::new(value),
        }
    }
}
