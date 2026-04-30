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
    list_item, list_item_component, list_item_group_component, mount,
    observe_layout_frame as observe_layout_frame_impl,
    observe_layout_size as observe_layout_size_impl,
    observe_text_layout as observe_text_layout_impl, patch, progress, progress_component, radio,
    radio_component, realize_attached_mount, refresh, refresh_component, row, row_component,
    scroll, scroll_component, slider, slider_component, stack, stack_component, swiper,
    swiper_component, text, text_area, text_area_component, text_component, text_input,
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

pub type ButtonElement<Message = (), AppTheme = Theme> =
    render_impl::ButtonElement<Message, AppTheme>;
pub type CalendarPickerElement<Message = (), AppTheme = Theme> =
    render_impl::CalendarPickerElement<Message, AppTheme>;
pub type CheckboxElement<Message = (), AppTheme = Theme> =
    render_impl::CheckboxElement<Message, AppTheme>;
pub type ColumnElement<Message = (), AppTheme = Theme> =
    render_impl::ColumnElement<Message, AppTheme>;
pub type ContainerElement<Message = (), AppTheme = Theme> =
    render_impl::ContainerElement<Message, AppTheme>;
pub type DatePickerElement<Message = (), AppTheme = Theme> =
    render_impl::DatePickerElement<Message, AppTheme>;
pub type FlexElement<Message = (), AppTheme = Theme> = render_impl::FlexElement<Message, AppTheme>;
pub type FlowItemElement<Message = (), AppTheme = Theme> =
    render_impl::FlowItemElement<Message, AppTheme>;
pub type GridElement<Message = (), AppTheme = Theme> = render_impl::GridElement<Message, AppTheme>;
pub type GridItemElement<Message = (), AppTheme = Theme> =
    render_impl::GridItemElement<Message, AppTheme>;
pub type ImageElement<Message = (), AppTheme = Theme> =
    render_impl::ImageElement<Message, AppTheme>;
pub type ListElement<Message = (), AppTheme = Theme> = render_impl::ListElement<Message, AppTheme>;
pub type ListItemElement<Message = (), AppTheme = Theme> =
    render_impl::ListItemElement<Message, AppTheme>;
pub type ListItemGroupElement<Message = (), AppTheme = Theme> =
    render_impl::ListItemGroupElement<Message, AppTheme>;
pub type ProgressElement<Message = (), AppTheme = Theme> =
    render_impl::ProgressElement<Message, AppTheme>;
pub type RadioElement<Message = (), AppTheme = Theme> =
    render_impl::RadioElement<Message, AppTheme>;
pub type RefreshElement<Message = (), AppTheme = Theme> =
    render_impl::RefreshElement<Message, AppTheme>;
pub type RowElement<Message = (), AppTheme = Theme> = render_impl::RowElement<Message, AppTheme>;
pub type ScrollElement<Message = (), AppTheme = Theme> =
    render_impl::ScrollElement<Message, AppTheme>;
pub type SliderElement<Message = (), AppTheme = Theme> =
    render_impl::SliderElement<Message, AppTheme>;
pub type StackElement<Message = (), AppTheme = Theme> =
    render_impl::StackElement<Message, AppTheme>;
pub type SwiperElement<Message = (), AppTheme = Theme> =
    render_impl::SwiperElement<Message, AppTheme>;
pub type TextAreaElement<Message = (), AppTheme = Theme> =
    render_impl::TextAreaElement<Message, AppTheme>;
pub type TextElement<Message = (), AppTheme = Theme> = render_impl::TextElement<Message, AppTheme>;
pub type TextInputElement<Message = (), AppTheme = Theme> =
    render_impl::TextInputElement<Message, AppTheme>;
pub type ToggleElement<Message = (), AppTheme = Theme> =
    render_impl::ToggleElement<Message, AppTheme>;
pub type WaterFlowElement<Message = (), AppTheme = Theme> =
    render_impl::WaterFlowElement<Message, AppTheme>;
#[cfg(feature = "webview")]
pub type WebViewElement<Message = (), AppTheme = Theme> =
    render_impl::WebViewElement<Message, AppTheme>;

pub fn observe_layout_size<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    on_change: impl Fn(LayoutSize) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    observe_layout_size_impl(element, on_change)
}

pub fn observe_text_layout<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    text: impl Into<String>,
    on_change: impl Fn(TextLayoutSnapshot) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    observe_text_layout_impl(element, text, on_change)
}

pub fn observe_layout_frame<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    on_change: impl Fn(LayoutFrame) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    observe_layout_frame_impl(element, true, on_change)
}

pub fn observe_layout_frame_enabled<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    enabled: bool,
    on_change: impl Fn(LayoutFrame) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    observe_layout_frame_impl(element, enabled, on_change)
}

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
