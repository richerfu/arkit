use arkit::ohos_arkui_binding::arkui_input_binding::UIInputAction;
use arkit::prelude::ArkUINodeAttributeType;
use arkit::{
    BorderStyle, ButtonElement, ButtonType, CalendarPickerElement, DatePickerElement, Element,
    FontStyle, FontWeight, HitTestBehavior, ItemAlignment, JustifyContent, Node, ProgressElement,
    RowElement, ScrollElement, SliderElement, SwiperElement, TextAreaElement, TextElement,
    TextInputElement, ToggleElement, Visibility,
};

use crate::styles::{
    body_text, body_text_regular, border_color, card_surface, input_surface, margin_top,
    muted_text, panel_surface, shadow_sm, title_text,
};
use crate::theme::{color, radius, spacing, typography};
use std::rc::Rc;

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

pub use accordion::{
    accordion, accordion_content, accordion_item_parts, accordion_item_spec,
    accordion_single_controlled, accordion_trigger, accordion_trigger_text, AccordionContentSpec,
    AccordionItemSpec,
};
pub use alert::*;
pub use alert_dialog::{
    alert_dialog, alert_dialog_actions, alert_dialog_modal_message as alert_dialog_modal,
};
pub use aspect_ratio::*;
pub use avatar::*;
pub use badge::*;
pub use breadcrumb::*;
pub use button::*;
pub use calendar::*;
pub use card::*;
pub use carousel::*;
pub use chart::*;
pub use checkbox::{
    checkbox_message as checkbox,
    checkbox_with_checked_color_message as checkbox_with_checked_color, disabled_checkbox,
};
pub use collapsible::collapsible_message as collapsible;
pub use combobox::*;
pub use command::*;
pub use context_menu::{
    context_menu_checkbox_item_message as context_menu_checkbox_item, context_menu_item,
    context_menu_item_destructive, context_menu_item_inset, context_menu_item_inset_destructive,
    context_menu_item_inset_with_shortcut,
    context_menu_item_inset_with_shortcut_action_message as context_menu_item_inset_with_shortcut_action,
    context_menu_item_with_shortcut, context_menu_label, context_menu_label_inset,
    context_menu_message as context_menu,
    context_menu_radio_item_message as context_menu_radio_item, context_menu_separator,
    context_menu_sub_trigger_inset, context_menu_subcontent,
    context_menu_submenu_inset_message as context_menu_submenu_inset,
    disabled_context_menu_item_inset_with_shortcut,
};
pub use date_picker::*;
pub use dialog::{dialog_footer, dialog_header, dialog_message as dialog};
pub use drawer::*;
pub use dropdown_menu::{
    disabled_dropdown_item, disabled_dropdown_item_with_shortcut,
    dropdown_checkbox_item_message as dropdown_checkbox_item, dropdown_item,
    dropdown_item_destructive, dropdown_item_inset, dropdown_item_inset_with_shortcut,
    dropdown_item_with_shortcut, dropdown_label, dropdown_label_inset,
    dropdown_menu_message as dropdown_menu, dropdown_radio_item_message as dropdown_radio_item,
    dropdown_separator, dropdown_subcontent, dropdown_submenu_message as dropdown_submenu,
};
pub use form::*;
pub use hover_card::{
    hover_card_message as hover_card, hover_card_with_width_message as hover_card_with_width,
};
pub use input::*;
pub use input_otp::*;
pub use label::*;
pub use menu_common::MenuEntry;
pub use menubar::{
    menubar, menubar_item, menubar_item_active, menubar_menu,
    menubar_message as menubar_with_menus, MenubarEntry, MenubarMenuSpec,
};
pub use navigation_menu::*;
pub use pagination::*;
pub use popover::{
    popover_card, popover_message as popover, popover_with_width_message as popover_with_width,
};
pub use progress::*;
pub use radio_group::radio_group_message as radio_group;
pub use resizable::*;
pub use scroll_area::*;
pub use select::select_message as select;
pub use separator::*;
pub use sheet::*;
pub use sidebar::*;
pub use skeleton::*;
pub use slider::*;
pub use surfaces::{sonner, toast, toast_destructive};
pub use switch::*;
pub use table::*;
pub use tabs::tabs_message as tabs;
pub use text::*;
pub use textarea::*;
pub use toggle::{toggle_icon_message as toggle_icon, toggle_message as toggle};
pub use toggle_group::{
    toggle_group_icons_message as toggle_group_icons,
    toggle_group_icons_multi_message as toggle_group_icons_multi,
    toggle_group_message as toggle_group, toggle_group_multi_message as toggle_group_multi,
};
pub use tooltip::tooltip_message as tooltip;

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

pub(crate) fn visibility_gate<Message, AppTheme>(
    element: Node<Message, AppTheme>,
    open: bool,
) -> Node<Message, AppTheme> {
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
        .border_radius(radius::FULL)
        .background_color(color::PRIMARY_TRACK)
}

pub(crate) fn rounded_table_surface<Message, AppTheme>(
    element: Node<Message, AppTheme>,
) -> Node<Message, AppTheme> {
    element.border_radius(radius::SM).clip(true)
}

pub(crate) fn rounded_menubar_surface<Message>(
    element: RowElement<Message>,
) -> RowElement<Message> {
    element
        .padding(3.0)
        .height(36.0)
        .align_items_center()
        .border_radius(radius::MD)
        .border_width(1.0)
        .border_color(color::BORDER)
        .background_color(color::BACKGROUND)
}

pub(crate) fn rounded_tabs_list_surface<Message>(
    element: RowElement<Message>,
) -> RowElement<Message> {
    element
        .padding(3.0)
        .height(36.0)
        .align_items_center()
        .border_radius(radius::LG)
        .background_color(color::MUTED)
}
