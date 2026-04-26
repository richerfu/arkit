#![allow(dead_code, unused_imports)]

use arkit::ohos_arkui_binding::arkui_input_binding::UIInputAction;
use arkit::prelude::ArkUINodeAttributeType;
use arkit::{
    BorderStyle, ButtonElement, CalendarPickerElement, Component, DatePickerElement, Element,
    FontStyle, FontWeight, HitTestBehavior, ItemAlignment, JustifyContent, ProgressElement,
    ProgressLinearStyle, ProgressType, RowElement, ScrollElement, SliderElement, SwiperElement,
    TextAreaElement, TextElement, TextInputElement, ToggleElement, Visibility,
};

use crate::styles::{
    body_text, body_text_regular, border_color, card_surface, input_surface, margin_top,
    muted_text, panel_surface, shadow_sm, title_text,
};
use crate::theme::{colors, radii, spacing, typography, with_alpha};
use std::rc::Rc;

const FIX_AT_IDEAL_SIZE_POLICY: i32 = 2;
const TABS_LIST_HEIGHT: f32 = 36.0;
const TABS_LIST_PADDING: f32 = 3.0;
const TABS_TRIGGER_HEIGHT: f32 = TABS_LIST_HEIGHT - (TABS_LIST_PADDING * 2.0);

pub(crate) fn touch_activate<Message: 'static, AppTheme: 'static>(
    row: RowElement<Message, AppTheme>,
    on_activate: impl Fn() + 'static,
) -> RowElement<Message, AppTheme> {
    row.on_event(arkit::prelude::NodeEventType::TouchEvent, move |event| {
        let Some(input_event) = event.input_event() else {
            return;
        };
        let _ = input_event.pointer_set_stop_propagation(true);

        if matches!(input_event.action, UIInputAction::Up) {
            on_activate();
        }
    })
}

pub(crate) fn take_component_slot<T>(slot: &std::cell::RefCell<Option<T>>, name: &str) -> T {
    slot.borrow_mut()
        .take()
        .unwrap_or_else(|| panic!("{name} was already consumed"))
}

pub(crate) fn widget_state<T: 'static>(
    tree: &mut arkit::advanced::widget::Tree,
    init: impl FnOnce() -> T,
) -> std::rc::Rc<std::cell::RefCell<T>> {
    tree.state()
        .get_or_insert_with(|| std::rc::Rc::new(std::cell::RefCell::new(init())))
        .clone()
}

pub(crate) fn request_widget_rerender() {
    arkit::internal::queue_ui_loop(|| {
        if let Some(runtime) = arkit::internal::current_runtime() {
            runtime.request_rerender();
        }
    });
}

macro_rules! impl_component_widget {
    ($type:ty, $message:ident, $render:expr) => {
        impl<$message: 'static> arkit::advanced::Widget<$message, arkit::Theme, arkit::Renderer>
            for $type
        {
            fn body(
                &self,
                _tree: &mut arkit::advanced::widget::Tree,
                _renderer: &arkit::Renderer,
            ) -> Option<Element<$message>> {
                Some($render(self))
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
                self
            }
        }

        impl<$message: 'static> From<$type> for Element<$message> {
            fn from(value: $type) -> Self {
                Element::new(value)
            }
        }
    };
}

mod accordion;
mod alert;
mod alert_dialog;
mod aspect_ratio;
mod avatar;
mod badge;
mod breadcrumb;
mod button;
mod calendar;
mod card;
mod carousel;
mod chart;
mod checkbox;
mod collapsible;
mod combobox;
mod command;
mod context_menu;
mod date_picker;
mod dialog;
mod drawer;
mod dropdown_menu;
mod floating_layer;
mod form;
mod hover_card;
mod input;
mod input_otp;
mod label;
mod menu_common;
mod menubar;
mod navigation_menu;
mod pagination;
mod popover;
mod progress;
mod radio_group;
mod resizable;
mod scroll_area;
mod select;
mod separator;
mod sheet;
mod sidebar;
mod skeleton;
mod slider;
mod surfaces;
mod switch;
mod table;
mod tabs;
mod text;
mod textarea;
mod toggle;
mod toggle_group;
mod tooltip;

pub use accordion::{Accordion, AccordionContentSpec, AccordionItemSpec, AccordionTriggerSpec};
pub use alert::{Alert, AlertDescription, AlertList, AlertTitle, AlertVariant};
pub use alert_dialog::AlertDialog;
pub use avatar::Avatar;
pub use badge::Badge;
pub use badge::BadgeVariant;
pub use button::{Button, ButtonSize, ButtonStyleExt, ButtonVariant};
pub use card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
pub use checkbox::Checkbox;
pub use collapsible::Collapsible;
pub use context_menu::{ContextMenu, ContextMenuEntry};
pub use dialog::{Dialog, DialogFooter, DialogHeader};
pub use dropdown_menu::{DropdownMenu, DropdownMenuEntry};
pub use floating_layer::FloatingAlign;
pub use hover_card::HoverCard;
pub use input::Input;
pub use label::Label;
pub use menu_common::{
    MenuActionEntry, MenuCheckboxEntry, MenuEntry, MenuLabelEntry, MenuRadioEntry, MenuSubmenuEntry,
};
pub use menubar::{Menubar, MenubarEntry, MenubarMenuSpec};
pub use popover::Popover;
pub use progress::Progress;
pub use radio_group::RadioGroup;
pub use select::Select;
pub use separator::Separator;
pub use skeleton::Skeleton;
pub use switch::Switch;
pub use table::Table;
pub use tabs::Tabs;
pub use text::{Text, TextVariant};
pub use textarea::Textarea;
pub use toggle::Toggle;
pub use toggle_group::ToggleGroup;
pub use tooltip::Tooltip;

pub(crate) fn dispatch_message<Message>(message: Message)
where
    Message: Send + 'static,
{
    arkit::internal::dispatch(message);
}

pub(crate) fn dispatch_optional_string<Message>(
    map: impl Fn(Option<String>) -> Message + 'static,
) -> Rc<dyn Fn(Option<String>)>
where
    Message: Send + 'static,
{
    Rc::new(move |value| dispatch_message(map(value)))
}

pub(crate) fn visibility_gate<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
    open: bool,
) -> Component<Message, AppTheme, Kind> {
    element
        .visibility(if open {
            Visibility::Visible
        } else {
            Visibility::None
        })
        .opacity(if open { 1.0_f32 } else { 0.0_f32 })
        .hit_test_behavior(if open {
            HitTestBehavior::Default
        } else {
            HitTestBehavior::Transparent
        })
}

pub(crate) fn stack<Message: 'static>(
    children: Vec<Element<Message>>,
    gap: f32,
) -> Element<Message> {
    let items = children
        .into_iter()
        .enumerate()
        .map(|(index, child)| {
            if index == 0 {
                arkit::row_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .children(vec![child])
                    .into()
            } else {
                margin_top(
                    arkit::row_component::<Message, arkit::Theme>()
                        .percent_width(1.0)
                        .children(vec![child]),
                    gap,
                )
                .into()
            }
        })
        .collect::<Vec<_>>();

    arkit::column_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .children(items)
        .into()
}

pub(crate) fn inline<Message: 'static>(
    children: Vec<Element<Message>>,
    gap: f32,
) -> Vec<Element<Message>> {
    children
        .into_iter()
        .enumerate()
        .map(|(index, child)| {
            if index == 0 {
                child
            } else {
                arkit::row_component::<Message, arkit::Theme>()
                    .margin_left(gap)
                    .children(vec![child])
                    .into()
            }
        })
        .collect()
}

pub(crate) fn rounded_progress<Message>(
    element: ProgressElement<Message>,
) -> ProgressElement<Message> {
    element
        .progress_type(ProgressType::Linear)
        .progress_linear_style(ProgressLinearStyle::new(8.0, 4.0))
        .border_radius(radii().full)
        .clip(true)
        .background_color(colors().secondary)
}

pub(crate) fn rounded_table_surface<Message, AppTheme, Kind>(
    element: Component<Message, AppTheme, Kind>,
) -> Component<Message, AppTheme, Kind> {
    element.border_radius(radii().sm).clip(true)
}

pub(crate) fn rounded_menubar_surface<Message>(
    element: RowElement<Message>,
) -> RowElement<Message> {
    element
        .padding(spacing::XXS)
        .height(36.0)
        .align_items_center()
        .border_radius(radii().md)
        .border_width(1.0)
        .border_color(colors().border)
        .background_color(colors().background)
}

pub(crate) fn rounded_tabs_list_surface<Message>(
    element: RowElement<Message>,
) -> RowElement<Message> {
    element
        .attr(
            ArkUINodeAttributeType::WidthLayoutpolicy,
            FIX_AT_IDEAL_SIZE_POLICY,
        )
        .padding(TABS_LIST_PADDING)
        .height(TABS_LIST_HEIGHT)
        .align_items_center()
        .justify_content_center()
        .border_radius(radii().lg)
        .background_color(colors().muted)
}
