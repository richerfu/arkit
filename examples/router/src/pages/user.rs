use crate::components::{NavButton, PageShell};
use crate::routes::UserNavigationState;
use crate::Message;
use arkit::prelude::*;
use arkit::router::{Outlet, RouteContext, RouterMessage};
use arkit::Element;

pub(crate) struct UserLayout {
    id: u32,
    outlet: std::cell::RefCell<Option<Element<Message>>>,
}

impl UserLayout {
    pub(crate) fn new(context: RouteContext, outlet: Outlet<Message>) -> Self {
        Self {
            id: context.parse_param("id").unwrap_or_default(),
            outlet: std::cell::RefCell::new(Some(outlet.into())),
        }
    }
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for UserLayout {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(Element::new(PageShell::new(
            "User Layout",
            vec![
                text(format!("layout param id = {}", self.id))
                    .font_size(15.0)
                    .font_color(0xFF334155)
                    .into(),
                self.outlet
                    .borrow_mut()
                    .take()
                    .expect("UserLayout outlet consumed once"),
            ],
        )))
    }
}

pub(crate) struct UserPage {
    id: u32,
    tab: String,
    state_text: String,
}

impl UserPage {
    pub(crate) fn new(context: RouteContext) -> Self {
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
            id: context.parse_param("id").unwrap_or_default(),
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
                Element::new(NavButton::new(
                    "Settings with async guard",
                    Message::Router(RouterMessage::push(format!("/users/{}/settings", self.id))),
                )),
                Element::new(NavButton::new(
                    "Replace with user 9",
                    Message::Router(RouterMessage::replace_with_state(
                        "/users/9",
                        UserNavigationState {
                            source: String::from("user page replace"),
                            scroll_offset: 96,
                        },
                    )),
                )),
                Element::new(NavButton::new(
                    "Back",
                    Message::Router(RouterMessage::back()),
                )),
            ],
        )))
    }
}

pub(crate) struct UserSettingsPage {
    id: u32,
}

impl UserSettingsPage {
    pub(crate) fn new(context: RouteContext) -> Self {
        Self {
            id: context.parse_param("id").unwrap_or_default(),
        }
    }
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for UserSettingsPage {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(Element::new(PageShell::new(
            "User Settings",
            vec![
                text(format!("nested settings for user {}", self.id))
                    .font_size(15.0)
                    .font_color(0xFF334155)
                    .into(),
                Element::new(NavButton::new(
                    "User index",
                    Message::Router(RouterMessage::push(format!("/users/{}", self.id))),
                )),
            ],
        )))
    }
}
