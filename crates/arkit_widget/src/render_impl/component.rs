use std::any::Any;
use std::marker::PhantomData;

use super::*;

#[path = "component/component_effects.rs"]
mod component_effects;
#[path = "component/component_events.rs"]
mod component_events;
#[path = "component/component_specialized.rs"]
mod component_specialized;
#[path = "component/component_style.rs"]
mod component_style;

pub struct Component<Message, AppTheme = arkit_core::Theme, Kind = ()> {
    node: Node<Message, AppTheme>,
    _kind: PhantomData<fn() -> Kind>,
}

impl<Message, AppTheme, Kind> Component<Message, AppTheme, Kind> {
    pub(super) fn from_node(node: Node<Message, AppTheme>) -> Self {
        Self {
            node,
            _kind: PhantomData,
        }
    }

    pub(super) fn into_node(self) -> Node<Message, AppTheme> {
        self.node
    }

    #[cfg(test)]
    pub(super) fn kind(&self) -> NodeKind {
        self.node.kind
    }

    #[cfg(test)]
    pub(super) fn virtual_adapter_kind(&self) -> Option<VirtualContainerKind> {
        self.node.virtual_adapter.as_ref().map(|spec| spec.kind)
    }

    pub(super) fn virtual_adapter(
        self,
        kind: VirtualContainerKind,
        total_count: u32,
        render_item: Rc<dyn Fn(u32) -> Element<Message, AppTheme>>,
    ) -> Self {
        self.map_node(|node| node.virtual_adapter(kind, total_count, render_item))
    }

    pub(super) fn map_node(
        mut self,
        update: impl FnOnce(Node<Message, AppTheme>) -> Node<Message, AppTheme>,
    ) -> Self {
        self.node = update(self.node);
        self
    }
}

impl<Message, AppTheme, Kind> From<Component<Message, AppTheme, Kind>> for Node<Message, AppTheme> {
    fn from(value: Component<Message, AppTheme, Kind>) -> Self {
        value.node
    }
}

impl<Message, AppTheme> From<Node<Message, AppTheme>> for Component<Message, AppTheme> {
    fn from(value: Node<Message, AppTheme>) -> Self {
        Self::from_node(value)
    }
}

impl<Message: 'static, AppTheme: 'static, Kind: 'static> From<Component<Message, AppTheme, Kind>>
    for Element<Message, AppTheme>
{
    fn from(value: Component<Message, AppTheme, Kind>) -> Self {
        value.node.into()
    }
}

impl<Message: 'static, AppTheme: 'static, Kind: 'static>
    advanced::Widget<Message, AppTheme, Renderer> for Component<Message, AppTheme, Kind>
{
    fn tag(&self) -> advanced::widget::Tag {
        advanced::Widget::tag(&self.node)
    }

    fn state(&self) -> advanced::widget::State {
        advanced::Widget::state(&self.node)
    }

    fn persistent_key(&self) -> Option<&str> {
        advanced::Widget::persistent_key(&self.node)
    }

    fn children(&self) -> Vec<advanced::widget::Tree> {
        advanced::Widget::children(&self.node)
    }

    fn diff(&self, tree: &mut advanced::widget::Tree)
    where
        Self: 'static,
    {
        advanced::Widget::diff(&self.node, tree);
    }

    fn size_hint(&self) -> Size<Length> {
        advanced::Widget::size_hint(&self.node)
    }

    fn layout(&self) -> arkit_core::layout::Node {
        advanced::Widget::layout(&self.node)
    }
}

macro_rules! component_kinds {
    ($($kind:ident => $alias:ident,)*) => {
        $(
            #[doc(hidden)]
            #[allow(dead_code)]
            pub struct $kind;
            pub type $alias<Message = (), AppTheme = arkit_core::Theme> =
                Component<Message, AppTheme, $kind>;
        )*

        pub(super) fn is_component_widget<Message: 'static, AppTheme: 'static>(
            value: &dyn Any,
        ) -> bool {
            false $(|| value.is::<$alias<Message, AppTheme>>())*
        }

        pub(super) fn component_into_node<Message: 'static, AppTheme: 'static>(
            value: Box<dyn Any>,
        ) -> Option<Node<Message, AppTheme>> {
            $(
                if value.is::<$alias<Message, AppTheme>>() {
                    return value
                        .downcast::<$alias<Message, AppTheme>>()
                        .ok()
                        .map(|component| component.into_node());
                }
            )*
            None
        }
    };
}

component_kinds! {
    ButtonKind => ButtonElement,
    CalendarPickerKind => CalendarPickerElement,
    CheckboxKind => CheckboxElement,
    ContainerKind => ContainerElement,
    ColumnKind => ColumnElement,
    DatePickerKind => DatePickerElement,
    FlexKind => FlexElement,
    FlowItemKind => FlowItemElement,
    GridKind => GridElement,
    GridItemKind => GridItemElement,
    ImageKind => ImageElement,
    ListKind => ListElement,
    ListItemKind => ListItemElement,
    ListItemGroupKind => ListItemGroupElement,
    ProgressKind => ProgressElement,
    RadioKind => RadioElement,
    RefreshKind => RefreshElement,
    RowKind => RowElement,
    ScrollKind => ScrollElement,
    SliderKind => SliderElement,
    StackKind => StackElement,
    SwiperKind => SwiperElement,
    TextAreaKind => TextAreaElement,
    TextKind => TextElement,
    TextInputKind => TextInputElement,
    ToggleKind => ToggleElement,
    WaterFlowKind => WaterFlowElement,
    WebViewKind => WebViewElement,
}
