#[cfg(not(feature = "api-22"))]
compile_error!("arkit_widget requires feature `api-22` as the baseline");

mod internal;
mod render_impl;

pub use arkit_core::advanced;
pub use arkit_core::theme;
pub use arkit_core::{Horizontal, Length, Padding, Settings, Size, Theme, Vertical};
pub use render_impl::{
    button, button_component, calendar_picker, calendar_picker_component, checkbox,
    checkbox_component, column, column_component, container, date_picker, date_picker_component,
    image, image_component, mount, patch, progress, progress_component, radio, radio_component,
    row, row_component, scroll, scroll_component, slider, slider_component, stack,
    stack_component, swiper, swiper_component, text, text_area, text_area_component,
    text_component, text_input, text_input_component, toggle, toggle_component,
    Attribute as ArkUINodeAttributeType, AttributeValue as ArkUINodeAttributeItem, Element,
    MountedNode, Node, Renderer,
};
pub use internal::*;
pub use ohos_arkui_binding::api::node_custom_event::NodeCustomEvent;
pub use ohos_arkui_binding::event::inner_event::Event as ArkEvent;
pub use ohos_arkui_binding::types::advanced::NodeCustomEventType;
pub use ohos_arkui_binding::types::alignment::Alignment;
pub use ohos_arkui_binding::types::direction::Direction;
pub use ohos_arkui_binding::types::event::NodeEventType;

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
pub type ProgressElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type RadioElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type RowElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type ScrollElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type SliderElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type SwiperElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type TextAreaElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type TextElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type TextInputElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;
pub type ToggleElement<Message = (), AppTheme = Theme> = Node<Message, AppTheme>;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeOverlayPlacement {
    pub alignment: Alignment,
    pub offset_x: f32,
    pub offset_y: f32,
    pub direction: Direction,
}

impl NativeOverlayPlacement {
    pub fn new(alignment: Alignment) -> Self {
        Self {
            alignment,
            offset_x: 0.0,
            offset_y: 0.0,
            direction: Direction::Auto,
        }
    }

    pub fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
}

impl Default for NativeOverlayPlacement {
    fn default() -> Self {
        Self::new(Alignment::TopStart)
    }
}

impl LayoutSize {
    pub fn is_measured(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

impl LayoutFrame {
    pub fn is_measured(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

pub fn anchored_overlay<Message: 'static, AppTheme: 'static>(
    trigger: Element<Message, AppTheme>,
    panel: Option<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    match panel {
        Some(panel) => column(vec![trigger, panel]),
        None => trigger,
    }
}

pub fn native_overlay<Message: 'static, AppTheme: 'static>(
    trigger: Element<Message, AppTheme>,
    panel: Option<Element<Message, AppTheme>>,
    _placement: NativeOverlayPlacement,
) -> Element<Message, AppTheme> {
    anchored_overlay(trigger, panel)
}

pub fn observe_layout_size<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    _on_change: impl Fn(LayoutSize) + 'static,
) -> Element<Message, AppTheme> {
    element
}

pub fn observe_layout_frame<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    _on_change: impl Fn(LayoutFrame) + 'static,
) -> Element<Message, AppTheme> {
    element
}

pub fn observe_layout_frame_enabled<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    _enabled: bool,
    _on_change: impl Fn(LayoutFrame) + 'static,
) -> Element<Message, AppTheme> {
    element
}

pub fn on_cleanup(_cleanup: impl FnOnce() + 'static) {}

pub mod prelude {
    pub use crate::{
        advanced, button, button_component, calendar_picker, calendar_picker_component, checkbox,
        checkbox_component, column, column_component, container, date_picker,
        date_picker_component, image, image_component, progress, progress_component, radio,
        radio_component, row, row_component, scroll, scroll_component, slider, slider_component,
        stack, stack_component, swiper, swiper_component, text, text_area, text_area_component,
        text_component, text_input, text_input_component, toggle, toggle_component, ArkEvent,
        ArkUINodeAttributeItem, ArkUINodeAttributeType, Element, Horizontal, Length,
        LifecycleEvent, NodeCustomEvent, NodeCustomEventType, NodeEventType, Padding, Size, Theme,
        Vertical,
    };
}
