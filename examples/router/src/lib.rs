use arkit::entry;
use arkit::router::{
    NavigationEvent, RouteGuardContext, RouteGuardDecision, Router, RouterMessage,
    RouterNavigationExt, RouterOutlet, Routes,
};
use arkit::{application, Element, Task};
use std::time::Duration;

mod components;
mod pages;
mod routes;

use pages::{HomePage, NotFoundPage, SettingsPage, UserLayout, UserPage, UserSettingsPage};

#[derive(Debug, Clone)]
enum Message {
    Router(RouterMessage),
}

#[derive(Clone)]
struct AppState {
    router: Router,
    last_navigation: Option<NavigationEvent>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            router: Router::new("/users/7?tab=profile"),
            last_navigation: None,
        }
    }
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Router(RouterMessage::Event(event)) => {
            state.last_navigation = Some(event);
            Task::none()
        }
        Message::Router(message) => state.router.handle(message, Message::Router),
    }
}

fn user_settings_guard(
    context: RouteGuardContext,
) -> impl std::future::Future<Output = RouteGuardDecision> + Send {
    async move {
        tokio::time::sleep(Duration::from_millis(250)).await;
        if context.to.param("id") == Some("0") {
            RouteGuardDecision::Redirect("/users/7".to_string())
        } else {
            RouteGuardDecision::Allow
        }
    }
}

fn view(state: &AppState) -> Element<Message> {
    RouterOutlet::new(
        state.router.clone(),
        Routes::new()
            .route("/", |context| Element::new(HomePage::new(context)))
            .nest(
                "/users/:id",
                |context, outlet| Element::new(UserLayout::new(context, outlet)),
                |users| {
                    users
                        .index(|context| Element::new(UserPage::new(context)))
                        .guard_async(user_settings_guard, |guarded| {
                            guarded.route("settings", |context| {
                                Element::new(UserSettingsPage::new(context))
                            })
                        })
                },
            )
            .route("/settings", |context| {
                Element::new(SettingsPage::new(context))
            })
            .fallback("*rest", |context| Element::new(NotFoundPage::new(context))),
    )
    .into()
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(AppState::default, update, view)
}
