use arkit::entry;
use arkit::ohos_arkui_binding::common::error::ArkUIResult;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use arkit::ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use arkit::ohos_arkui_binding::types::curve::Curve;
use arkit::{application, Element as ArkElement, Task};
use arkit_animation::{Motion, MotionExt};
use arkit_router::{
    Route as RouterRoute, RouteDefinition, RouteTransitionDirection, Router, StructuredRoute,
};
use std::cell::Cell;
use std::rc::Rc;

mod showcase;

pub(crate) mod prelude {
    pub(crate) type Element = arkit::Element<crate::Message>;
    pub(crate) use arkit::prelude::*;
    pub(crate) use arkit_shadcn::ButtonStyleExt;
}

use showcase::{catalog_home, component_page, DemoContext};

const ROUTE_TRANSITION_DISTANCE: f32 = 28.0;
const ROUTE_TRANSITION_DURATION_MS: i32 = 180;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Route {
    Home,
    Component { slug: String },
}

impl Route {
    fn key(&self) -> String {
        self.path()
    }

    fn from_router_route(route: &RouterRoute) -> Option<Self> {
        <Self as StructuredRoute>::from_route(route)
    }
}

impl StructuredRoute for Route {
    fn definitions() -> Vec<RouteDefinition> {
        vec![
            RouteDefinition::named("home", "/").expect("home route definition"),
            RouteDefinition::named("component", "/components/:slug")
                .expect("component route definition"),
        ]
    }

    fn path(&self) -> String {
        match self {
            Route::Home => "/".to_string(),
            Route::Component { slug } => format!("/components/{slug}"),
        }
    }

    fn from_route(route: &RouterRoute) -> Option<Self> {
        match route.name()? {
            "home" => Some(Route::Home),
            "component" => Some(Route::Component {
                slug: route.param("slug")?.to_string(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Navigate(Route),
    Back,
    ButtonPreviewPressed(String),
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

struct ShowcaseState {
    router: Router,
    route: Route,
    route_transition_direction: Rc<Cell<RouteTransitionDirection>>,
    home_search: String,
    button_preview_feedback: Option<String>,
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
        let router = Router::new("/");
        router
            .register_structured::<Route>()
            .expect("register showcase routes");

        Self {
            router,
            route: Route::Home,
            route_transition_direction: Rc::new(Cell::new(RouteTransitionDirection::None)),
            home_search: String::new(),
            button_preview_feedback: None,
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
            button_preview_feedback: self.button_preview_feedback.clone(),
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
        self.button_preview_feedback = None;
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
            if route != state.route {
                state.reset_component_demo_state();
                match state.router.push_structured(route.clone()) {
                    Ok(resolved) => {
                        state
                            .route_transition_direction
                            .set(RouteTransitionDirection::Forward);
                        state.route = Route::from_router_route(&resolved).unwrap_or(route);
                    }
                    Err(error) => {
                        ohos_hilog_binding::error(format!("navigation failed: {error}"));
                    }
                }
            }
        }
        Message::Back => {
            if state.router.can_go_back() {
                state.reset_component_demo_state();
                if state.router.back() {
                    state
                        .route_transition_direction
                        .set(RouteTransitionDirection::Backward);
                    state.route = state
                        .router
                        .current_structured::<Route>()
                        .unwrap_or(Route::Home);
                }
            } else if state.route != Route::Home {
                state.reset_component_demo_state();
                match state.router.replace_structured(Route::Home) {
                    Ok(resolved) => {
                        state
                            .route_transition_direction
                            .set(RouteTransitionDirection::Replace);
                        state.route = Route::from_router_route(&resolved).unwrap_or(Route::Home);
                    }
                    Err(error) => {
                        ohos_hilog_binding::error(format!("navigation failed: {error}"));
                    }
                }
            }
        }
        Message::ButtonPreviewPressed(label) => {
            state.button_preview_feedback =
                Some(format!("Last action: button preview pressed: {label}"));
            ohos_hilog_binding::info(format!("button preview pressed: {label}"));
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

fn route_motion() -> Motion {
    Motion::new()
        .duration_ms(ROUTE_TRANSITION_DURATION_MS)
        .curve(Curve::EaseOut)
}

fn enter_offset(direction: RouteTransitionDirection) -> f32 {
    match direction {
        RouteTransitionDirection::Forward => ROUTE_TRANSITION_DISTANCE,
        RouteTransitionDirection::Backward => -ROUTE_TRANSITION_DISTANCE,
        RouteTransitionDirection::None | RouteTransitionDirection::Replace => 0.0,
    }
}

fn exit_offset(direction: RouteTransitionDirection) -> f32 {
    match direction {
        RouteTransitionDirection::Forward => -ROUTE_TRANSITION_DISTANCE,
        RouteTransitionDirection::Backward => ROUTE_TRANSITION_DISTANCE,
        RouteTransitionDirection::None | RouteTransitionDirection::Replace => 0.0,
    }
}

fn apply_route_frame(node: &mut ArkUINode, offset_x: f32, opacity: f32) -> ArkUIResult<()> {
    node.set_attribute(ArkUINodeAttributeType::Opacity, opacity.into())?;
    node.set_attribute(
        ArkUINodeAttributeType::Translate,
        vec![offset_x, 0.0, 0.0].into(),
    )
}

fn route_page(
    route: &Route,
    direction: Rc<Cell<RouteTransitionDirection>>,
    content: ArkElement<Message>,
) -> ArkElement<Message> {
    let enter_direction = direction.clone();
    let exit_direction = direction;

    arkit::stack_component()
        .key(route.key())
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![content])
        .with_enter_exit_motion(
            route_motion(),
            move |node| apply_route_frame(node, enter_offset(enter_direction.get()), 0.0),
            move |node| apply_route_frame(node, 0.0, 1.0),
            route_motion(),
            move |node| apply_route_frame(node, exit_offset(exit_direction.get()), 0.0),
        )
        .into()
}

fn view(state: &ShowcaseState) -> ArkElement<Message> {
    let content = match &state.route {
        Route::Home => catalog_home(state.home_search.clone()),
        Route::Component { slug } => component_page(slug.clone(), state.demo_context()),
    };

    route_page(
        &state.route,
        state.route_transition_direction.clone(),
        content,
    )
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(ShowcaseState::new, update, view)
}
