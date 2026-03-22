use arkit::prelude::*;
use arkit_shadcn as shadcn;

mod showcase;

#[entry]
fn app() -> Element {
    use_component_lifecycle(
        || {
            let _ = register_route("/");
            let _ = register_named_route("components", "/components/:name");
            let _ = reset_route("/");
        },
        || {},
    );

    let route = use_route();
    let route_key = format!("route:{}", route.path());

    let screen = if route.path() == "/" {
        showcase::catalog_home()
    } else if let Some(name) = route.param("name") {
        showcase::component_page(name.to_string())
    } else {
        arkit::column(vec![
            showcase::nav_bar("Not Found", true),
            showcase::page_scroll(vec![shadcn::card(vec![
                shadcn::card_title("Route Not Found"),
                shadcn::card_description("无法解析当前路由"),
                shadcn::button("返回首页", shadcn::ButtonVariant::Default)
                    .on_click(|| {
                        let _ = reset_route("/");
                    })
                    .into(),
            ])]),
        ])
    };

    arkit::column(vec![arkit::column_component()
        .key(route_key)
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![screen])
        .into()])
}
