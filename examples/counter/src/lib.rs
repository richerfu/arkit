use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arkit::ohos_arkui_binding::common::error::ArkUIResult;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use arkit::ohos_arkui_binding::types::curve::Curve;
use arkit::prelude::*;
use arkit::queue_after_mount;
use arkit_animation::{Motion, MotionExt};
use arkit_shadcn as shadcn;
use ohos_hilogs_sys::{
    LogLevel, LogLevel_LOG_ERROR, LogLevel_LOG_INFO, LogType_LOG_APP, OH_LOG_PrintMsgByLen,
};

mod showcase;

const ROUTE_TRANSITION_MS: i32 = 150;
const ROUTE_REPLACE_TRANSITION_MS: i32 = 130;
const ROUTE_ENTER_SCALE: f32 = 0.94;
const ROUTE_EXIT_SCALE: f32 = 0.94;
const ROUTE_ENTER_OPACITY: f32 = 0.0;
const ROUTE_EXIT_OPACITY: f32 = 0.0;
const ROUTE_ALIGN_CENTER: i32 = 4;
const ROUTE_LOG_TAG: &[u8] = b"arkit_route";
const ROUTE_DEBUG_COLOR: u32 = 0xFF64748B;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RouteLayerMode {
    Enter,
    Exit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RouteSnapshot {
    raw: String,
    path: String,
    pattern: String,
    name_param: Option<String>,
    slug: Option<String>,
}

impl RouteSnapshot {
    fn from_route(route: &Route) -> Self {
        let raw = route.raw().to_string();
        let path = route.path().to_string();
        let pattern = route.pattern().to_string();
        let name_param = route
            .param("name")
            .filter(|value| !value.is_empty() && !value.starts_with(':'))
            .map(ToOwned::to_owned);
        let slug = name_param
            .clone()
            .or_else(|| extract_components_slug(&raw))
            .or_else(|| extract_components_slug(&path));

        Self {
            raw,
            path,
            pattern,
            name_param,
            slug,
        }
    }

    fn is_home(&self) -> bool {
        self.path == "/" || self.raw == "/"
    }

    fn key(&self) -> &str {
        if self.raw.is_empty() {
            &self.path
        } else {
            &self.raw
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RouteOverlay {
    id: u64,
    route: RouteSnapshot,
    direction: RouteTransitionDirection,
    mode: RouteLayerMode,
    settles_to: RouteSnapshot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RoutePose {
    scale: f32,
    opacity: f32,
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

fn extract_components_slug(path: &str) -> Option<String> {
    let clean_path = path.split('?').next().unwrap_or(path);
    let mut segments = clean_path.split('/').filter(|segment| !segment.is_empty());
    match (segments.next(), segments.next(), segments.next()) {
        (Some("components"), Some(slug), None) if !slug.is_empty() && !slug.starts_with(':') => {
            Some(slug.to_string())
        }
        _ => None,
    }
}

fn log_route_render(route: &RouteSnapshot) {
    let message = format!(
        "route render raw={} path={} pattern={} name_param={:?} slug={:?}",
        route.raw, route.path, route.pattern, route.name_param, route.slug
    );

    if route.slug.is_some() || route.is_home() {
        route_log_info(message);
    } else {
        route_log_error(message);
    }
}

fn render_route(route: &RouteSnapshot) -> Element {
    log_route_render(route);

    if route.is_home() {
        showcase::catalog_home()
    } else if let Some(name) = route.slug.clone() {
        showcase::component_page(name)
    } else {
        arkit::column(vec![
            showcase::nav_bar("Not Found", true),
            showcase::page_scroll(vec![shadcn::card(vec![
                shadcn::card_title("Route Not Found"),
                shadcn::card_description("无法解析当前路由"),
                arkit::text(format!("raw: {}", route.raw))
                    .font_size(12.0)
                    .style(ArkUINodeAttributeType::FontColor, ROUTE_DEBUG_COLOR)
                    .into(),
                arkit::text(format!("path: {}", route.path))
                    .font_size(12.0)
                    .style(ArkUINodeAttributeType::FontColor, ROUTE_DEBUG_COLOR)
                    .into(),
                arkit::text(format!("pattern: {}", route.pattern))
                    .font_size(12.0)
                    .style(ArkUINodeAttributeType::FontColor, ROUTE_DEBUG_COLOR)
                    .into(),
                arkit::text(format!("name_param: {:?}", route.name_param))
                    .font_size(12.0)
                    .style(ArkUINodeAttributeType::FontColor, ROUTE_DEBUG_COLOR)
                    .into(),
                shadcn::button("返回首页", shadcn::ButtonVariant::Default)
                    .on_click(|| {
                        let _ = reset_route("/");
                    })
                    .into(),
            ])]),
        ])
    }
}

fn route_motion(direction: RouteTransitionDirection) -> Option<Motion> {
    match direction {
        RouteTransitionDirection::Forward => Some(
            Motion::new()
                .duration_ms(ROUTE_TRANSITION_MS)
                .curve(Curve::FastOutSlowIn),
        ),
        RouteTransitionDirection::Backward => Some(
            Motion::new()
                .duration_ms(ROUTE_TRANSITION_MS)
                .curve(Curve::EaseInOut),
        ),
        RouteTransitionDirection::Replace => Some(
            Motion::new()
                .duration_ms(ROUTE_REPLACE_TRANSITION_MS)
                .curve(Curve::FastOutSlowIn),
        ),
        RouteTransitionDirection::None => None,
    }
}

fn route_rest_pose() -> RoutePose {
    RoutePose {
        scale: 1.0,
        opacity: 1.0,
    }
}

fn route_enter_pose(_direction: RouteTransitionDirection) -> RoutePose {
    RoutePose {
        scale: ROUTE_ENTER_SCALE,
        opacity: ROUTE_ENTER_OPACITY,
    }
}

fn route_exit_pose(_direction: RouteTransitionDirection) -> RoutePose {
    RoutePose {
        scale: ROUTE_EXIT_SCALE,
        opacity: ROUTE_EXIT_OPACITY,
    }
}

fn apply_route_pose(node: &mut ArkUINode, pose: RoutePose) -> ArkUIResult<()> {
    node.set_transform_center(vec![0.0, 0.0, 0.0, 0.5, 0.5, 0.0])?;
    node.set_scale(vec![pose.scale, pose.scale])?;
    node.opacity(pose.opacity)
}

fn route_scene(route: &RouteSnapshot) -> Element {
    let route_key = format!("route-scene:{}", route.key());
    let route = route.clone();
    arkit::keyed_scope(route_key, move || render_route(&route))
}

fn route_surface(route: &RouteSnapshot, key: impl Into<String>) -> Element {
    arkit::column_component()
        .key(key)
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![route_scene(route)])
        .into()
}

fn route_layer(
    route: &RouteSnapshot,
    key: impl Into<String>,
    z_index: i32,
    enabled: bool,
) -> Element {
    let surface_key = format!("route-surface:{}", route.key());
    arkit::stack_component()
        .key(key)
        .percent_width(1.0)
        .percent_height(1.0)
        .style(ArkUINodeAttributeType::Alignment, ROUTE_ALIGN_CENTER)
        .style(ArkUINodeAttributeType::ZIndex, z_index)
        .style(ArkUINodeAttributeType::Enabled, enabled)
        .children(vec![route_surface(route, surface_key)])
        .into()
}

fn animated_route_layer(
    overlay: RouteOverlay,
    transition_id: Signal<u64>,
    base_route: Signal<RouteSnapshot>,
    overlay_state: Signal<Option<RouteOverlay>>,
) -> Element {
    let layer_key = format!("route-overlay:{}:{}", overlay.id, overlay.route.key());
    let surface_key = format!(
        "route-overlay-surface:{}:{}",
        overlay.id,
        overlay.route.key()
    );
    let mut layer = arkit::stack_component()
        .key(layer_key)
        .percent_width(1.0)
        .percent_height(1.0)
        .style(ArkUINodeAttributeType::Alignment, ROUTE_ALIGN_CENTER)
        .style(ArkUINodeAttributeType::ZIndex, 1_i32)
        .style(ArkUINodeAttributeType::Enabled, false)
        .children(vec![route_surface(&overlay.route, surface_key)]);

    if let Some(motion) = route_motion(overlay.direction) {
        let initial_frame = match overlay.mode {
            RouteLayerMode::Enter => route_enter_pose(overlay.direction),
            RouteLayerMode::Exit => route_rest_pose(),
        };
        let target_frame = match overlay.mode {
            RouteLayerMode::Enter => route_rest_pose(),
            RouteLayerMode::Exit => route_exit_pose(overlay.direction),
        };
        let expected_id = overlay.id;
        let settles_to = overlay.settles_to.clone();
        layer = layer.with_mount_motion_finish(
            motion,
            move |node| apply_route_pose(node, initial_frame),
            move |node| apply_route_pose(node, target_frame),
            move || {
                if transition_id.get() != expected_id {
                    return;
                }
                base_route.set(settles_to.clone());
                overlay_state.set(None);
            },
        );
    }

    layer.into()
}

#[entry]
fn app() -> Element {
    let active_router = use_router();
    let route = RouteSnapshot::from_route(&active_router.current_route());
    let animations_ready = create_signal(false);
    let base_route = create_signal(route.clone());
    let overlay_route = create_signal(None::<RouteOverlay>);
    let transition_id = create_signal(0_u64);
    let subscription_id = Rc::new(RefCell::new(None::<usize>));
    let previous_route = Rc::new(RefCell::new(route.clone()));
    let previous_stack_len = Rc::new(Cell::new(active_router.stack_len()));

    // Mount: register routes and subscribe to router changes.
    {
        let router = active_router.clone();
        let animations_ready = animations_ready.clone();
        let base_route = base_route.clone();
        let overlay_route = overlay_route.clone();
        let transition_id = transition_id.clone();
        let subscription_id = subscription_id.clone();
        let previous_route = previous_route.clone();
        let previous_stack_len = previous_stack_len.clone();

        let _ = register_route("/");
        let _ = register_named_route("components", "/components/:name");
        let id = router.subscribe({
            let router = router.clone();
            let animations_ready = animations_ready.clone();
            let base_route = base_route.clone();
            let overlay_route = overlay_route.clone();
            let transition_id = transition_id.clone();
            let previous_route = previous_route.clone();
            let previous_stack_len = previous_stack_len.clone();
            move |next_route| {
                let next_route = RouteSnapshot::from_route(&next_route);
                route_log_info(format!(
                    "route change raw={} path={} pattern={} name_param={:?} slug={:?} stack_len={}",
                    next_route.raw,
                    next_route.path,
                    next_route.pattern,
                    next_route.name_param,
                    next_route.slug,
                    router.stack_len()
                ));

                let next_stack_len = router.stack_len();
                let prev_stack_len = previous_stack_len.get();
                let prev_route_snapshot = previous_route.borrow().clone();
                let direction = if next_stack_len > prev_stack_len {
                    RouteTransitionDirection::Forward
                } else if next_stack_len < prev_stack_len {
                    RouteTransitionDirection::Backward
                } else if next_route.raw != prev_route_snapshot.raw {
                    RouteTransitionDirection::Replace
                } else {
                    RouteTransitionDirection::None
                };

                previous_stack_len.set(next_stack_len);
                *previous_route.borrow_mut() = next_route.clone();

                if !animations_ready.get()
                    || matches!(direction, RouteTransitionDirection::None)
                {
                    overlay_route.set(None);
                    base_route.set(next_route);
                    return;
                }

                let next_transition_id = transition_id.get().wrapping_add(1);
                transition_id.set(next_transition_id);

                match direction {
                    RouteTransitionDirection::Forward
                    | RouteTransitionDirection::Replace => {
                        base_route.set(prev_route_snapshot.clone());
                        overlay_route.set(Some(RouteOverlay {
                            id: next_transition_id,
                            route: next_route.clone(),
                            direction,
                            mode: RouteLayerMode::Enter,
                            settles_to: next_route,
                        }));
                    }
                    RouteTransitionDirection::Backward => {
                        base_route.set(next_route.clone());
                        overlay_route.set(Some(RouteOverlay {
                            id: next_transition_id,
                            route: prev_route_snapshot,
                            direction,
                            mode: RouteLayerMode::Exit,
                            settles_to: next_route,
                        }));
                    }
                    RouteTransitionDirection::None => {
                        overlay_route.set(None);
                        base_route.set(next_route);
                    }
                }
            }
        });
        *subscription_id.borrow_mut() = Some(id);
        let _ = reset_route("/");
        queue_after_mount(move || {
            animations_ready.set(true);
        });
    }

    // Cleanup: unsubscribe from router on disposal.
    on_cleanup({
        let router = active_router.clone();
        let subscription_id = subscription_id.clone();
        move || {
            if let Some(id) = subscription_id.borrow_mut().take() {
                router.unsubscribe(id);
            }
        }
    });

    let base = base_route.get();
    let is_transitioning = overlay_route.get().is_some();
    let mut layers = vec![route_layer(
        &base,
        format!("route-base:{}", base.key()),
        0,
        !is_transitioning,
    )];
    if let Some(overlay) = overlay_route.get() {
        layers.push(animated_route_layer(
            overlay,
            transition_id.clone(),
            base_route.clone(),
            overlay_route.clone(),
        ));
    }

    arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .style(ArkUINodeAttributeType::Alignment, ROUTE_ALIGN_CENTER)
        .children(layers)
        .into()
}
