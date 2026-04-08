mod ohos;

pub use arkit_core::advanced;
pub use arkit_core::theme;
pub use arkit_core::{window, Horizontal, Length, Padding, Settings, Size, Theme, Vertical};
pub use arkit_derive::entry;
pub use arkit_widget::Renderer;
pub use ohos::{
    mount_application, mount_entry, napi_derive_ohos, napi_ohos, ohos_arkui_binding,
    openharmony_ability, ApplicationRuntime, EntryPoint, MountedEntryHandle,
};
pub use arkit_runtime::{Application, Preset, Program, Subscription, SubscriptionHandle, Task};
pub use arkit_widget::{
    button, button_component, calendar_picker, calendar_picker_component, checkbox,
    checkbox_component, column, column_component, container, date_picker, date_picker_component,
    image, image_component, progress, progress_component, radio, radio_component, row,
    row_component, scroll, scroll_component, slider, slider_component, stack, stack_component,
    swiper, swiper_component, text, text_area, text_area_component, text_component, text_input,
    text_input_component, toggle, toggle_component, ArkEvent, ArkUINodeAttributeItem,
    ArkUINodeAttributeType, ButtonElement, CalendarPickerElement, CheckboxElement,
    ContainerElement, DatePickerElement, Element, LayoutFrame, LayoutSize, LifecycleEvent,
    NativeOverlayPlacement, Node, NodeCustomEvent, NodeCustomEventType, NodeEventType,
    ProgressElement, RadioElement, RowElement, ScrollElement, SliderElement, SwiperElement,
    TextAreaElement, TextElement, TextInputElement, ToggleElement,
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
        date_picker_component, image, image_component, progress, progress_component, radio,
        radio_component, row, row_component, scroll, scroll_component, slider, slider_component,
        stack, stack_component, swiper, swiper_component, text, text_area, text_area_component,
        text_component, text_input, text_input_component, toggle, toggle_component,
    };
}

pub mod prelude {
    pub use crate::widget::*;
    pub use crate::{
        application, entry, run, window, ArkEvent, ArkUINodeAttributeItem, ArkUINodeAttributeType,
        Element, Horizontal, Length, LifecycleEvent, NodeCustomEvent, NodeCustomEventType,
        NodeEventType, Padding, Program, Renderer, Settings, Size, Subscription,
        SubscriptionHandle, Task, Theme, Vertical,
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
