//! Demo page — dispatches `/demo/:slug` to the matching example page.
//!
//! No outer scroll wrapper here: pages that own scrolling (shadcn_showcase,
//! router, complex_cases) provide their own viewport, and simple pages render
//! fine inside the router's bare page frame.

use arkit::prelude::*;
use arkit::router::RouteTransition;

use crate::registry;
use crate::Route;

// Named `Demo` to match the `Route::Demo` variant: the `Routable` derive
// renders each variant through a component of the same name.
#[component]
pub fn Demo(slug: String) -> Element {
    let known = registry::find_demo(&slug).is_some();

    rsx! {
        RouteTransition::<Route> {
            if known {
                match slug.as_str() {
                    "counter" => rsx! { counter::CounterPage {} },
                    "async_task" => rsx! { async_task::AsyncTaskPage {} },
                    "animation" => rsx! { animation::AnimationPage {} },
                    "canvas" => rsx! { canvas_example::CanvasPage {} },
                    "chart" => rsx! { chart_example::ChartPage {} },
                    "camera" => rsx! { camera_example::CameraPage {} },
                    "barcode" => rsx! { barcode_example::BarcodePage {} },
                    "complex_cases" => rsx! { complex_cases::ComplexCasesPage {} },
                    "i18n" => rsx! { i18n_example::I18nPage {} },
                    "lottie" => rsx! { lottie_example::LottiePage {} },
                    "video" => rsx! { video_example::VideoPage {} },
                    "router" => rsx! { router_example::RouterPage {} },
                    "shadcn_showcase" => rsx! { shadcn_showcase::ShadcnShowcasePage {} },
                    "terminal" => rsx! { terminal_example::TerminalPage {} },
                    "webview" => rsx! { webview_example::WebviewPage {} },
                    _ => rsx! { UnknownDemo { slug } },
                }
            } else {
                UnknownDemo { slug }
            }
        }
    }
}

#[component]
fn UnknownDemo(slug: String) -> Element {
    rsx! {
        column {
            width: "100%",
            height: "100%",
            align_items: "center",
            justify_content: "center",
            text {
                font_size: 20.0,
                font_weight: 600,
                "未知示例"
            }
            text {
                margin_top: 8.0,
                font_size: 14.0,
                font_color: "#ff71717a",
                "demo slug: {slug}"
            }
        }
    }
}
