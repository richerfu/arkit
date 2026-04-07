use arkit::prelude::*;
use arkit::{animated_router_view, RouteTransitionConfig};
use arkit_shadcn as shadcn;
use ohos_hilogs_sys::{
    LogLevel, LogLevel_LOG_ERROR, LogLevel_LOG_INFO, LogType_LOG_APP, OH_LOG_PrintMsgByLen,
};

mod showcase;

const ROUTE_LOG_TAG: &[u8] = b"arkit_route";
const ROUTE_DEBUG_COLOR: u32 = 0xFF64748B;

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

fn is_home(route: &Route) -> bool {
    route.path() == "/" || route.raw() == "/"
}

fn get_slug(route: &Route) -> Option<String> {
    route
        .param("name")
        .filter(|value| !value.is_empty() && !value.starts_with(':'))
        .map(ToOwned::to_owned)
        .or_else(|| extract_components_slug(route.raw()))
        .or_else(|| extract_components_slug(route.path()))
}

fn render_route(route: &Route) -> Element {
    if is_home(route) {
        route_log_info(format!("route render: home"));
        showcase::catalog_home()
    } else if let Some(name) = get_slug(route) {
        route_log_info(format!("route render: component={name}"));
        showcase::component_page(name)
    } else {
        route_log_error(format!(
            "route render: unknown raw={} path={} pattern={}",
            route.raw(),
            route.path(),
            route.pattern()
        ));
        arkit::column(vec![
            showcase::nav_bar("Not Found", true),
            showcase::page_scroll(vec![shadcn::card(vec![
                shadcn::card_title("Route Not Found"),
                shadcn::card_description("无法解析当前路由"),
                arkit::text(format!("raw: {}", route.raw()))
                    .font_size(12.0)
                    .style(ArkUINodeAttributeType::FontColor, ROUTE_DEBUG_COLOR)
                    .into(),
                arkit::text(format!("path: {}", route.path()))
                    .font_size(12.0)
                    .style(ArkUINodeAttributeType::FontColor, ROUTE_DEBUG_COLOR)
                    .into(),
                arkit::text(format!("pattern: {}", route.pattern()))
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

#[entry]
fn app() -> Element {
    let _ = register_route("/");
    let _ = register_named_route("components", "/components/:name");
    let _ = reset_route("/");

    animated_router_view(RouteTransitionConfig::default(), |route| {
        render_route(route)
    })
}
