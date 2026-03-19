use crate::ohos_arkui_binding::api::node_custom_event::NodeCustomEvent;
use crate::ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use crate::ohos_arkui_binding::common::error::ArkUIResult;
use crate::ohos_arkui_binding::common::node::ArkUINode;
use crate::ohos_arkui_binding::component::attribute::{
    ArkUICommonAttribute, ArkUICommonFontAttribute, ArkUIEvent, ArkUIGesture,
};
use crate::ohos_arkui_binding::event::inner_event::Event as ArkEvent;
use crate::ohos_arkui_binding::types::advanced::NodeCustomEventType;
use crate::ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use crate::ohos_arkui_binding::types::event::NodeEventType;

use super::element::{Element, ViewNode};

type Mutator<T> = Box<dyn FnOnce(&mut T) -> ArkUIResult<()>>;

pub struct ComponentElement<T> {
    constructor: fn() -> ArkUIResult<T>,
    mutators: Vec<Mutator<T>>,
    children: Vec<Element>,
}

impl<T> ComponentElement<T> {
    pub fn new(constructor: fn() -> ArkUIResult<T>) -> Self {
        Self {
            constructor,
            mutators: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn with(mut self, mutator: impl FnOnce(&mut T) -> ArkUIResult<()> + 'static) -> Self {
        self.mutators.push(Box::new(mutator));
        self
    }

    pub fn native(self, mutator: impl FnOnce(&mut T) -> ArkUIResult<()> + 'static) -> Self {
        self.with(mutator)
    }
}

impl<T> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        let value = value.into();
        self.mutators
            .push(Box::new(move |node| node.set_attribute(attr, value)));
        self
    }

    pub fn style(
        self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.attr(attr, value)
    }

    pub fn width(mut self, value: f32) -> Self {
        self.mutators.push(Box::new(move |node| node.width(value)));
        self
    }

    pub fn height(mut self, value: f32) -> Self {
        self.mutators.push(Box::new(move |node| node.height(value)));
        self
    }

    pub fn percent_width(mut self, value: f32) -> Self {
        self.mutators
            .push(Box::new(move |node| node.percent_width(value)));
        self
    }

    pub fn percent_height(mut self, value: f32) -> Self {
        self.mutators
            .push(Box::new(move |node| node.percent_height(value)));
        self
    }

    pub fn background_color(mut self, value: u32) -> Self {
        self.mutators
            .push(Box::new(move |node| node.background_color(value)));
        self
    }

    pub fn gesture(mut self, mutator: impl FnOnce(&mut T) -> ArkUIResult<()> + 'static) -> Self
    where
        T: ArkUIGesture,
    {
        self.mutators.push(Box::new(mutator));
        self
    }
}

impl<T> ComponentElement<T>
where
    T: ArkUICommonFontAttribute + 'static,
{
    pub fn font_size(mut self, value: f32) -> Self {
        self.mutators
            .push(Box::new(move |node| node.font_size(value)));
        self
    }
}

impl<T> ComponentElement<T>
where
    T: ArkUIEvent + 'static,
{
    pub fn on_click(mut self, callback: impl Fn() + 'static) -> Self {
        self.mutators.push(Box::new(move |node| {
            node.on_click(move || callback());
            Ok(())
        }));
        self
    }

    pub fn on_event(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn(&ArkEvent) + 'static,
    ) -> Self {
        self.mutators.push(Box::new(move |node| {
            node.on_event(event_type, move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_event_no_param(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn() + 'static,
    ) -> Self {
        self.mutators.push(Box::new(move |node| {
            node.on_event_no_param(event_type, move || callback());
            Ok(())
        }));
        self
    }

    pub fn on_custom_event(
        mut self,
        event_type: NodeCustomEventType,
        callback: impl Fn(&NodeCustomEvent) + 'static,
    ) -> Self {
        self.mutators.push(Box::new(move |node| {
            node.on_custom_event(event_type, move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_measure(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.mutators.push(Box::new(move |node| {
            node.on_custom_measure(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_layout(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.mutators.push(Box::new(move |node| {
            node.on_custom_layout(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_draw(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.mutators.push(Box::new(move |node| {
            node.on_custom_draw(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_foreground_draw(
        mut self,
        callback: impl Fn(&NodeCustomEvent) + 'static,
    ) -> Self {
        self.mutators.push(Box::new(move |node| {
            node.on_custom_foreground_draw(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_overlay_draw(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.mutators.push(Box::new(move |node| {
            node.on_custom_overlay_draw(move |event| callback(event));
            Ok(())
        }));
        self
    }
}

impl<T> ViewNode for ComponentElement<T>
where
    T: ArkUICommonAttribute + Into<ArkUINode> + 'static,
{
    fn build(self: Box<Self>) -> ArkUIResult<ArkUINode> {
        let Self {
            constructor,
            mutators,
            children,
        } = *self;

        let mut node = constructor()?;
        for mutator in mutators {
            mutator(&mut node)?;
        }
        for child in children {
            node.add_child(child.build()?)?;
        }

        Ok(node.into())
    }
}
