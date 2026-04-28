use crate::Message;
use arkit::prelude::*;
use arkit::Element;

pub(crate) struct PageShell {
    title: &'static str,
    children: std::cell::RefCell<Option<Vec<Element<Message>>>>,
}

impl PageShell {
    pub(crate) fn new(title: &'static str, children: Vec<Element<Message>>) -> Self {
        Self {
            title,
            children: std::cell::RefCell::new(Some(children)),
        }
    }
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for PageShell {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let mut children = vec![text(self.title)
            .font_size(28.0)
            .font_weight(FontWeight::W700)
            .line_height(34.0)
            .font_color(0xFF0F172A)
            .into()];
        children.extend(
            self.children
                .borrow_mut()
                .take()
                .expect("PageShell children consumed once"),
        );

        Some(
            column_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .padding(24.0)
                .background_color(0xFFF8FAFC)
                .children(children)
                .into(),
        )
    }
}

pub(crate) struct NavButton {
    label: &'static str,
    message: Message,
}

impl NavButton {
    pub(crate) fn new(label: &'static str, message: Message) -> Self {
        Self { label, message }
    }
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for NavButton {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(
            button(self.label)
                .margin([12.0, 8.0, 0.0, 0.0])
                .padding([8.0, 12.0, 8.0, 12.0])
                .background_color(0xFF111827)
                .font_color(0xFFFFFFFF)
                .border_radius(6.0)
                .on_press(self.message.clone())
                .into(),
        )
    }
}
