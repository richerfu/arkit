use crate::ohos_arkui_binding::common::error::ArkUIResult;
use crate::ohos_arkui_binding::common::node::ArkUINode;

pub(crate) trait ViewNode {
    fn build(self: Box<Self>) -> ArkUIResult<ArkUINode>;
}

pub struct Element {
    inner: Box<dyn ViewNode>,
}

impl Element {
    pub(crate) fn build(self) -> ArkUIResult<ArkUINode> {
        self.inner.build()
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
