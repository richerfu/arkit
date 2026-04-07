use arkit::prelude::ArkUINodeAttributeType;
use arkit::{
    ButtonElement, CalendarPickerElement, ComponentElement, DatePickerElement, Element,
    ProgressElement, ReactiveHost, RowElement, ScrollElement, SliderElement, SwiperElement,
    TextAreaElement, TextElement, TextInputElement, ToggleElement,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::styles::{
    body_text, body_text_regular, border_color, card_surface, input_surface, margin_top,
    muted_text, panel_surface, shadow_sm, title_text,
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
pub(crate) const HIT_TEST_TRANSPARENT: i32 = 2;
pub(crate) const VISIBILITY_HIDDEN: i32 = 2;

pub(crate) fn visibility_gate<T>(
    element: ComponentElement<T>,
    open: bool,
) -> ComponentElement<T>
where
    T: ReactiveHost,
{
    element
        .style(
            ArkUINodeAttributeType::Visibility,
            if open { 0_i32 } else { VISIBILITY_HIDDEN },
        )
        .style(ArkUINodeAttributeType::Opacity, if open { 1.0_f32 } else { 0.0_f32 })
        .style(
            ArkUINodeAttributeType::HitTestBehavior,
            if open { 0_i32 } else { HIT_TEST_TRANSPARENT },
        )
}

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

pub(crate) fn rounded_menubar_surface(element: RowElement) -> RowElement {
    element
        .style(ArkUINodeAttributeType::Padding, vec![4.0, 4.0, 4.0, 4.0])
        .height(40.0)
        .align_items_center()
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

pub(crate) fn rounded_tabs_list_surface(element: RowElement) -> RowElement {
    element
        .style(ArkUINodeAttributeType::Padding, vec![3.0, 3.0, 3.0, 3.0])
        .height(36.0)
        .align_items_center()
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::LG, radius::LG, radius::LG, radius::LG],
        )
        .background_color(color::MUTED)
}

pub(crate) fn request_runtime_rerender() {
    arkit::queue_ui_loop(|| {
        if let Some(runtime) = arkit_runtime::current_runtime() {
            let _ = runtime.request_rerender();
        }
    });
}

pub(crate) fn local_bool_state<T: Clone + 'static>(marker: T, initial: bool) -> Rc<Cell<bool>> {
    #[derive(Clone)]
    struct LocalBool<T> {
        _marker: T,
        value: Rc<Cell<bool>>,
    }

    // Component-local state must stay on the current owner only. Walking up the
    // owner tree would incorrectly reuse parent widget state in nested widgets.
    if let Some(state) = arkit::use_local_context::<LocalBool<T>>() {
        return state.value;
    }

    let value = Rc::new(Cell::new(initial));
    arkit::provide_context(LocalBool {
        _marker: marker,
        value: value.clone(),
    });
    value
}

pub(crate) fn local_ref_state<M: Clone + 'static, T: Clone + 'static>(
    marker: M,
    initial: T,
) -> Rc<RefCell<T>> {
    #[derive(Clone)]
    struct LocalRef<M, T> {
        _marker: M,
        value: Rc<RefCell<T>>,
    }

    // Same rule as `local_bool_state`: this state is private to the current
    // component owner and must not inherit from ancestors.
    if let Some(state) = arkit::use_local_context::<LocalRef<M, T>>() {
        return state.value;
    }

    let value = Rc::new(RefCell::new(initial));
    arkit::provide_context(LocalRef {
        _marker: marker,
        value: value.clone(),
    });
    value
}
