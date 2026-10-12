//! arkit router — a thin facade over `dioxus-router` 0.7.
//!
//! We reuse `dioxus-router` directly (not a fork) for:
//! - `#[derive(Routable)]` enum-based structured routing with `#[route("/path")]`
//! - Nested rendering via `<Outlet>`
//! - Navigation hooks: `use_route`, `navigator`
//!
//! Arkit-specific additions:
//! - [`ArkLink`] — an ArkUI-native `<Link>` replacement (renders a `button`,
//!   not an HTML `<a>`) with the same `to: Route` API.
//! - [`use_back_handler`] — OHOS back-button integration.
//! - [`AnimatedOutlet`] — overlapping page fades with retained exit content.
//! - [`RouteTransition`] — an enter-only helper for keyed route content.
//! - [`RouteProvider`] — a native page root that restores its default scroll
//!   position when navigating back.

mod provider;
mod scroll;

// Keep the upstream crate available as an explicit namespace for advanced
// APIs. The flattened facade is deliberately narrow so upstream additions do
// not silently become Arkit's public API and `Link` remains ArkUI-native.
pub use dioxus_router;
pub use dioxus_router::{
    navigator, root_router, router, try_router, use_navigator, use_outlet_context, use_route,
    GenericRouterContext, NavigationTarget, Navigator, Outlet, OutletContext, ParseRouteError,
    Routable, RouterConfig, RouterContext,
};

use std::rc::Rc;

pub use arkit_animation::TransitionPreset;
use arkit_animation::{
    page_enter_spring, page_exit_spring, spring_duration_ms, use_animate_presence, PresenceKey,
    PresenceMode, PresencePhase, PAGE_ENTER_DELAY_MS,
};
use arkit_prelude::*;
use dioxus_core::{provide_context, Element};
use dioxus_core_macro::{component, rsx, Props};
pub use provider::{MemoryRouter, Router, RouterProps};
pub use scroll::{RouteProvider, RouteProviderProps};

/// Register the OHOS back-button handler to navigate the dioxus-router history
/// back. Call once near the app root (inside a component rendered by the
/// router).
///
/// A back press is consumed (returns `true`) when the router can go back;
/// otherwise it passes through to the system. The native interceptor enters
/// through a Dioxus callback so router history is resolved in the component
/// scope that installed this hook, even though ArkTS initiated the call.
pub fn use_back_handler() -> impl Fn() -> bool {
    // Headless VirtualDom tests and non-ArkUI renderers can still use the
    // returned navigation callback; only native interception requires a root
    // runtime context.
    let runtime = dioxus_core::try_consume_context::<arkit_runtime::RuntimeHandle>();
    let navigator = dioxus_router::navigator();
    let scoped_handler = dioxus_hooks::use_callback(move |()| {
        if navigator.can_go_back() {
            navigator.go_back();
            true
        } else {
            false
        }
    });
    let handler: Rc<dyn Fn() -> bool> = Rc::new(move || scoped_handler.call(()));
    let registered_handler = handler.clone();
    let _registration = use_hook(move || {
        runtime.map(|runtime| Rc::new(runtime.register_back_handler(registered_handler)))
    });
    move || handler()
}

// ---------------------------------------------------------------------------
// Route transitions — Dioxus Router composition
// ---------------------------------------------------------------------------

/// Props shared by route transition components.
#[derive(Props, Clone, PartialEq)]
pub struct RouteTransitionProps {
    /// Transition preset used when the route subtree mounts. Defaults to `Fade`.
    #[props(default)]
    pub preset: Option<TransitionPreset>,
    /// Duration in milliseconds; defaults to the page enter spring's natural duration.
    #[props(default)]
    pub duration_ms: Option<i32>,
    /// Animation delay in milliseconds. Defaults to `0`.
    #[props(default)]
    pub delay_ms: Option<i32>,
    /// Whether the wrapper should fill its parent. Defaults to `true` for
    /// route transitions.
    #[props(default)]
    pub fill: Option<bool>,
    /// Route content to animate.
    pub children: Element,
}

/// Play an enter-only fade with the page spring for already-keyed route content.
///
/// Key the subtree to replay on navigation. This helper cannot retain its
/// subtree after its parent unmounts it, so it has no exit animation. Prefer
/// [`AnimatedOutlet`] for page cross-fades, including back and parameter changes.
#[component]
pub fn RouteTransition<R: dioxus_router::Routable + Clone>(props: RouteTransitionProps) -> Element {
    let _ = std::marker::PhantomData::<R>;
    let spring = page_enter_spring();
    let preset = props.preset.or(Some(TransitionPreset::Fade));
    let duration_ms = props.duration_ms.or(Some(spring_duration_ms(spring)));
    let fill = props.fill.or(Some(true));

    rsx! {
        arkit_animation::MountTransition {
            preset,
            spring,
            duration_ms,
            delay_ms: props.delay_ms,
            fill,
            {props.children}
        }
    }
}

/// Props for [`AnimatedOutlet`].
#[derive(Props, Clone, PartialEq)]
pub struct AnimatedOutletProps {
    /// Preset for both entering and leaving pages. Defaults to opacity-only `Fade`.
    #[props(default)]
    pub preset: Option<TransitionPreset>,
    /// Entering fade duration in milliseconds; defaults to the page enter
    /// spring's natural duration. The leaving fade uses the shorter exit spring
    /// and never runs longer than this.
    #[props(default)]
    pub duration_ms: Option<i32>,
    /// Delay before the entering page fades in, in milliseconds. Defaults to
    /// `40` so the leaving page clears first. The leaving page never waits.
    #[props(default)]
    pub delay_ms: Option<i32>,
    /// Whether the wrapper should fill its parent. Defaults to `true`.
    #[props(default)]
    pub fill: Option<bool>,
}

/// A fading variant of [`Outlet`] using the page springs by default.
///
/// The first route is immediately present. On push, back and parameter changes
/// the leaving page fades out at once on a fast spring while the entering page
/// fades in after a short delay, so the two pages' content barely overlaps
/// (about 400 ms in total). Each leaving route
/// retains its stored value until its exit completes and does not receive input.
/// Explicit preset, duration, delay and sizing overrides remain available.
///
/// Outlet depth follows `dioxus-router`. A leaving *layout* containing a nested
/// [`Outlet`] still renders the router's current child during its fade-out:
/// upstream outlets read the live route, not the retained route value. Prefer
/// placing this component in a persistent layout around leaf pages.
#[component]
pub fn AnimatedOutlet<R: dioxus_router::Routable + Clone>(props: AnimatedOutletProps) -> Element {
    let route = use_route::<R>();
    let key = PresenceKey::new(route.to_string());
    let presence = use_animate_presence(PresenceMode::Sync, [(key.clone(), route)]);
    use_hook(|| {
        // Only the first entry skips enter; returning to this key later animates.
        presence.mark_present(&key);
    });
    let outlet = use_outlet_context::<R>();
    let level = outlet.level();
    provide_context(outlet.next());
    let preset = props.preset.or(Some(TransitionPreset::Fade));
    // The leaving page fades out immediately on a faster spring while the
    // entering page waits briefly, so their content barely overlaps.
    let enter_spring = page_enter_spring();
    let exit_spring = page_exit_spring();
    let enter_ms = props
        .duration_ms
        .unwrap_or_else(|| spring_duration_ms(enter_spring));
    let exit_ms = spring_duration_ms(exit_spring).min(enter_ms);
    let enter_delay_ms = props.delay_ms.unwrap_or(PAGE_ENTER_DELAY_MS);
    let fill = props.fill.unwrap_or(true);
    let mut entries = presence.entries();
    // Stable sort keeps navigation order within each group.
    entries.sort_by_key(|entry| entry.phase != PresencePhase::Leaving);

    rsx! {
        stack {
            width: "100%",
            height: if fill { "100%" },
            alignment: "top-start",
            hit_test_behavior: "transparent",
            // Leaving pages are drawn underneath and the live page claims hits
            // with `default`: `none` alone only skips the wrapper itself, so its
            // buttons would still catch taps through the new page's empty areas.
            for entry in entries {
                column {
                    key: "{entry.key.as_str()}",
                    width: "100%",
                    height: if fill { "100%" },
                    align_items: "start",
                    hit_test_behavior: if entry.phase == PresencePhase::Leaving { "none" } else { "default" },
                    arkit_animation::PresenceTransition {
                        phase: entry.phase,
                        preset,
                        spring: if entry.phase == PresencePhase::Leaving { exit_spring } else { enter_spring },
                        duration_ms: enter_ms,
                        exit_duration_ms: exit_ms,
                        delay_ms: if entry.phase == PresencePhase::Leaving { 0 } else { enter_delay_ms },
                        fill,
                        on_terminal: {
                            let presence = presence.clone();
                            let key = entry.key.clone();
                            move |phase| match phase {
                                PresencePhase::Entering => { presence.mark_present(&key); }
                                PresencePhase::Leaving => { presence.settle_exit(&key); }
                                PresencePhase::Present => {}
                            }
                        },
                        RetainedRoute {
                            route_key: entry.key.as_str().to_owned(),
                            {entry.value.render(level)}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RetainedRoute(route_key: String, children: Element) -> Element {
    use_context_provider(|| scroll::RouteScrollKey(Some(route_key)));
    children
}

// ---------------------------------------------------------------------------
// Link — an ArkUI-native navigation link
// ---------------------------------------------------------------------------

/// Props for [`Link`]. `to` is the route to navigate to on click; `children`
/// is the link's content (typically a `text` element styled by the caller).
/// `font_color` sets the default link text color (defaults to blue).
#[derive(Props, Clone, PartialEq)]
pub struct LinkProps<R: Clone + PartialEq + 'static> {
    /// The route to navigate to when the link is clicked.
    pub to: R,
    /// The link's content (typically a `text` element or plain string).
    pub children: Element,
    /// Optional font color for the link text (hex string, e.g. `"#ff2563eb"`).
    /// If `None`, defaults to a standard blue.
    #[props(default)]
    pub color: Option<String>,
    /// Optional font size (vp).
    #[props(default)]
    pub font_size: Option<f32>,
    /// Accessible name. When omitted, ArkUI derives it from the link content.
    #[props(default)]
    pub accessibility_label: Option<String>,
    /// Optional destination or usage hint.
    #[props(default)]
    pub accessibility_description: Option<String>,
}

/// An ArkUI-native navigation link. Renders as a clickable `text` element
/// (not a `button`) that calls `navigator().push(to)` on click. The caller
/// controls styling through `children` (e.g. wrap in a `text { font_size, font_color, ... }`).
///
/// This shadows dioxus-router's `Link` (which renders an HTML `<a>`) to work
/// with the ArkUI renderer.
///
/// ```ignore
/// // Simple: plain text link with default styling
/// Link { to: Route::Settings {}, "Go to Settings" }
///
/// // Custom: wrap content in styled text
/// Link { to: Route::Settings {},
///     text { font_size: 20.0, font_color: "#ffdc2626", "Settings" }
/// }
/// ```
#[component]
pub fn Link<R: Clone + PartialEq + 'static + std::fmt::Debug + dioxus_router::Routable>(
    props: LinkProps<R>,
) -> Element {
    let to = props.to.clone();
    let navigator = dioxus_router::navigator();

    // If the children is a plain string, wrap it in a text element with the
    // link's default/optional styling. Otherwise render children as-is (the
    // caller controls styling).
    let color = props
        .color
        .clone()
        .unwrap_or_else(|| "#ff2563eb".to_string());
    let font_size = props.font_size.unwrap_or(18.0);

    rsx! {
        text {
            font_size: font_size,
            font_color: color,
            accessibility_role: "link",
            accessibility_text: if let Some(label) = props.accessibility_label { label },
            accessibility_description: if let Some(description) = props.accessibility_description { description },
            accessibility_group: true,
            accessibility_actions: "click",
            focusable: true,
            focus_on_touch: true,
            onclick: move |_| {
                navigator.push(to.clone());
            },
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use arkit_prelude::dioxus_core::VNode;
    use arkit_prelude::*;
    use dioxus_core::NoOpMutations;

    use super::{dioxus_router, use_back_handler, MemoryRouter, Routable, Router};

    thread_local! {
        static INSTALLED_BACK_HANDLER: RefCell<Option<Rc<dyn Fn() -> bool>>> =
            const { RefCell::new(None) };
        static NESTED_CHILD_RENDERED: Cell<bool> = const { Cell::new(false) };
    }

    #[derive(Routable, Clone, Debug, PartialEq)]
    enum TestRoute {
        #[layout(TestShell)]
        #[route("/")]
        Home {},
        #[route("/other")]
        Other {},
    }

    fn test_app() -> Element {
        rsx! { Router::<TestRoute> {} }
    }

    #[component]
    fn TestShell() -> Element {
        let back_handler: Rc<dyn Fn() -> bool> = Rc::new(use_back_handler());
        let installed_handler = back_handler.clone();
        use_hook(move || {
            INSTALLED_BACK_HANDLER.with(|slot| slot.replace(Some(installed_handler)));
        });

        let navigator = dioxus_router::navigator();
        use_effect(move || {
            navigator.push(TestRoute::Other {});
        });

        rsx! { dioxus_router::Outlet::<TestRoute> {} }
    }

    #[component]
    fn Home() -> Element {
        rsx! { "home" }
    }

    #[component]
    fn Other() -> Element {
        rsx! { "other" }
    }

    #[derive(Routable, Clone, Debug, PartialEq)]
    enum ParentRoute {
        #[route("/")]
        NestedParentHome {},
        #[route("/nested")]
        NestedParentPage {},
    }

    #[derive(Routable, Clone, Debug, PartialEq)]
    enum ChildRoute {
        #[route("/")]
        NestedChildHome {},
    }

    fn nested_test_app() -> Element {
        rsx! { Router::<ParentRoute> {} }
    }

    #[component]
    fn NestedParentHome() -> Element {
        let navigator = dioxus_router::navigator();
        use_effect(move || {
            navigator.push(ParentRoute::NestedParentPage {});
        });
        VNode::empty()
    }

    #[component]
    fn NestedParentPage() -> Element {
        use_context_provider(|| super::scroll::RouteScrollKey(Some("/nested".to_owned())));
        rsx! { MemoryRouter::<ChildRoute> {} }
    }

    #[component]
    fn NestedChildHome() -> Element {
        assert_eq!(super::scroll::use_scroll_route(), "/");
        use_hook(|| {
            NESTED_CHILD_RENDERED.with(|rendered| rendered.set(true));
        });
        VNode::empty()
    }

    #[test]
    fn back_handler_reenters_the_installing_dioxus_scope() {
        let mut dom = dioxus_core::VirtualDom::new(test_app);
        let mut mutations = NoOpMutations;
        dom.rebuild(&mut mutations);
        dom.render_immediate(&mut mutations);

        let handler = INSTALLED_BACK_HANDLER.with(|slot| {
            slot.borrow()
                .as_ref()
                .expect("test shell should install a back handler")
                .clone()
        });

        assert!(handler(), "the pushed route should consume system back");
        assert!(!handler(), "the root route should pass system back through");

        INSTALLED_BACK_HANDLER.with(|slot| slot.borrow_mut().take());
    }

    #[test]
    fn memory_router_does_not_parse_the_parent_path() {
        NESTED_CHILD_RENDERED.with(|rendered| rendered.set(false));

        let mut dom = dioxus_core::VirtualDom::new(nested_test_app);
        let mut mutations = NoOpMutations;
        dom.rebuild(&mut mutations);
        dom.render_immediate(&mut mutations);

        assert!(
            NESTED_CHILD_RENDERED.with(Cell::get),
            "the child router must start at its own root instead of parsing /nested",
        );
    }
}

#[cfg(test)]
mod transition_tests {
    use std::cell::RefCell;

    use super::*;
    use dioxus_core::{AttributeValue, Mutation, Mutations, VNode, VirtualDom};

    thread_local! {
        static NAVIGATOR: RefCell<Option<(ScopeId, Navigator)>> = const { RefCell::new(None) };
        static MOUNTED: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
        static DROPPED: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
        static SCROLL_KEYS: RefCell<Vec<(u32, String)>> = const { RefCell::new(Vec::new()) };
    }

    #[derive(Routable, Clone, Debug, PartialEq)]
    enum Route {
        #[layout(Shell)]
        #[route("/")]
        Home {},
        #[route("/page/:id")]
        Page { id: u32 },
    }

    fn app() -> Element {
        arkit_animation::engine::use_animation_host_provider();
        rsx! { MemoryRouter::<Route> {} }
    }

    #[component]
    fn Shell() -> Element {
        let navigator = navigator();
        use_hook(move || {
            NAVIGATOR.with(|slot| slot.replace(Some((current_scope_id(), navigator))))
        });
        rsx! { AnimatedOutlet::<Route> {} }
    }

    #[component]
    fn Home() -> Element {
        rsx! { Page { id: 0 } }
    }

    #[component]
    fn Page(id: u32) -> Element {
        let _route = use_route::<Route>();
        let scroll_key = scroll::use_scroll_route();
        SCROLL_KEYS.with(|keys| keys.borrow_mut().push((id, scroll_key)));
        use_hook(move || MOUNTED.with(|mounted| mounted.borrow_mut().push(id)));
        use_drop(move || DROPPED.with(|dropped| dropped.borrow_mut().push(id)));
        rsx! { "page {id}" }
    }

    #[test]
    fn outlet_retains_leaf_pages_through_parameter_changes_and_back() {
        MOUNTED.with(|mounted| mounted.borrow_mut().clear());
        DROPPED.with(|dropped| dropped.borrow_mut().clear());
        SCROLL_KEYS.with(|keys| keys.borrow_mut().clear());
        let mut dom = VirtualDom::new(app);
        let initial = dom.rebuild_to_vec();
        assert!(initial.edits.iter().any(|edit| matches!(
            edit,
            Mutation::SetAttribute {
                name: "opacity",
                value: AttributeValue::Float(1.0),
                ..
            }
        )));
        assert_eq!(MOUNTED.with(|mounted| mounted.borrow().clone()), [0]);
        let (scope, navigator) = NAVIGATOR.with(|slot| slot.borrow().unwrap());
        for id in [1, 2] {
            dom.in_scope(scope, || navigator.push(Route::Page { id }));
            let mut mutations = Mutations::default();
            dom.render_immediate(&mut mutations);
            assert!(mutations.edits.iter().any(|edit| matches!(edit,
                Mutation::SetAttribute { name: "hit_test_behavior", value: AttributeValue::Text(value), .. } if value == "none"
            )));
            assert!(mutations.edits.iter().any(|edit| matches!(
                edit,
                Mutation::SetAttribute {
                    name: "opacity",
                    value: AttributeValue::Float(0.0),
                    ..
                }
            )));
        }
        dom.in_scope(scope, || navigator.go_back());
        dom.render_immediate(&mut dioxus_core::NoOpMutations);
        assert_eq!(MOUNTED.with(|mounted| mounted.borrow().clone()), [0, 1, 2]);
        assert!(DROPPED.with(|dropped| dropped.borrow().is_empty()));
        SCROLL_KEYS.with(|keys| {
            let keys = keys.borrow();
            assert!(keys.iter().filter(|(id, _)| *id == 0).count() > 1);
            for (id, key) in keys.iter() {
                let expected = if *id == 0 {
                    "/".to_owned()
                } else {
                    format!("/page/{id}")
                };
                assert_eq!(key, &expected, "leaving pages retain their scroll identity");
            }
        });
        // No native frame driver runs in this test: all entries remain until
        // terminal callbacks or outlet teardown. Presence tests cover settlement.
        drop(dom);
        let mut dropped = DROPPED.with(|dropped| dropped.borrow().clone());
        dropped.sort_unstable();
        assert_eq!(dropped, [0, 1, 2]);
        NAVIGATOR.with(|slot| slot.borrow_mut().take());
    }
}
