use arkit::StructuredRoute;

#[derive(Debug, Clone, PartialEq, Eq, StructuredRoute)]
#[route(path = "/", name = "home")]
pub(crate) struct HomeRoute;

#[derive(Debug, Clone, PartialEq, Eq, StructuredRoute)]
#[route(path = "/users/:id", name = "user")]
pub(crate) struct UserRoute {
    pub(crate) id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, StructuredRoute)]
#[route(path = "/settings", name = "settings")]
pub(crate) struct SettingsRoute;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserNavigationState {
    pub(crate) source: String,
    pub(crate) scroll_offset: u32,
}
