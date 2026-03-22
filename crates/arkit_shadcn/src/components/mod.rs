use arkit::prelude::ArkUINodeAttributeType;
use arkit::{
    ButtonElement, CalendarPickerElement, ComponentElement, DatePickerElement, Element,
    ProgressElement, ScrollElement, Signal, SliderElement, SwiperElement, TextAreaElement,
    TextElement, TextInputElement, ToggleElement,
};

use crate::styles::{
    body_text, body_text_regular, border_color, card_surface, input_surface,
    margin_top, muted_text, panel_surface, shadow_sm, title_text,
};
use crate::theme::{color, radius, spacing, typography};

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
mod form;
mod hover_card;
mod input;
mod input_otp;
mod label;
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

pub use accordion::*;
pub use alert::*;
pub use alert_dialog::*;
pub use aspect_ratio::*;
pub use avatar::*;
pub use badge::*;
pub use breadcrumb::*;
pub use button::*;
pub use calendar::*;
pub use card::*;
pub use carousel::*;
pub use chart::*;
pub use checkbox::*;
pub use collapsible::*;
pub use combobox::*;
pub use command::*;
pub use context_menu::*;
pub use date_picker::*;
pub use dialog::*;
pub use drawer::*;
pub use dropdown_menu::*;
pub use form::*;
pub use hover_card::*;
pub use input::*;
pub use input_otp::*;
pub use label::*;
pub use menubar::*;
pub use navigation_menu::*;
pub use pagination::*;
pub use popover::*;
pub use progress::*;
pub use radio_group::*;
pub use resizable::*;
pub use scroll_area::*;
pub use select::*;
pub use separator::*;
pub use sheet::*;
pub use sidebar::*;
pub use skeleton::*;
pub use slider::*;
pub use surfaces::{sonner, toast, toast_destructive};
pub use switch::*;
pub use table::*;
pub use tabs::*;
pub use text::*;
pub use textarea::*;
pub use toggle::*;
pub use toggle_group::*;
pub use tooltip::*;

pub(crate) const FLEX_ALIGN_CENTER: i32 = 2;
pub(crate) const FLEX_ALIGN_END: i32 = 3;
pub(crate) const FLEX_ALIGN_SPACE_BETWEEN: i32 = 6;
pub(crate) const FLEX_ALIGN_START: i32 = 1;

pub(crate) fn stack(children: Vec<Element>, gap: f32) -> Element {
    let items = children
        .into_iter()
        .enumerate()
        .map(|(index, child)| {
            if index == 0 {
                arkit::row_component()
                    .percent_width(1.0)
                    .children(vec![child])
                    .into()
            } else {
                margin_top(
                    arkit::row_component()
                        .percent_width(1.0)
                        .children(vec![child]),
                    gap,
                )
                .into()
            }
        })
        .collect::<Vec<_>>();

    arkit::column_component()
        .percent_width(1.0)
        .children(items)
        .into()
}

pub(crate) fn inline(children: Vec<Element>, gap: f32) -> Vec<Element> {
    children
        .into_iter()
        .enumerate()
        .map(|(index, child)| {
            if index == 0 {
                child
            } else {
                arkit::row_component()
                    .style(ArkUINodeAttributeType::Margin, vec![0.0, 0.0, 0.0, gap])
                    .children(vec![child])
                    .into()
            }
        })
        .collect()
}

pub(crate) fn rounded_progress(element: ProgressElement) -> ProgressElement {
    element
        .style(ArkUINodeAttributeType::BorderRadius, vec![radius::FULL])
        .style(
            ArkUINodeAttributeType::BackgroundColor,
            color::PRIMARY_TRACK,
        )
}

pub(crate) fn rounded_table_surface<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute + 'static,
{
    element
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::SM, radius::SM, radius::SM, radius::SM],
        )
        .style(ArkUINodeAttributeType::Clip, true)
}

pub(crate) fn rounded_menubar_surface<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute + 'static,
{
    element
        .style(ArkUINodeAttributeType::Padding, vec![4.0, 4.0, 4.0, 4.0])
        .height(40.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::MD, radius::MD, radius::MD, radius::MD],
        )
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![1.0, 1.0, 1.0, 1.0],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .background_color(color::BACKGROUND)
}

pub(crate) fn rounded_tabs_list_surface<T>(element: ComponentElement<T>) -> ComponentElement<T>
where
    T: arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute + 'static,
{
    element
        .style(ArkUINodeAttributeType::Padding, vec![3.0, 3.0, 3.0, 3.0])
        .height(36.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::LG, radius::LG, radius::LG, radius::LG],
        )
        .background_color(color::MUTED)
}
