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
    rsx! { Outlet::<Route> {} }
}
