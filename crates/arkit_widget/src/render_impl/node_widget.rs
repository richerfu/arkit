use super::*;

impl<Message: 'static, AppTheme: 'static> advanced::Widget<Message, AppTheme, Renderer>
    for Node<Message, AppTheme>
{
    fn tag(&self) -> advanced::widget::Tag {
        node_widget_tag(self.kind)
    }

    fn state(&self) -> advanced::widget::State {
        match self.kind {
            NodeKind::Scroll => advanced::widget::State::new(Box::new(Rc::new(RefCell::new(
                ScrollState::default(),
            )))),
            _ => advanced::widget::State::none(),
        }
    }

    fn persistent_key(&self) -> Option<&str> {
        self.persistent_key.as_deref()
    }

    fn children(&self) -> Vec<advanced::widget::Tree> {
        self.children
            .iter()
            .map(arkit_core::advanced::tree_of)
            .collect()
    }

    fn diff(&self, tree: &mut advanced::widget::Tree)
    where
        Self: 'static,
    {
        tree.set_tag(self.tag());
        tree.set_persistent_key(self.persistent_key.clone());
    }

    fn size_hint(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&self) -> arkit_core::layout::Node {
        arkit_core::layout::Node::new(Size::new(0.0, 0.0))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl<Message, AppTheme> From<Node<Message, AppTheme>> for Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    fn from(value: Node<Message, AppTheme>) -> Self {
        arkit_core::Element::new(value)
    }
}
