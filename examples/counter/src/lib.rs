use arkit::prelude::*;
use arkit::{application, dispatch, NavigationStack, Task};
use ohos_hilogs_sys::{
    LogLevel, LogLevel_LOG_ERROR, LogLevel_LOG_INFO, LogType_LOG_APP, OH_LOG_PrintMsgByLen,
};
use std::rc::Rc;

mod showcase;

const ROUTE_LOG_TAG: &[u8] = b"arkit_route";
#[derive(Clone, Debug, PartialEq, Eq)]
enum AppRoute {
    Home,
    Component { name: String },
}

#[derive(Clone, Debug)]
enum Message {
    Navigate(AppRoute),
    Back,
    SetHomeSearch(String),
    SetActiveTab(usize),
    SetPage(i32),
    SetRadioChoice(String),
    SetSelectChoice(String),
    SetQuery(String),
    SetToggleState(bool),
    SetContextBookmarks(bool),
    SetContextFullUrls(bool),
    SetContextPerson(String),
    SetCheckboxFirst(bool),
    SetCheckboxSecond(bool),
    SetCheckboxCard(bool),
    SetToggleGroupValues(Vec<String>),
}

#[derive(Clone, Debug)]
struct DemoState {
    active_tab: usize,
    page: i32,
    radio_choice: String,
    select_choice: String,
    query: String,
    toggle_state: bool,
    context_bookmarks: bool,
    context_full_urls: bool,
    context_person: String,
    checkbox_first: bool,
    checkbox_second: bool,
    checkbox_card: bool,
    toggle_group_values: Vec<String>,
}

impl Default for DemoState {
    fn default() -> Self {
        Self {
            active_tab: 0,
            page: 1,
            radio_choice: String::from("Comfortable"),
            select_choice: String::new(),
            query: String::new(),
            toggle_state: false,
            context_bookmarks: false,
            context_full_urls: false,
            context_person: String::from("pedro"),
            checkbox_first: true,
            checkbox_second: true,
            checkbox_card: false,
            toggle_group_values: Vec::new(),
        }
    }
}

struct AppState {
    routes: NavigationStack<AppRoute>,
    home_search: String,
    demo: DemoState,
}

impl AppState {
    fn new() -> Self {
        Self {
            routes: NavigationStack::new(AppRoute::Home),
            home_search: String::new(),
            demo: DemoState::default(),
        }
    }
}

fn route_log(level: LogLevel, message: &str) {
    unsafe {
        OH_LOG_PrintMsgByLen(
            LogType_LOG_APP,
            level,
            0x0000,
            ROUTE_LOG_TAG.as_ptr().cast(),
            ROUTE_LOG_TAG.len(),
            message.as_ptr().cast(),
            message.len(),
        );
    }
}

fn route_log_info(message: impl AsRef<str>) {
    route_log(LogLevel_LOG_INFO, message.as_ref());
}

fn route_log_error(message: impl AsRef<str>) {
    route_log(LogLevel_LOG_ERROR, message.as_ref());
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Navigate(route) => {
            route_log_info(format!("route navigate: {:?}", route));
            if matches!(route, AppRoute::Component { .. }) {
                state.demo = DemoState::default();
            }
            state.routes.push(route);
        }
        Message::Back => {
            if !state.routes.back() {
                route_log_error("route back ignored: already at root");
            }
        }
        Message::SetHomeSearch(value) => state.home_search = value,
        Message::SetActiveTab(value) => state.demo.active_tab = value,
        Message::SetPage(value) => state.demo.page = value,
        Message::SetRadioChoice(value) => state.demo.radio_choice = value,
        Message::SetSelectChoice(value) => state.demo.select_choice = value,
        Message::SetQuery(value) => state.demo.query = value,
        Message::SetToggleState(value) => state.demo.toggle_state = value,
        Message::SetContextBookmarks(value) => state.demo.context_bookmarks = value,
        Message::SetContextFullUrls(value) => state.demo.context_full_urls = value,
        Message::SetContextPerson(value) => state.demo.context_person = value,
        Message::SetCheckboxFirst(value) => state.demo.checkbox_first = value,
        Message::SetCheckboxSecond(value) => state.demo.checkbox_second = value,
        Message::SetCheckboxCard(value) => state.demo.checkbox_card = value,
        Message::SetToggleGroupValues(value) => state.demo.toggle_group_values = value,
    }

    Task::none()
}

fn view(state: &AppState) -> Element {
    let on_back: Rc<dyn Fn()> = Rc::new(|| dispatch(Message::Back));
    let on_open: Rc<dyn Fn(String)> =
        Rc::new(|name| dispatch(Message::Navigate(AppRoute::Component { name })));
    let on_home_search: Rc<dyn Fn(String)> = Rc::new(|value| dispatch(Message::SetHomeSearch(value)));

    match state.routes.current().clone() {
        AppRoute::Home => {
            route_log_info("route render: home");
            showcase::catalog_home(state.home_search.clone(), on_home_search, on_open)
        }
        AppRoute::Component { name } => {
            route_log_info(format!("route render: component={name}"));
            showcase::component_page(
                name,
                showcase::DemoContext {
                    active_tab: state.demo.active_tab,
                    on_active_tab: Rc::new(|value| dispatch(Message::SetActiveTab(value))),
                    page: state.demo.page,
                    on_page: Rc::new(|value| dispatch(Message::SetPage(value))),
                    radio_choice: state.demo.radio_choice.clone(),
                    on_radio_choice: Rc::new(|value| dispatch(Message::SetRadioChoice(value))),
                    select_choice: state.demo.select_choice.clone(),
                    on_select_choice: Rc::new(|value| dispatch(Message::SetSelectChoice(value))),
                    query: state.demo.query.clone(),
                    on_query: Rc::new(|value| dispatch(Message::SetQuery(value))),
                    toggle_state: state.demo.toggle_state,
                    on_toggle_state: Rc::new(|value| dispatch(Message::SetToggleState(value))),
                    context_bookmarks: state.demo.context_bookmarks,
                    on_context_bookmarks: Rc::new(|value| dispatch(Message::SetContextBookmarks(value))),
                    context_full_urls: state.demo.context_full_urls,
                    on_context_full_urls: Rc::new(|value| dispatch(Message::SetContextFullUrls(value))),
                    context_person: state.demo.context_person.clone(),
                    on_context_person: Rc::new(|value| dispatch(Message::SetContextPerson(value))),
                    checkbox_first: state.demo.checkbox_first,
                    on_checkbox_first: Rc::new(|value| dispatch(Message::SetCheckboxFirst(value))),
                    checkbox_second: state.demo.checkbox_second,
                    on_checkbox_second: Rc::new(|value| dispatch(Message::SetCheckboxSecond(value))),
                    checkbox_card: state.demo.checkbox_card,
                    on_checkbox_card: Rc::new(|value| dispatch(Message::SetCheckboxCard(value))),
                    toggle_group_values: state.demo.toggle_group_values.clone(),
                    on_toggle_group_values: Rc::new(|value| dispatch(Message::SetToggleGroupValues(value))),
                },
                on_back,
            )
        }
    }
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application("arkit counter", update, view).run_with(|| (AppState::new(), Task::none()))
}
