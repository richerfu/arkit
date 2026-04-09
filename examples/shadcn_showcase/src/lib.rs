use arkit::entry;
use arkit::{application, Element as ArkElement, Task};

mod showcase;

pub(crate) mod prelude {
    pub(crate) type Element = arkit::Element<crate::Message>;
    pub(crate) use arkit::prelude::*;
}

use showcase::{catalog_home, component_page, DemoContext};

#[derive(Debug, Clone)]
enum Route {
    Home,
    Component { slug: String },
}

#[derive(Debug, Clone)]
enum Message {
    Navigate(Route),
    Back,
    SetHomeSearch(String),
    SetActiveTab(usize),
    SetPage(i32),
    SetRadioChoice(String),
    SetSelectChoice(String),
    SetQuery(String),
    SetToggleState(bool),
    SetContextMenuOpen(bool),
    SetDropdownMenuOpen(bool),
    SetPopoverOpen(bool),
    SetTooltipOpen(bool),
    SetSelectOpen(bool),
    SetAccordionSingleOpen(Option<String>),
    SetContextBookmarks(bool),
    SetContextFullUrls(bool),
    SetContextPerson(String),
    SetCheckboxFirst(bool),
    SetCheckboxSecond(bool),
    SetCheckboxCard(bool),
    SetToggleGroupValues(Vec<String>),
    SetMenubarActive(Option<usize>),
}

#[derive(Debug, Clone)]
struct ShowcaseState {
    route: Route,
    home_search: String,
    active_tab: usize,
    page: i32,
    radio_choice: String,
    select_choice: String,
    query: String,
    toggle_state: bool,
    context_menu_open: bool,
    dropdown_menu_open: bool,
    popover_open: bool,
    tooltip_open: bool,
    select_open: bool,
    accordion_single_open: Option<String>,
    context_bookmarks: bool,
    context_full_urls: bool,
    context_person: String,
    checkbox_first: bool,
    checkbox_second: bool,
    checkbox_card: bool,
    toggle_group_values: Vec<String>,
    menubar_active: Option<usize>,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        Self {
            route: Route::Home,
            home_search: String::new(),
            active_tab: 0,
            page: 1,
            radio_choice: String::from("Default"),
            select_choice: String::from("Apple"),
            query: String::new(),
            toggle_state: false,
            context_menu_open: false,
            dropdown_menu_open: false,
            popover_open: false,
            tooltip_open: false,
            select_open: false,
            accordion_single_open: Some(String::from("item-1")),
            context_bookmarks: true,
            context_full_urls: false,
            context_person: String::from("pedro"),
            checkbox_first: false,
            checkbox_second: false,
            checkbox_card: false,
            toggle_group_values: vec![String::from("bold")],
            menubar_active: None,
        }
    }
}

impl ShowcaseState {
    fn new() -> Self {
        Self::default()
    }

    fn demo_context(&self) -> DemoContext {
        DemoContext {
            active_tab: self.active_tab,
            page: self.page,
            radio_choice: self.radio_choice.clone(),
            select_choice: self.select_choice.clone(),
            query: self.query.clone(),
            toggle_state: self.toggle_state,
            context_menu_open: self.context_menu_open,
            dropdown_menu_open: self.dropdown_menu_open,
            popover_open: self.popover_open,
            tooltip_open: self.tooltip_open,
            select_open: self.select_open,
            accordion_single_open: self.accordion_single_open.clone(),
            context_bookmarks: self.context_bookmarks,
            context_full_urls: self.context_full_urls,
            context_person: self.context_person.clone(),
            checkbox_first: self.checkbox_first,
            checkbox_second: self.checkbox_second,
            checkbox_card: self.checkbox_card,
            toggle_group_values: self.toggle_group_values.clone(),
            menubar_active: self.menubar_active,
        }
    }

    fn reset_component_demo_state(&mut self) {
        self.active_tab = 0;
        self.page = 1;
        self.radio_choice = String::from("Default");
        self.select_choice = String::from("Apple");
        self.query.clear();
        self.toggle_state = false;
        self.context_menu_open = false;
        self.dropdown_menu_open = false;
        self.popover_open = false;
        self.tooltip_open = false;
        self.select_open = false;
        self.accordion_single_open = Some(String::from("item-1"));
        self.context_bookmarks = true;
        self.context_full_urls = false;
        self.context_person = String::from("pedro");
        self.checkbox_first = false;
        self.checkbox_second = false;
        self.checkbox_card = false;
        self.toggle_group_values = vec![String::from("bold")];
        self.menubar_active = None;
    }
}

fn update(state: &mut ShowcaseState, message: Message) -> Task<Message> {
    match message {
        Message::Navigate(route) => {
            state.reset_component_demo_state();
            state.route = route;
        }
        Message::Back => {
            state.reset_component_demo_state();
            state.route = Route::Home;
        }
        Message::SetHomeSearch(value) => state.home_search = value,
        Message::SetActiveTab(value) => state.active_tab = value,
        Message::SetPage(value) => state.page = value.max(1),
        Message::SetRadioChoice(value) => state.radio_choice = value,
        Message::SetSelectChoice(value) => state.select_choice = value,
        Message::SetQuery(value) => state.query = value,
        Message::SetToggleState(value) => state.toggle_state = value,
        Message::SetContextMenuOpen(value) => state.context_menu_open = value,
        Message::SetDropdownMenuOpen(value) => state.dropdown_menu_open = value,
        Message::SetPopoverOpen(value) => state.popover_open = value,
        Message::SetTooltipOpen(value) => state.tooltip_open = value,
        Message::SetSelectOpen(value) => state.select_open = value,
        Message::SetAccordionSingleOpen(value) => state.accordion_single_open = value,
        Message::SetContextBookmarks(value) => state.context_bookmarks = value,
        Message::SetContextFullUrls(value) => state.context_full_urls = value,
        Message::SetContextPerson(value) => state.context_person = value,
        Message::SetCheckboxFirst(value) => state.checkbox_first = value,
        Message::SetCheckboxSecond(value) => state.checkbox_second = value,
        Message::SetCheckboxCard(value) => state.checkbox_card = value,
        Message::SetToggleGroupValues(value) => state.toggle_group_values = value,
        Message::SetMenubarActive(value) => state.menubar_active = value,
    }

    Task::none()
}

fn view(state: &ShowcaseState) -> ArkElement<Message> {
    match &state.route {
        Route::Home => catalog_home(state.home_search.clone()),
        Route::Component { slug } => component_page(slug.clone(), state.demo_context()),
    }
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(ShowcaseState::new, update, view)
}
