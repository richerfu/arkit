use arkit::entry;
use arkit::router::{RouteStateKey, Router, RouterOutlet};
use arkit::{application, Element, Task};

mod components;
mod pages;
mod routes;

use pages::{HomePage, NotFoundPage, SettingsPage, UserPage};
use routes::{HomeRoute, SettingsRoute, UserRoute};

#[derive(Debug, Clone)]
enum Message {}

#[derive(Clone)]
struct AppState {
    router: Router,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            router: Router::new("/users/7?tab=profile"),
        }
    }
}

fn update(_state: &mut AppState, message: Message) -> Task<Message> {
    match message {}
}

fn view(state: &AppState) -> Element<Message> {
    RouterOutlet::new(state.router.clone())
        .route::<HomeRoute, HomePage>()
        .route_keyed::<UserRoute, UserPage>(RouteStateKey::Raw)
        .route::<SettingsRoute, SettingsPage>()
        .fallback::<NotFoundPage>()
        .into()
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(AppState::default, update, view)
}
