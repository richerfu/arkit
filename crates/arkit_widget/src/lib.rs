#[cfg(not(feature = "api-22"))]
compile_error!("arkit_widget requires feature `api-22` as the baseline");

mod internal;
mod overlay;
mod render_impl;

pub use arkit_core::advanced;
pub use arkit_core::theme;
pub use arkit_core::{Horizontal, Length, Padding, Settings, Size, Theme, Vertical};
pub use internal::*;
pub use ohos_arkui_binding::api::node_custom_event::NodeCustomEvent;
pub use ohos_arkui_binding::event::inner_event::Event as ArkEvent;
pub use ohos_arkui_binding::types::advanced::{FontWeight, NodeCustomEventType, ShadowStyle};
pub use ohos_arkui_binding::types::alignment::Alignment;
pub use ohos_arkui_binding::types::direction::Direction;
pub use ohos_arkui_binding::types::event::NodeEventType;
pub use ohos_arkui_binding::types::text_alignment::TextAlignment;
pub use overlay::{
    anchored_overlay, floating_overlay, floating_overlay_with_builder,
    floating_overlay_with_builder_and_surfaces, floating_overlay_with_surfaces, modal_overlay,
    native_overlay, FloatingAlign, FloatingOverlaySpec, FloatingSide, FloatingSurfaceRegistry,
    LayoutFrame, LayoutSize, ModalOverlaySpec, ModalPresentation, NativeOverlayPlacement,
    OverlayDismissMode, OverlayStrategy,
};
pub use render_impl::ListVisibleContentChangeEvent;
pub use render_impl::{
    button, button_component, calendar_picker, calendar_picker_component, checkbox,
    checkbox_component, column, column_component, container, date_picker, date_picker_component,
    flex, flex_component, flow_item, flow_item_component, grid, grid_component, grid_item,
    grid_item_component, grouped_virtual_list, image, image_component, lazy, list, list_component,
    list_item, list_item_component, list_item_group_component, mount, observe_layout_frame,
    observe_layout_frame_enabled, observe_layout_size, observe_text_layout, patch, progress,
    progress_component, radio, radio_component, realize_attached_mount, refresh, refresh_component,
    row, row_component, scroll, scroll_component, slider, slider_component, stack, stack_component,
    swiper, swiper_component, text, text_area, text_area_component, text_component, text_input,
    text_input_component, toggle, toggle_component, virtual_grid, virtual_grid_component,
    virtual_list, virtual_list_component, virtual_water_flow, virtual_water_flow_component,
    water_flow_component, Attribute as ArkUINodeAttributeType,
    AttributeValue as ArkUINodeAttributeItem, BorderStyle, ButtonType, Component, Element,
    FlexDirection, FlexOptions, FlexWrap, FontStyle, GridScrollIndexEvent, HitTestBehavior,
    ItemAlignment, JustifyContent, Lazy, ListScrollIndexEvent, ListStickyStyle, MountedNode, Node,
    ObjectFit, ProgressLinearStyle, ProgressType, Renderer, ScrollOffset, ScrollViewport,
    TextLayoutLine, TextLayoutSnapshot, UiState, VirtualListGroup, VirtualVisibleRange, Visibility,
    WaterFlowScrollIndexEvent,
};
#[cfg(feature = "webview")]
pub use render_impl::{
    web_view, web_view_component, DownloadStartResult, WebViewController, WebViewStyle, Webview,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent {
    Mount,
    Unmount,
}

pub use render_impl::{
    ButtonElement, CalendarPickerElement, CheckboxElement, ColumnElement, ContainerElement,
    DatePickerElement, FlexElement, FlowItemElement, GridElement, GridItemElement, ImageElement,
    ListElement, ListItemElement, ListItemGroupElement, ProgressElement, RadioElement,
    RefreshElement, RowElement, ScrollElement, SliderElement, StackElement, SwiperElement,
    TextAreaElement, TextElement, TextInputElement, ToggleElement, WaterFlowElement,
};
#[cfg(feature = "webview")]
pub use render_impl::WebViewElement;

pub mod prelude {
    pub use crate::ListVisibleContentChangeEvent;
    pub use crate::{
        advanced, button, button_component, calendar_picker, calendar_picker_component, checkbox,
        checkbox_component, column, column_component, container, date_picker,
        date_picker_component, flow_item, flow_item_component, grid, grid_component, grid_item,
        grid_item_component, grouped_virtual_list, image, image_component, lazy, list,
        list_component, list_item, list_item_component, list_item_group_component, progress,
        progress_component, radio, radio_component, realize_attached_mount, refresh,
        refresh_component, row, row_component, scroll, scroll_component, slider, slider_component,
        stack, stack_component, swiper, swiper_component, text, text_area, text_area_component,
        text_component, text_input, text_input_component, toggle, toggle_component, virtual_grid,
        virtual_grid_component, virtual_list, virtual_list_component, virtual_water_flow,
        virtual_water_flow_component, water_flow_component, ArkEvent, ArkUINodeAttributeItem,
        ArkUINodeAttributeType, BorderStyle, ButtonType, Element, FloatingAlign,
        FloatingOverlaySpec, FloatingSide, FontStyle, FontWeight, GridScrollIndexEvent,
        HitTestBehavior, Horizontal, ItemAlignment, JustifyContent, LayoutFrame, LayoutSize, Lazy,
        Length, LifecycleEvent, ListScrollIndexEvent, ListStickyStyle, ModalOverlaySpec,
        ModalPresentation, NativeOverlayPlacement, NodeCustomEvent, NodeCustomEventType,
        NodeEventType, ObjectFit, OverlayDismissMode, OverlayStrategy, Padding,
        ProgressLinearStyle, ProgressType, ScrollOffset, ScrollViewport, ShadowStyle, Size,
        TextAlignment, TextLayoutLine, TextLayoutSnapshot, Theme, UiState, Vertical,
        VirtualListGroup, VirtualVisibleRange, Visibility, WaterFlowScrollIndexEvent,
    };
    #[cfg(feature = "webview")]
    pub use crate::{
        web_view, web_view_component, DownloadStartResult, WebViewController, WebViewElement,
        WebViewStyle, Webview,
    };
}
