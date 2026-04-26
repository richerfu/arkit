use crate::components::{NavButton, PageShell};
use crate::routes::{HomeRoute, SettingsRoute};
use crate::Message;
use arkit::prelude::*;
use arkit::router::{RouteContext, RoutePage, Router};
use arkit::Element;

pub(crate) struct SettingsPage {
    router: Router,
}

impl RoutePage<Message, SettingsRoute> for SettingsPage {
    fn from_route(context: RouteContext<SettingsRoute>) -> Self {
        Self {
            router: context.router().clone(),
        }
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
                Element::new(NavButton::new("Home", {
                    let router = self.router.clone();
                    move || {
                        let _ = router.navigate(HomeRoute);
                    }
                })),
            ],
        )))
    }
}
