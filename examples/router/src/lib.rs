//! Router example — dioxus-router with enum-based routes and full-screen page
//! switching. Navigation via arkit's ArkUI-native `<Link>` (renders as styled
//! clickable text, not a button or HTML anchor).

use arkit::dioxus_core::VNode;
use arkit::prelude::*;
// The upstream derive emits `::dioxus_router` paths. Bind that name through
// Arkit's supported router namespace instead of adding an internal crate edge.
use arkit::router::dioxus_router;
use arkit::router::{
    use_back_handler, Link, Outlet, Routable, RouteProvider, RouteTransition, Router,
};

#[derive(Routable, Clone, PartialEq, Debug)]
enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Home {},
    #[route("/settings")]
    Settings {},
    #[route("/users/:id")]
    Users { id: u32 },
}

#[component]
pub fn RouterPage() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn AppShell() -> Element {
    let _back_handler = use_back_handler();
    rsx! { Outlet::<Route> {} }
}

#[component]
fn Home() -> Element {
    rsx! {
        RouteTransition::<Route> {
            RouteProvider {
                column {
                    width: "100%",
                    align_items: "center",
                    background_color: "#fffef3c7",
                    padding_top: 48.0,
                    padding_bottom: 48.0,

                    text { font_size: 32.0, "Home" }
                    text {
                        margin_top: 12.0,
                        font_size: 16.0,
                        "Scroll, open a route, then go back to verify restoration."
                    }

                    Link { to: Route::Settings {}, "Go to Settings →" }
                    Link { to: Route::Users { id: 42 }, "Go to User 42 →" }

                    for index in 1..=30 {
                        text {
                            margin_top: 16.0,
                            font_size: 16.0,
                            "Scrollable row {index}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Settings() -> Element {
    rsx! {
        RouteTransition::<Route> {
            RouteProvider {
                column {
                    width: "100%",
                    height: "100%",
                    align_items: "center",
                    justify_content: "center",
                    background_color: "#ffe0f2fe",

                    text { font_size: 32.0, "Settings" }
                    text { margin_top: 12.0, font_size: 16.0, "Full-screen settings page" }

                    Link { to: Route::Home {}, "← Push a new Home visit" }
                }
            }
        }
    }
}

#[component]
fn Users(id: u32) -> Element {
    rsx! {
        RouteTransition::<Route> {
            RouteProvider {
                column {
                    width: "100%",
                    height: "100%",
                    align_items: "center",
                    justify_content: "center",
                    background_color: "#fffdf2f8",

                    text { font_size: 32.0, "User {id}" }
                    text { margin_top: 12.0, font_size: 16.0, "Full-screen user detail page" }

                    Link { to: Route::Home {}, "← Push a new Home visit" }
                }
            }
        }
    }
}
