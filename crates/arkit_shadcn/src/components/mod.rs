use arkit::prelude::ArkUINodeAttributeType;
use arkit::{
    ButtonElement, CalendarPickerElement, ComponentElement, DatePickerElement, Element,
    ProgressElement, ScrollElement, Signal, SliderElement, SwiperElement, TextAreaElement,
    TextElement, TextInputElement, ToggleElement,
};

use crate::styles::{
    body_text, body_text_regular, border_color, card_surface, chip_surface, input_surface,
    margin_top, muted_text, panel_surface, title_text,
};
use crate::theme::{color, radius, spacing, typography};

mod basic;
mod data;
mod navigation;
mod overlays;
mod surfaces;

pub use basic::*;
pub use data::*;
pub use navigation::*;
pub use overlays::*;
pub use surfaces::*;

pub(crate) const FLEX_ALIGN_CENTER: i32 = 2;
pub(crate) const FLEX_ALIGN_END: i32 = 3;
pub(crate) const FLEX_ALIGN_SPACE_BETWEEN: i32 = 6;

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
        .style(ArkUINodeAttributeType::BackgroundColor, color::MUTED)
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
        .style(ArkUINodeAttributeType::Padding, vec![3.0, 3.0, 3.0, 3.0])
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::SM, radius::SM, radius::SM, radius::SM],
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
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::MD, radius::MD, radius::MD, radius::MD],
        )
        .background_color(color::MUTED)
}

pub(crate) fn rounded_button_surface(element: ButtonElement) -> ButtonElement {
    element
        .style(
            ArkUINodeAttributeType::Padding,
            vec![8.0, spacing::LG, 8.0, spacing::LG],
        )
        .style(ArkUINodeAttributeType::BorderRadius, vec![radius::SM; 4])
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![0.0, 0.0, 0.0, 0.0],
        )
}
