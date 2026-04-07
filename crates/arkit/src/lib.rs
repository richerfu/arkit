pub use arkit_derive::{component, entry};
pub use arkit_router::NavigationStack;
pub use arkit_runtime::{
    anchored_overlay, animated_router_view, application, back_route, button,
    button_component, calendar_picker, calendar_picker_component, checkbox, checkbox_component,
    column, column_component, container, create_root, create_scope, current_app, current_route, custom,
    custom_component, custom_span, custom_span_component, date_picker, date_picker_component,
    dispatch, dynamic, embedded_component, embedded_component_component, enter_scope, flex,
    flex_component, flow_item, flow_item_component, for_each, grid, grid_component, grid_item,
    grid_item_component, image, image_animator, image_animator_component, image_component,
    image_span, image_span_component, keyed_scope, list, list_component, list_item,
    list_item_component, list_item_group, list_item_group_component, loading_progress,
    loading_progress_component, mount_application, mount_entry, napi_derive_ohos, napi_ohos,
    native_overlay, observe_layout_frame, observe_layout_frame_enabled, observe_layout_size,
    ohos_arkui_binding, on_cleanup, on_mount, openharmony_ability, portal_scope, progress,
    progress_component, provide_context, push_route, queue_after_mount, queue_ui_loop, radio,
    radio_component, refresh, refresh_component, register_fallback_route, register_named_route,
    register_route, register_routes, relative_container, relative_container_component,
    replace_route, reset_route, router, router_register_tree, router_view, row, row_component,
    scope, scope_owned, scroll, scroll_component, set_router, show, slider,
    slider_component, stack, stack_component, style, swiper, swiper_component, text, text_area,
    text_area_component, text_component, text_input, text_input_component, text_picker,
    text_picker_component, time_picker, time_picker_component, toggle, toggle_component, undefined,
    undefined_component, use_back_handler, use_before_leave, use_context, use_local_context, use_outlet,
    use_route, use_route_context, use_route_param, use_route_query, use_route_transition,
    use_router, water_flow,
    water_flow_component, with_dispatcher, xcomponent, xcomponent_component, xcomponent_texture,
    xcomponent_texture_component, Application, ArkEvent, ArkUINodeAttributeItem,
    ArkUINodeAttributeType, ButtonElement, ButtonMessageExt, ButtonStyleExt, CalendarPickerElement,
    CheckboxMessageExt, CheckboxStyleExt, ComponentElement, ContainerAlignmentExt,
    ContainerElement, ContainerStyleExt, DatePickerElement, Element, EntryPoint, Horizontal,
    LayoutFrame, LayoutSize, Length, LifecycleEvent, MountedEntryHandle, NativeOverlayPlacement,
    NodeCustomEvent, NodeCustomEventType, NodeEventType, Padding, Program, ProgressElement,
    ReactiveHost, Renderer, ResolvedRoute, Route, RouteContext, RouteDefinition,
    RouteError, RouteNode, RouteSegmentMatch, RouteTransition, RouteTransitionConfig,
    RouteTransitionDirection, Router, RowElement, Runtime, ScrollElement, Settings, SliderElement,
    Subscription, SwiperElement, Task, TextAreaElement, TextAreaMessageExt, TextElement,
    TextInputElement, TextInputMessageExt, TextInputStyleExt, TextStyleExt, Theme, ToggleElement,
    Vertical,
};

pub mod runtime {
    pub use arkit_runtime::{
        current_app, mount_entry, queue_after_mount, queue_ui_loop, EntryPoint, MountedEntryHandle,
        Runtime,
    };
}

pub mod widget {
    pub use arkit_runtime::{
        button, button_component, calendar_picker, calendar_picker_component, checkbox,
        checkbox_component, column, column_component, container, custom, custom_component,
        custom_span, custom_span_component, date_picker, date_picker_component, dynamic,
        embedded_component, embedded_component_component, flex, flex_component, flow_item,
        flow_item_component, for_each, grid, grid_component, grid_item, grid_item_component, image,
        image_animator, image_animator_component, image_component, image_span,
        image_span_component, keyed_scope, list, list_component, list_item, list_item_component,
        list_item_group, list_item_group_component, loading_progress, loading_progress_component,
        progress, progress_component, radio, radio_component, refresh, refresh_component,
        relative_container, relative_container_component, row, row_component, scope, scope_owned,
        scroll, scroll_component, show, slider, slider_component, stack, stack_component, style,
        swiper, swiper_component, text, text_area, text_area_component, text_component, text_input,
        text_input_component, text_picker, text_picker_component, time_picker,
        time_picker_component, toggle, toggle_component, undefined, undefined_component,
        water_flow, water_flow_component, xcomponent, xcomponent_component, xcomponent_texture,
        xcomponent_texture_component, ArkEvent, ArkUINodeAttributeItem, ArkUINodeAttributeType,
        ButtonElement, ButtonMessageExt, ButtonStyleExt, CalendarPickerElement, CheckboxMessageExt,
        CheckboxStyleExt, ComponentElement, ContainerAlignmentExt, ContainerElement,
        ContainerStyleExt, DatePickerElement, Element, Horizontal, Length, NodeCustomEvent,
        NodeCustomEventType, NodeEventType, Padding, ProgressElement, RowElement, ScrollElement,
        SliderElement, SwiperElement, TextAreaElement, TextAreaMessageExt, TextElement,
        TextInputElement, TextInputMessageExt, TextInputStyleExt, TextStyleExt, ToggleElement,
        Vertical,
    };
}

pub mod advanced {
    pub use arkit_runtime::{Program, ReactiveHost, RouteTransitionConfig, Subscription, Task};
}

pub mod prelude {
    pub use crate::widget::*;
    pub use crate::{
        anchored_overlay, animated_router_view, application, back_route, component, create_root,
        create_scope, current_app, current_route, dispatch, entry, native_overlay,
        observe_layout_frame, observe_layout_frame_enabled, observe_layout_size, on_cleanup,
        on_mount, portal_scope, provide_context, push_route, register_fallback_route,
        register_named_route, register_route, register_routes, replace_route, reset_route, router,
        router_register_tree, router_view, scope, scope_owned, set_router, use_back_handler,
        use_before_leave, use_context, use_local_context, use_outlet, use_route,
        use_route_context, use_route_param, use_route_query, use_route_transition, use_router, Application,
        LayoutFrame, LayoutSize, LifecycleEvent, NativeOverlayPlacement, NavigationStack, Program,
        ReactiveHost, Renderer, ResolvedRoute, Route, RouteContext, RouteDefinition, RouteError,
        RouteNode, RouteSegmentMatch, RouteTransition, RouteTransitionConfig,
        RouteTransitionDirection, Router, Runtime, Settings, Subscription, Task, Theme,
    };
}

#[macro_export]
macro_rules! row {
    ($($child:expr),* $(,)?) => {
        $crate::row(vec![$(($child).into()),*])
    };
}

#[macro_export]
macro_rules! column {
    ($($child:expr),* $(,)?) => {
        $crate::column(vec![$(($child).into()),*])
    };
}
