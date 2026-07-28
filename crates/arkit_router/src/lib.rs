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
//! - [`RouteTransition`] / [`AnimatedOutlet`] — ArkUI-native route enter
//!   transitions that compose with Dioxus `Router`/`Outlet`.
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
use arkit_prelude::*;
use dioxus_core::Element;
use dioxus_core_macro::{component, rsx, Props};
pub use provider::{Router, RouterProps};
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
    let _registration = use_hook(|| {
        Rc::new(arkit_runtime::register_back_press_handler(
            registered_handler,
        ))
    });
    move || handler()
}

// ---------------------------------------------------------------------------
// Route transitions — Dioxus Router composition
// ---------------------------------------------------------------------------

/// Props shared by route transition components.
#[derive(Props, Clone, PartialEq)]
pub struct RouteTransitionProps {
    /// Transition preset used when the route subtree mounts.
    #[props(default)]
    pub preset: Option<TransitionPreset>,
    /// Animation duration in milliseconds. Defaults to `180`.
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

/// Animate already-keyed route content.
///
/// This component must be rendered inside a `dioxus-router` [`Router`]. It
/// intentionally does not own navigation state; callers that need route-change
/// remounts should key the subtree themselves or use [`AnimatedOutlet`].
#[component]
pub fn RouteTransition<R: dioxus_router::Routable + Clone>(props: RouteTransitionProps) -> Element {
    let _ = std::marker::PhantomData::<R>;
    let preset = props.preset.or(Some(TransitionPreset::SlideLeft));
    let duration_ms = props.duration_ms.or(Some(220));
    let fill = props.fill.or(Some(true));

    rsx! {
        arkit_animation::MountTransition {
            preset,
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
    /// Transition preset used when the active outlet route changes.
    #[props(default)]
    pub preset: Option<TransitionPreset>,
    /// Animation duration in milliseconds. Defaults to `220`.
    #[props(default)]
    pub duration_ms: Option<i32>,
    /// Animation delay in milliseconds. Defaults to `0`.
    #[props(default)]
    pub delay_ms: Option<i32>,
    /// Whether the wrapper should fill its parent. Defaults to `true`.
    #[props(default)]
    pub fill: Option<bool>,
}

/// A Dioxus-compatible animated variant of [`Outlet`].
///
/// Use this in layout routes where Dioxus examples would normally render
/// `Outlet::<Route> {}`. The nested outlet depth is still owned by
/// `dioxus-router`; this component only wraps the rendered outlet in a keyed
/// mount transition.
#[component]
pub fn AnimatedOutlet<R: dioxus_router::Routable + Clone>(props: AnimatedOutletProps) -> Element {
    let route_key = dioxus_router::use_route::<R>().to_string();
    let preset = props.preset.or(Some(TransitionPreset::SlideLeft));
    let duration_ms = props.duration_ms.or(Some(220));
    let fill = props.fill.or(Some(true));

    rsx! {
        arkit_animation::MountTransition {
            key: "{route_key}",
            preset,
            duration_ms,
            delay_ms: props.delay_ms,
            fill,
            dioxus_router::Outlet::<R> {}
        }
    }
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
            onclick: move |_| {
                navigator.push(to.clone());
            },
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use arkit_prelude::dioxus_core::VNode;
    use arkit_prelude::*;
    use dioxus_core::NoOpMutations;

    use super::{dioxus_router, use_back_handler, Routable, Router};

    thread_local! {
        static INSTALLED_BACK_HANDLER: RefCell<Option<Rc<dyn Fn() -> bool>>> =
            const { RefCell::new(None) };
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
}
