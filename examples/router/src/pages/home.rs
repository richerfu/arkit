use crate::components::{NavButton, PageShell};
use crate::routes::UserNavigationState;
use crate::Message;
use arkit::prelude::*;
use arkit::router::RouterMessage;
use arkit::Element;

pub(crate) struct HomePage;

impl HomePage {
    pub(crate) fn new(_context: arkit::router::RouteContext) -> Self {
        Self
    }
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for HomePage {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(Element::new(PageShell::new(
            "Home",
            vec![
                text("This page is produced by HomePage from HomeRoute.")
                    .font_size(15.0)
                    .font_color(0xFF334155)
                    .into(),
                Element::new(NavButton::new(
                    "Open user 42",
                    Message::Router(RouterMessage::push_with_state(
                        "/users/42",
                        UserNavigationState {
                            source: String::from("home page"),
                            scroll_offset: 320,
                        },
                    )),
                )),
                Element::new(NavButton::new(
                    "Settings",
                    Message::Router(RouterMessage::push("/settings")),
                )),
            ],
        )))
    }
}
