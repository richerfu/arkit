use arkit::entry;
use arkit::ohos_arkui_binding::common::error::ArkUIResult;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use arkit::ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use arkit::ohos_arkui_binding::types::curve::Curve;
use arkit::router::{
    Navigation, NavigationEvent, Route as RouterRoute, RouteDefinition, RouteTransitionDirection,
    Router, RouterMessage, RouterNavigationExt,
};
use arkit::{application, Element as ArkElement, Task};
use arkit_animation::{Motion, MotionExt};
use arkit_shadcn::theme::{ColorTokens, RadiusTokens, ThemeMode, ThemePreset};
use std::cell::Cell;
use std::rc::Rc;

mod showcase;

pub(crate) mod prelude {
    pub(crate) type Element = arkit::Element<crate::Message>;
    pub(crate) use arkit::prelude::*;
}

use showcase::{CatalogHome, ComponentPage, DemoContext};

const ROUTE_TRANSITION_DISTANCE: f32 = 28.0;
const ROUTE_TRANSITION_DURATION_MS: i32 = 180;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Route {
    Home,
    Component { slug: String },
}

impl Route {
    fn key(&self) -> String {
        match self {
            Route::Home => "/".to_string(),
            Route::Component { slug } => format!("/components/{slug}"),
        }
    }

    fn from_router_route(route: &RouterRoute) -> Option<Self> {
        if route.pattern() == "/" {
            return Some(Route::Home);
        }

        if route.pattern() == "/components/:slug" {
            return route.param("slug").map(|slug| Route::Component {
                slug: slug.to_string(),
            });
        }

        None
    }

    fn router_message(&self) -> RouterMessage {
        match self {
            Route::Home => RouterMessage::push("/"),
            Route::Component { slug } => RouterMessage::push(format!("/components/{slug}")),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Router(RouterMessage),
    ButtonPreviewPressed(String),
    SetHomeSearch(String),
    SetPage(i32),
    SetRadioChoice(String),
    SetSelectChoice(String),
    SetQuery(String),
    SetToggleState(bool),
    SetContextMenuOpen(bool),
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
    SetThemeMenuOpen(bool),
    SetThemeMode(ThemeMode),
    SetThemePreset(ThemePreset),
    SetCustomTheme(bool),
}

pub(crate) struct ShowcaseState {
    router: Router,
    route: Route,
    route_transition_direction: Rc<Cell<RouteTransitionDirection>>,
    home_search: String,
    button_preview_feedback: Option<String>,
    page: i32,
    radio_choice: String,
    select_choice: String,
    query: String,
    toggle_state: bool,
    context_menu_open: bool,
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
    pub(crate) theme_menu_open: bool,
    pub(crate) theme_mode: ThemeMode,
    pub(crate) theme_preset: ThemePreset,
    pub(crate) custom_theme: bool,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        let router = Router::new("/");
        router
            .register_definition(RouteDefinition::new("/").expect("home route"))
            .expect("register home route");
        router
            .register_definition(
                RouteDefinition::new("/components/:slug").expect("component route"),
            )
            .expect("register component route");

        Self {
            router,
            route: Route::Home,
            route_transition_direction: Rc::new(Cell::new(RouteTransitionDirection::None)),
            home_search: String::new(),
            button_preview_feedback: None,
            page: 1,
            radio_choice: String::from("Default"),
            select_choice: String::from("Apple"),
            query: String::new(),
            toggle_state: false,
            context_menu_open: false,
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
            theme_menu_open: false,
            theme_mode: ThemeMode::Light,
            theme_preset: ThemePreset::Zinc,
            custom_theme: false,
        }
    }
}

impl ShowcaseState {
    fn new() -> Self {
        Self::default()
    }

    fn demo_context(&self) -> DemoContext {
        DemoContext {
            page: self.page,
            button_preview_feedback: self.button_preview_feedback.clone(),
            radio_choice: self.radio_choice.clone(),
            select_choice: self.select_choice.clone(),
            query: self.query.clone(),
            toggle_state: self.toggle_state,
            context_menu_open: self.context_menu_open,
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
            theme_menu_open: self.theme_menu_open,
            theme_mode: self.theme_mode,
            theme_preset: self.theme_preset,
            custom_theme: self.custom_theme,
        }
    }

    fn reset_component_demo_state(&mut self) {
        self.page = 1;
        self.button_preview_feedback = None;
        self.radio_choice = String::from("Default");
        self.select_choice = String::from("Apple");
        self.query.clear();
        self.toggle_state = false;
        self.context_menu_open = false;
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

    fn theme(&self) -> arkit_shadcn::theme::Theme {
        if self.custom_theme {
            let colors = custom_theme_colors(self.theme_mode);
            return arkit_shadcn::theme::Theme::custom(colors)
                .with_mode(self.theme_mode)
                .with_radius(RadiusTokens::from_base(10.0));
        }

        arkit_shadcn::theme::Theme::preset(self.theme_preset, self.theme_mode)
    }
}

fn custom_theme_colors(mode: ThemeMode) -> ColorTokens {
    let mut colors = arkit_shadcn::theme::Theme::preset(ThemePreset::Mist, mode).colors;

    match mode {
        ThemeMode::Light => {
            colors.primary = 0xFF0F766E;
            colors.primary_foreground = 0xFFF0FDFA;
            colors.primary_track = arkit_shadcn::theme::with_alpha(colors.primary, 0x33);
            colors.ring = 0xFF0F766E;
            colors.chart_1 = 0xFF0F766E;
            colors.chart_2 = 0xFF2563EB;
            colors.chart_3 = 0xFF7C3AED;
            colors.sidebar_primary = colors.primary;
            colors.sidebar_primary_foreground = colors.primary_foreground;
        }
        ThemeMode::Dark => {
            colors.primary = 0xFF5EEAD4;
            colors.primary_foreground = 0xFF042F2E;
            colors.primary_track = arkit_shadcn::theme::with_alpha(colors.primary, 0x33);
            colors.ring = 0xFF5EEAD4;
            colors.chart_1 = 0xFF5EEAD4;
            colors.chart_2 = 0xFF60A5FA;
            colors.chart_3 = 0xFFC084FC;
            colors.sidebar_primary = colors.primary;
            colors.sidebar_primary_foreground = colors.primary_foreground;
        }
    }

    colors
}

fn update(state: &mut ShowcaseState, message: Message) -> Task<Message> {
    match message {
        Message::Router(RouterMessage::Event(event)) => {
            return handle_navigation_event(state, event);
        }
        Message::Router(message) => {
            return state.router.handle(message, Message::Router);
        }
        Message::ButtonPreviewPressed(label) => {
            state.button_preview_feedback =
                Some(format!("Last action: button preview pressed: {label}"));
            ohos_hilog_binding::info(format!("button preview pressed: {label}"));
        }
        Message::SetHomeSearch(value) => state.home_search = value,
        Message::SetPage(value) => state.page = value.max(1),
        Message::SetRadioChoice(value) => state.radio_choice = value,
        Message::SetSelectChoice(value) => state.select_choice = value,
        Message::SetQuery(value) => state.query = value,
        Message::SetToggleState(value) => state.toggle_state = value,
        Message::SetContextMenuOpen(value) => state.context_menu_open = value,
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
        Message::SetThemeMenuOpen(value) => state.theme_menu_open = value,
        Message::SetThemeMode(value) => {
            state.theme_mode = value;
            state.theme_menu_open = false;
        }
        Message::SetThemePreset(value) => {
            state.theme_preset = value;
            state.custom_theme = false;
            state.theme_menu_open = false;
        }
        Message::SetCustomTheme(value) => {
            state.custom_theme = value;
            state.theme_menu_open = false;
        }
    }

    Task::none()
}

fn handle_navigation_event(state: &mut ShowcaseState, event: NavigationEvent) -> Task<Message> {
    match event.result {
        Ok(resolved) => {
            let route = Route::from_router_route(&resolved).unwrap_or(Route::Home);
            if route != state.route {
                state.reset_component_demo_state();
            }

            state
                .route_transition_direction
                .set(navigation_direction(&event.navigation));
            state.route = route;
        }
        Err(error) => {
            if matches!(event.navigation, Navigation::Back) && state.route != Route::Home {
                state.reset_component_demo_state();
                return state
                    .router
                    .handle(RouterMessage::replace("/"), Message::Router);
            }

            ohos_hilog_binding::error(format!("navigation failed: {error}"));
        }
    }

    Task::none()
}

fn navigation_direction(navigation: &Navigation) -> RouteTransitionDirection {
    match navigation {
        Navigation::Push(_) => RouteTransitionDirection::Forward,
        Navigation::Back => RouteTransitionDirection::Backward,
        Navigation::Replace(_) | Navigation::Reset(_) => RouteTransitionDirection::Replace,
    }
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
        .persistent_state_key(route.key())
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
    arkit_shadcn::theme::with_theme(state.theme(), || {
        let content = match &state.route {
            Route::Home => ArkElement::new(CatalogHome::new(state)),
            Route::Component { slug } => {
                ArkElement::new(ComponentPage::new(slug.clone(), state.demo_context()))
            }
        };

        route_page(
            &state.route,
            state.route_transition_direction.clone(),
            content,
        )
    })
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(ShowcaseState::new, update, view)
}
