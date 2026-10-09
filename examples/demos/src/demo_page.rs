//! Demo page — dispatches `/demo/:slug` to the matching example page.
//!
//! No outer scroll wrapper here: pages that own scrolling (shadcn_showcase,
//! router, complex_cases) provide their own viewport, and simple pages render
//! fine inside the router's bare page frame.

use arkit::prelude::*;
use arkit::router::{use_navigator, RouteTransition};
use arkit::shadcn::icon::icon_placeholder;

use crate::registry;
use crate::Route;

mod accessibility;
mod chart_contracts;
mod regressions;
use regressions::RegressionPage;
pub(crate) use regressions::RegressionRootState;

// Named `Demo` to match the `Route::Demo` variant: the `Routable` derive
// renders each variant through a component of the same name.
#[component]
pub fn Demo(slug: String) -> Element {
    let title = registry::find_demo(&slug)
        .map(|spec| spec.name)
        .unwrap_or("未知示例")
        .to_string();

    let page = match slug.as_str() {
        "accessibility" => rsx! { accessibility::AccessibilityPage {} },
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
        "router" => rsx! { router_example::RouterPage {} },
        "shadcn_showcase" => rsx! { shadcn_showcase::ShadcnShowcasePage {} },
        "terminal" => rsx! { terminal_example::TerminalPage {} },
        "webview" => rsx! { webview_example::WebviewPage {} },
        "regressions" => rsx! { RegressionPage {} },
        _ => rsx! { UnknownDemo { slug: slug.clone() } },
    };

    // Keep this diagnostic page free of a native transition wrapper. Its
    // final two siblings deliberately exercise a renderer-root placeholder
    // replacement while an already-mounted portal is active.
    if slug == "regressions" {
        return rsx! {
            DemoFrame { title, {page} }
        };
    }

    rsx! {
        RouteTransition::<Route> {
            DemoFrame { title, {page} }
        }
    }
}

/// PC windows do not have the phone navigation bar/gesture affordance. Keep
/// the exit action in the common route frame so every top-level demo —
/// including demos with their own nested router — has the same escape path.
#[component]
fn DemoFrame(title: String, children: Element) -> Element {
    rsx! {
        column {
            width: "100%",
            height: "100%",
            background_color: "#fff9f9fa",
            DemoOperationBar { title }
            column {
                width: "100%",
                layout_weight: 1.0,
                clip: true,
                {children}
            }
        }
    }
}

#[component]
fn DemoOperationBar(title: String) -> Element {
    let navigator = use_navigator();
    let mut hovering = use_signal(|| false);
    let mut focused = use_signal(|| false);
    let go_back = EventHandler::new(move |_: ()| {
        if navigator.can_go_back() {
            navigator.go_back();
        } else {
            navigator.replace(Route::Home {});
        }
    });

    rsx! {
        row {
            width: "100%",
            height: 52.0,
            padding_right: 20.0,
            padding_left: 12.0,
            align_items: "center",
            background_color: "#fff9f9fa",
            border_width: "0,0,1,0",
            border_color: "#14000000",
            border_style: "solid",
            tab_stop: true,

            button {
                button_type: "normal",
                width: 120.0,
                height: 36.0,
                padding_right: 12.0,
                padding_left: 10.0,
                alignment: "center",
                background_color: if hovering() || focused() { "#fff4f4f5" } else { "#00ffffff" },
                foreground_color: "#ff18181b",
                border_width: 0.0,
                border_radius: 6.0,
                focusable: true,
                focus_on_touch: false,
                default_focus: true,
                onhover: move |event| hovering.set(event.data().is_hovering),
                onfocus: move |_| focused.set(true),
                onblur: move |_| focused.set(false),
                onclick: move |_| go_back.call(()),
                row {
                    width: "100%",
                    align_items: "center",
                    justify_content: "center",
                    {icon_placeholder("chevron-left", 17.0, 0xff18181b)}
                    text {
                        margin_left: 6.0,
                        font_size: 13.0,
                        line_height: 18.0,
                        font_weight: 500,
                        font_color: "#ff18181b",
                        "返回示例列表"
                    }
                }
            }

            row {
                margin_left: 12.0,
                layout_weight: 1.0,
                clip: true,
                text {
                    width: "100%",
                    content: title,
                    font_size: 14.0,
                    line_height: 20.0,
                    font_weight: 600,
                    font_color: "#ff18181b",
                    max_lines: 1_i32,
                    text_overflow: "ellipsis",
                }
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
