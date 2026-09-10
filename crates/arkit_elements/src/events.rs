//! ArkUI event handlers usable inside `rsx!`.
//!
//! `rsx!` emits references to `dioxus_elements::events::<name>` (and
//! `dioxus_elements::events::<name>::call_with_explicit_closure` for inline
//! closures). Each event is a callable returning a `dioxus_core::Attribute`.
//!
//! The listener wraps the user callback and converts the platform
//! [`ArkEventData`](crate::event::ArkEventData) into the typed event data.

use crate::event::{
    AreaData, ArkEventKind, ChangeData, ClickData, FocusData, HoverData, PointerData, ReachEndData,
    RefreshData, ScrollData, SubmitData, SwiperChangeData,
};

/// Define every RSX listener, its payload type, and its semantic identity.
///
/// The declaration is the single source of truth: it generates the listener
/// functions, their `call_with_explicit_closure` companions *and*
/// [`EVENT_KINDS`], which [`crate::event::classify_event_name`] consults.
/// Adding an event in one place cannot leave the classifier behind.
macro_rules! impl_events {
    ($(
        $data:ty;
        $($name:ident => $kind:ident),* $(,)?;
    )*) => {
        $($(
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
        )*)*

        /// Every listener name declared in this file, paired with its
        /// semantic identity.
        ///
        /// Names are stored in their RSX spelling (`onclick`);
        /// [`crate::event::classify_event_name`] also accepts the compact
        /// spelling Dioxus hands to renderers (`click`).
        pub(crate) const EVENT_KINDS: &[(&str, ArkEventKind)] = &[
            $($(
                (stringify!($name), ArkEventKind::$kind),
            )*)*
        ];
    };
}

impl_events! {
    ClickData;
    onclick => Click, onlongpress => LongPress;
    ChangeData;
    onchange => Change, oninput => Change, ontoggle => Change;
    SubmitData;
    onsubmit => Submit;
    ScrollData;
    onscroll => Scroll;
    ReachEndData;
    onreachend => ReachEnd;
    SwiperChangeData;
    onswiperchange => SwiperChange;
    RefreshData;
    onrefresh => Refresh;
    AreaData;
    onarea => AreaChange;
    HoverData;
    onhover => Hover;
    FocusData;
    onfocus => Focus, onblur => Blur;
    PointerData;
    onhovermove => HoverMove, ondragstart => DragStart, ondragmove => DragMove,
    ondragend => DragEnd, ondragleave => DragLeave, ondragenter => DragEnter, ontouch => Touch;
}
