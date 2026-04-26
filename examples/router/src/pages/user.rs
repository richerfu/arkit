use crate::components::{NavButton, PageShell};
use crate::routes::{UserNavigationState, UserRoute};
use crate::Message;
use arkit::prelude::*;
use arkit::router::{RouteContext, RoutePage, Router};
use arkit::Element;

pub(crate) struct UserPage {
    router: Router,
    id: u32,
    tab: String,
    state_text: String,
}

impl RoutePage<Message, UserRoute> for UserPage {
    fn from_route(context: RouteContext<UserRoute>) -> Self {
        let state_text = context
            .state::<UserNavigationState>()
            .map(|state| {
                format!(
                    "state source={}, scroll_offset={}",
                    state.source, state.scroll_offset
                )
            })
            .unwrap_or_else(|| String::from("state none"));

        Self {
            router: context.router().clone(),
            id: context.route().id,
            tab: context
                .query("tab")
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| String::from("<none>")),
            state_text,
        }
    }
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for UserPage {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(Element::new(PageShell::new(
            "User",
            vec![
                text(format!("typed param id = {}", self.id))
                    .font_size(15.0)
                    .font_color(0xFF334155)
                    .into(),
                text(format!("query tab = {}", self.tab))
                    .margin_top(6.0)
                    .font_size(15.0)
                    .font_color(0xFF334155)
                    .into(),
                text(self.state_text.clone())
                    .margin_top(6.0)
                    .font_size(15.0)
                    .font_color(0xFF334155)
                    .into(),
                Element::new(NavButton::new("Replace with user 9", {
                    let router = self.router.clone();
                    move || {
                        let _ = router.replace_with_state(
                            UserRoute { id: 9 },
                            UserNavigationState {
                                source: String::from("user page replace"),
                                scroll_offset: 96,
                            },
                        );
                    }
                })),
                Element::new(NavButton::new("Back", {
                    let router = self.router.clone();
                    move || {
                        let _ = router.back();
                    }
                })),
            ],
        )))
    }
}
