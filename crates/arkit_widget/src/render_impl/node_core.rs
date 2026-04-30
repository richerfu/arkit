use super::*;

pub struct Node<Message, AppTheme = arkit_core::Theme> {
    pub(super) kind: NodeKind,
    pub(super) key: Option<String>,
    pub(super) persistent_key: Option<String>,
    pub(super) init_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    pub(super) patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    pub(super) event_handlers: Vec<EventHandlerSpec>,
    pub(super) long_press_handler: Option<LongPressHandlerSpec>,
    pub(super) mount_effects: Vec<MountEffect>,
    pub(super) attach_effects: Vec<AttachEffect>,
    pub(super) patch_effects: Vec<PatchEffect>,
    pub(super) exit_effect: Option<ExitEffect>,
    pub(super) state_bound: bool,
    pub(super) virtual_adapter: Option<VirtualAdapterSpec<Message, AppTheme>>,
    #[cfg(feature = "webview")]
    pub(super) webview: Option<WebViewSpec>,
    pub(super) children: Vec<Element<Message, AppTheme>>,
}

impl<Message, AppTheme> Node<Message, AppTheme> {
    pub(super) fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            key: None,
            persistent_key: None,
            init_attrs: Vec::new(),
            patch_attrs: Vec::new(),
            event_handlers: Vec::new(),
            long_press_handler: None,
            mount_effects: Vec::new(),
            attach_effects: Vec::new(),
            patch_effects: Vec::new(),
            exit_effect: None,
            state_bound: false,
            virtual_adapter: None,
            #[cfg(feature = "webview")]
            webview: None,
            children: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(super) fn kind(&self) -> NodeKind {
        self.kind
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn persistent_state_key(mut self, key: impl Into<String>) -> Self {
        self.persistent_key = Some(key.into());
        self
    }

    pub fn child(mut self, child: impl Into<Element<Message, AppTheme>>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(mut self, children: Vec<Element<Message, AppTheme>>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn map_descendants(self, mut map: impl FnMut(Self) -> Self) -> Self
    where
        Message: 'static,
        AppTheme: 'static,
    {
        self.map_descendants_with(&mut map)
    }

    pub(super) fn map_descendants_with(self, map: &mut impl FnMut(Self) -> Self) -> Self
    where
        Message: 'static,
        AppTheme: 'static,
    {
        let Self {
            kind,
            key,
            persistent_key,
            init_attrs,
            patch_attrs,
            event_handlers,
            long_press_handler,
            mount_effects,
            attach_effects,
            patch_effects,
            exit_effect,
            state_bound,
            virtual_adapter,
            #[cfg(feature = "webview")]
            webview,
            children,
        } = self;

        let children = children
            .into_iter()
            .map(|child| into_node(child).map_descendants_with(map).into())
            .collect();

        map(Self {
            kind,
            key,
            persistent_key,
            init_attrs,
            patch_attrs,
            event_handlers,
            long_press_handler,
            mount_effects,
            attach_effects,
            patch_effects,
            exit_effect,
            state_bound,
            virtual_adapter,
            #[cfg(feature = "webview")]
            webview,
            children,
        })
    }

    pub fn attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.init_attrs.push((attr, value.into()));
        self
    }

    pub fn patch_attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.patch_attrs.push((attr, value.into()));
        self
    }

    pub(super) fn builder_attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        let value = value.into();
        self.init_attrs.push((attr, clone_attr_value(&value)));
        self.patch_attrs.push((attr, value));
        self
    }

    pub(super) fn virtual_adapter(
        mut self,
        kind: VirtualContainerKind,
        total_count: u32,
        render_item: Rc<dyn Fn(u32) -> Element<Message, AppTheme>>,
    ) -> Self {
        self.virtual_adapter = Some(VirtualAdapterSpec {
            kind,
            total_count,
            render_item,
        });
        self
    }

    pub fn attr_string(&self, attr: ArkUINodeAttributeType) -> Option<&str> {
        self.attr_value(attr).and_then(|value| match value {
            ArkUINodeAttributeItem::String(value) => Some(value.as_str()),
            ArkUINodeAttributeItem::NumberValue(_)
            | ArkUINodeAttributeItem::Object(_)
            | ArkUINodeAttributeItem::Composite(_) => None,
        })
    }

    pub fn attr_f32(&self, attr: ArkUINodeAttributeType) -> Option<f32> {
        self.attr_value(attr).and_then(|value| match value {
            ArkUINodeAttributeItem::NumberValue(values) => {
                values.first().map(|value| match value {
                    ArkUINodeAttributeNumber::Float(value) => *value,
                    ArkUINodeAttributeNumber::Int(value) => *value as f32,
                    ArkUINodeAttributeNumber::Uint(value) => *value as f32,
                })
            }
            ArkUINodeAttributeItem::Composite(value) => {
                value.number_values.first().map(|value| match value {
                    ArkUINodeAttributeNumber::Float(value) => *value,
                    ArkUINodeAttributeNumber::Int(value) => *value as f32,
                    ArkUINodeAttributeNumber::Uint(value) => *value as f32,
                })
            }
            ArkUINodeAttributeItem::String(_) | ArkUINodeAttributeItem::Object(_) => None,
        })
    }

    pub(super) fn attr_bool(&self, attr: ArkUINodeAttributeType) -> Option<bool> {
        self.attr_value(attr).and_then(|value| match value {
            ArkUINodeAttributeItem::NumberValue(values) => {
                values.first().map(|value| match value {
                    ArkUINodeAttributeNumber::Float(value) => *value != 0.0,
                    ArkUINodeAttributeNumber::Int(value) => *value != 0,
                    ArkUINodeAttributeNumber::Uint(value) => *value != 0,
                })
            }
            ArkUINodeAttributeItem::String(_)
            | ArkUINodeAttributeItem::Object(_)
            | ArkUINodeAttributeItem::Composite(_) => None,
        })
    }

    pub(super) fn attr_value(
        &self,
        attr: ArkUINodeAttributeType,
    ) -> Option<&ArkUINodeAttributeItem> {
        self.patch_attrs
            .iter()
            .rev()
            .chain(self.init_attrs.iter().rev())
            .find_map(|(current_attr, value)| (*current_attr == attr).then_some(value))
    }

    #[cfg(feature = "webview")]
    pub(super) fn webview_spec_mut(&mut self) -> Option<&mut WebViewSpec> {
        self.webview.as_mut()
    }

    #[cfg(feature = "webview")]
    pub(super) fn map_webview(mut self, update: impl FnOnce(&mut WebViewSpec)) -> Self {
        if let Some(spec) = self.webview_spec_mut() {
            update(spec);
        }
        self
    }
}
