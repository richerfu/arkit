//! Consolidated demo — every example in one native module, dispatched through
//! a routed home page.
//!
//! The home page lists all examples grouped by category. Selecting an entry
//! pushes `/demo/:slug`; the demo page dispatches to the example's page
//! component. Examples that own navigation (shadcn_showcase, router) embed
//! their dioxus-router as a nested router, so back-press unwinds the inner
//! history first, then the outer history returns to the home page.

use arkit::dioxus_core::VNode;
use arkit::entry;
use arkit::prelude::*;
// The Routable derive emits `::dioxus_router` paths. Bind that name through
// arkit's supported router namespace instead of adding an internal crate edge.
use arkit::router::dioxus_router;
use arkit::router::{use_back_handler, Outlet, Routable, Router};

mod demo_page;
mod home;
mod registry;

// The `Routable` derive renders each route variant through a component of the
// same name, so the page components must be in scope next to the enum.
use demo_page::Demo;
use demo_page::RegressionRootState;
use home::Home;

#[derive(Routable, Clone, PartialEq, Debug)]
enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Home {},
    #[route("/demo/:slug")]
    Demo { slug: String },
}

#[entry]
fn app() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn AppShell() -> Element {
    // Outer router back handling: consumes the system back press while the
    // router can go back (demo page → home). Inner routers (shadcn_showcase,
    // router_example) register their own handlers after this one, and the
    // runtime dispatches back presses newest-first, so nested pages unwind
    // before the outer history pops.
    let _back_handler = use_back_handler();
    let root_state = RegressionRootState {
        active: use_signal(|| false),
        marker: use_signal(|| false),
    };
    use_context_provider(|| root_state);
    let active = (root_state.active)();
    let marker = (root_state.marker)();

    rsx! {
        Outlet::<Route> {}

        // These are direct AppShell siblings so their native parent is the
        // renderer's synthetic root. The regression route first activates the
        // portal, then toggles the trailing placeholder into an ordinary node.
        if active {
            Portal {
                layer: OverlayLayer::Transient,
                stack {
                    width: "100%",
                    height: "100%",
                    alignment: "bottom-end",
                    hit_test_behavior: "none",
                    padding_right: 14.0,
                    padding_bottom: 18.0,
                    if marker {
                        row {
                            width: 168.0,
                            height: 54.0,
                            align_items: "center",
                            justify_content: "center",
                            background_color: "#E6DC2626",
                            border_radius: 9.0,
                            text {
                                font_size: 12.0,
                                font_weight: 700,
                                font_color: "#FFFFFFFF",
                                "PASS: PORTAL TOP"
                            }
                        }
                    }
                }
            }
        }

        if active && marker {
            stack {
                width: "100%",
                height: "100%",
                alignment: "bottom-end",
                hit_test_behavior: "none",
                padding_right: 14.0,
                padding_bottom: 18.0,
                row {
                    width: 168.0,
                    height: 54.0,
                    align_items: "center",
                    justify_content: "center",
                    background_color: "#E62563EB",
                    border_radius: 9.0,
                    text {
                        font_size: 12.0,
                        font_weight: 700,
                        font_color: "#FFFFFFFF",
                        "FAIL: ROOT ON TOP"
                    }
                }
            }
        }
    }
}
