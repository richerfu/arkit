//! ArkUI event handlers usable inside `rsx!`.
//!
//! `rsx!` emits references to `dioxus_elements::events::<name>` (and
//! `dioxus_elements::events::<name>::call_with_explicit_closure` for inline
//! closures). Each event is a callable returning a `dioxus_core::Attribute`.
//!
//! The listener wraps the user callback and converts the platform
//! [`ArkEventData`](crate::event::ArkEventData) into the typed event data.

use crate::event::{
    AreaData, ChangeData, ClickData, FocusData, HoverData, PointerData, ReachEndData, RefreshData,
    ScrollData, SubmitData, SwiperChangeData,
};

macro_rules! impl_event {
    ($data:ty; $($name:ident)*) => { $(
        #[inline]
        pub fn $name<__Marker>(mut _f: impl ::dioxus_core::SuperInto<::dioxus_core::ListenerCallback<$data>, __Marker>) -> ::dioxus_core::Attribute {
            let event_handler = _f.super_into();
            ::dioxus_core::Attribute::new(
                stringify!($name),
                ::dioxus_core::AttributeValue::listener(move |e: ::dioxus_core::Event<$crate::event::ArkEventData>| {
                    let event: ::dioxus_core::Event<$data> = e.map(|d| d.into());
                    event_handler.call(event.into_any());
                }),
                None,
                false,
            )
        }

        #[doc(hidden)]
        pub mod $name {
            use super::*;

            #[allow(deprecated)]
            pub fn call_with_explicit_closure<__Marker, Return: ::dioxus_core::SpawnIfAsync<__Marker> + 'static>(
                event_handler: impl FnMut(::dioxus_core::Event<$data>) -> Return + 'static,
            ) -> ::dioxus_core::Attribute {
                super::$name(event_handler)
            }
        }
    )* };
}

impl_event! {
    ClickData;
    onclick onlongpress
}

impl_event! {
    ChangeData;
    onchange oninput ontoggle
}

impl_event! {
    SubmitData;
    onsubmit
}

impl_event! {
    ScrollData;
    onscroll
}

impl_event! {
    ReachEndData;
    onreachend
}

impl_event! {
    SwiperChangeData;
    onswiperchange
}

impl_event! {
    RefreshData;
    onrefresh
}

impl_event! {
    AreaData;
    onarea
}

impl_event! {
    HoverData;
    onhover
}

impl_event! {
    FocusData;
    onfocus onblur
}

impl_event! {
    PointerData;
    onhovermove ondragstart ondragmove ondragend ondragleave ondragenter ontouch
}
