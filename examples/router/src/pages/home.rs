use crate::components::{NavButton, PageShell};
use crate::routes::{SettingsRoute, UserNavigationState, UserRoute};
use crate::Message;
use arkit::prelude::*;
use arkit::router::{RouteContext, RoutePage};
use arkit::{router::Router, Element};

pub(crate) struct HomePage {
    router: Router,
}

impl RoutePage<Message, crate::routes::HomeRoute> for HomePage {
    fn from_route(context: RouteContext<crate::routes::HomeRoute>) -> Self {
        Self {
            router: context.router().clone(),
        }
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
                Element::new(NavButton::new("Open user 42", {
                    let router = self.router.clone();
                    move || {
                        let _ = router.navigate_with_state(
                            UserRoute { id: 42 },
                            UserNavigationState {
                                source: String::from("home page"),
                                scroll_offset: 320,
                            },
                        );
                    }
                })),
                Element::new(NavButton::new("Settings", {
                    let router = self.router.clone();
                    move || {
                        let _ = router.navigate(SettingsRoute);
                    }
                })),
            ],
        )))
    }
}
