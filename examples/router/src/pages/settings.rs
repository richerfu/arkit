use crate::components::{NavButton, PageShell};
use crate::Message;
use arkit::prelude::*;
use arkit::router::{RouteContext, RouterMessage};
use arkit::Element;

pub(crate) struct SettingsPage;

impl SettingsPage {
    pub(crate) fn new(_context: RouteContext) -> Self {
        Self
    }
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for SettingsPage {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(Element::new(PageShell::new(
            "Settings",
            vec![
                text("SettingsPage is bound to SettingsRoute.")
                    .font_size(15.0)
                    .font_color(0xFF334155)
                    .into(),
                Element::new(NavButton::new(
                    "Home",
                    Message::Router(RouterMessage::push("/")),
                )),
            ],
        )))
    }
}
