use crate::components::PageShell;
use crate::Message;
use arkit::prelude::*;
use arkit::router::{FallbackRouteContext, FallbackRoutePage};
use arkit::Element;

pub(crate) struct NotFoundPage {
    path: String,
}

impl FallbackRoutePage<Message> for NotFoundPage {
    fn from_route(context: FallbackRouteContext) -> Self {
        Self {
            path: context.raw().path().to_string(),
        }
    }
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for NotFoundPage {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(Element::new(PageShell::new(
            "Not Found",
            vec![text(self.path.clone()).into()],
        )))
    }
}
