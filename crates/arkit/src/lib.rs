mod ohos;
pub mod router;

pub use arkit_core::advanced;
pub use arkit_core::theme;
pub use arkit_core::{window, Horizontal, Length, Padding, Settings, Size, Theme, Vertical};
pub use arkit_derive::entry;
pub use arkit_runtime::{Application, Program, Subscription, SubscriptionHandle, Task};
pub use arkit_widget::ListVisibleContentChangeEvent;
pub use arkit_widget::Renderer;
pub use arkit_widget::{
    button, button_component, calendar_picker, calendar_picker_component, checkbox,
    checkbox_component, column, column_component, container, date_picker, date_picker_component,
    floating_overlay, floating_overlay_with_builder, flow_item, flow_item_component, grid,
    grid_component, grid_item, grid_item_component, grouped_virtual_list, image, image_component,
    list, list_component, list_item, list_item_component, list_item_group_component, modal_overlay,
    observe_text_layout, progress, progress_component, radio, radio_component, refresh,
    refresh_component, row, row_component, scroll, scroll_component, slider, slider_component,
    stack, stack_component, swiper, swiper_component, text, text_area, text_area_component,
    text_component, text_input, text_input_component, toggle, toggle_component, virtual_grid,
    virtual_grid_component, virtual_list, virtual_list_component, virtual_water_flow,
    virtual_water_flow_component, water_flow_component, ArkEvent, ArkUINodeAttributeItem,
    ArkUINodeAttributeType, BorderStyle, ButtonElement, ButtonType, CalendarPickerElement,
    CheckboxElement, ColumnElement, Component, ContainerElement, DatePickerElement, Element,
    FloatingAlign, FloatingOverlaySpec, FloatingSide, FlowItemElement, FontStyle, FontWeight,
    GridElement, GridItemElement, GridScrollIndexEvent, HitTestBehavior, ImageElement,
    ItemAlignment, JustifyContent, LayoutFrame, LayoutSize, LifecycleEvent, ListElement,
    ListItemElement, ListItemGroupElement, ListScrollIndexEvent, ListStickyStyle, ModalOverlaySpec,
    ModalPresentation, NativeOverlayPlacement, Node, NodeCustomEvent, NodeCustomEventType,
    NodeEventType, ObjectFit, OverlayDismissMode, OverlayStrategy, ProgressElement,
    ProgressLinearStyle, ProgressType, RadioElement, RefreshElement, RowElement, ScrollElement,
    ScrollOffset, ScrollViewport, ShadowStyle, SliderElement, StackElement, SwiperElement,
    TextAlignment, TextAreaElement, TextElement, TextInputElement, TextLayoutLine,
    TextLayoutSnapshot, ToggleElement, UiState, VirtualListGroup, VirtualVisibleRange, Visibility,
    WaterFlowElement, WaterFlowScrollIndexEvent,
};
#[cfg(feature = "webview")]
pub use arkit_widget::{
    web_view, web_view_component, DownloadStartResult, WebViewController, WebViewElement,
    WebViewStyle, Webview,
};
pub use ohos::{
    mount_application, mount_entry, napi_derive_ohos, napi_ohos, ohos_arkui_binding,
    openharmony_ability, ApplicationRuntime, EntryPoint, MountedEntryHandle,
};

pub fn application<State, Message, Boot, Update, View>(
    boot: Boot,
    update: Update,
    view: View,
) -> Application<State, Message, Theme, Renderer>
where
    State: 'static,
    Message: Send + 'static,
    Boot: Fn() -> State + 'static,
    Update: Fn(&mut State, Message) -> Task<Message> + 'static,
    View: Fn(&State) -> Element<Message, Theme> + 'static,
{
    arkit_runtime::application::<State, Message, Boot, Update, View, Theme, Renderer>(
        boot, update, view,
    )
}

pub fn run<State, Message, Update, View>(
    update: Update,
    view: View,
) -> Application<State, Message, Theme, Renderer>
where
    State: Default + 'static,
    Message: Send + 'static,
    Update: Fn(&mut State, Message) -> Task<Message> + 'static,
    View: Fn(&State) -> Element<Message, Theme> + 'static,
{
    application(State::default, update, view)
}

pub mod widget {
    pub use crate::{
        button, button_component, calendar_picker, calendar_picker_component, checkbox,
        checkbox_component, column, column_component, container, date_picker,
        date_picker_component, flow_item, flow_item_component, grid, grid_component, grid_item,
        grid_item_component, grouped_virtual_list, image, image_component, list, list_component,
        list_item, list_item_component, list_item_group_component, progress, progress_component,
        radio, radio_component, refresh, refresh_component, row, row_component, scroll,
        scroll_component, slider, slider_component, stack, stack_component, swiper,
        swiper_component, text, text_area, text_area_component, text_component, text_input,
        text_input_component, toggle, toggle_component, virtual_grid, virtual_grid_component,
        virtual_list, virtual_list_component, virtual_water_flow, virtual_water_flow_component,
        water_flow_component,
    };
    #[cfg(feature = "webview")]
    pub use crate::{web_view, web_view_component};
}

pub mod prelude {
    pub use crate::router::RouterNavigationExt;
    pub use crate::widget::*;
    pub use crate::ListVisibleContentChangeEvent;
    pub use crate::{
        application, entry, observe_text_layout, run, window, ArkEvent, ArkUINodeAttributeItem,
        ArkUINodeAttributeType, BorderStyle, ButtonType, Element, FloatingAlign,
        FloatingOverlaySpec, FloatingSide, FontStyle, FontWeight, GridScrollIndexEvent,
        HitTestBehavior, Horizontal, ItemAlignment, JustifyContent, LayoutFrame, LayoutSize,
        Length, LifecycleEvent, ListScrollIndexEvent, ListStickyStyle, ModalOverlaySpec,
        ModalPresentation, NativeOverlayPlacement, NodeCustomEvent, NodeCustomEventType,
        NodeEventType, ObjectFit, OverlayDismissMode, OverlayStrategy, Padding, Program,
        ProgressLinearStyle, ProgressType, Renderer, ScrollOffset, ScrollViewport, Settings,
        ShadowStyle, Size, Subscription, SubscriptionHandle, Task, TextAlignment, TextLayoutLine,
        TextLayoutSnapshot, Theme, UiState, Vertical, VirtualListGroup, VirtualVisibleRange,
        Visibility, WaterFlowScrollIndexEvent,
    };
    #[cfg(feature = "webview")]
    pub use crate::{
        web_view, web_view_component, DownloadStartResult, WebViewController, WebViewElement,
        WebViewStyle, Webview,
    };
}

#[doc(hidden)]
pub mod internal {
    pub use arkit_runtime::internal::*;
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
