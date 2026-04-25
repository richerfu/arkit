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
    image, image_component, list, list_component, list_item, list_item_component, mount,
    observe_layout_frame as observe_layout_frame_impl,
    observe_layout_size as observe_layout_size_impl,
    observe_text_layout as observe_text_layout_impl, patch, progress, progress_component, radio,
    radio_component, realize_attached_mount, refresh, refresh_component, row, row_component,
    scroll, scroll_component, slider, slider_component, stack, stack_component, swiper,
    swiper_component, text, text_area, text_area_component, text_component, text_input,
    text_input_component, toggle, toggle_component, Attribute as ArkUINodeAttributeType,
    AttributeValue as ArkUINodeAttributeItem, BorderStyle, ButtonType, Element, FontStyle,
    HitTestBehavior, ItemAlignment, JustifyContent, ListScrollIndexEvent, MountedNode, Node,
    ObjectFit, ProgressLinearStyle, ProgressType, Renderer, ScrollOffset, ScrollViewport,
    TextLayoutLine, TextLayoutSnapshot, UiState, Visibility,
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

pub type ButtonElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type CalendarPickerElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type CheckboxElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type ContainerElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type DatePickerElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type ListElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type ListItemElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type ProgressElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type RadioElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type RefreshElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type RowElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type ScrollElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type SliderElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type SwiperElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type TextAreaElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type TextElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type TextInputElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type ToggleElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
#[cfg(feature = "webview")]
pub type WebViewElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;

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
        date_picker_component, image, image_component, list, list_component, list_item,
        list_item_component, progress, progress_component, radio, radio_component,
        realize_attached_mount, refresh, refresh_component, row, row_component, scroll,
        scroll_component, slider, slider_component, stack, stack_component, swiper,
        swiper_component, text, text_area, text_area_component, text_component, text_input,
        text_input_component, toggle, toggle_component, ArkEvent, ArkUINodeAttributeItem,
        ArkUINodeAttributeType, BorderStyle, ButtonType, Element, FloatingAlign,
        FloatingOverlaySpec, FloatingSide, FontStyle, FontWeight, HitTestBehavior, Horizontal,
        ItemAlignment, JustifyContent, LayoutFrame, LayoutSize, Length, LifecycleEvent,
        ListScrollIndexEvent, ModalOverlaySpec, ModalPresentation, NativeOverlayPlacement,
        NodeCustomEvent, NodeCustomEventType, NodeEventType, ObjectFit, OverlayDismissMode,
        OverlayStrategy, Padding, ProgressLinearStyle, ProgressType, ScrollOffset, ScrollViewport,
        ShadowStyle, Size, TextAlignment, TextLayoutLine, TextLayoutSnapshot, Theme, UiState,
        Vertical, Visibility,
    };
    #[cfg(feature = "webview")]
    pub use crate::{
        web_view, web_view_component, DownloadStartResult, WebViewController, WebViewElement,
        WebViewStyle, Webview,
    };
}
