use crate::prelude::*;
use arkit_icon as lucide;
use arkit_shadcn as shadcn;

use crate::{Message, Route};

const HOME_HEADER_HEIGHT: f32 = 80.0;
const DETAIL_HEADER_HEIGHT: f32 = 48.0;
const TRACKING_TIGHT: f32 = -0.35;

fn empty_box(width: f32, height: f32) -> Element {
    arkit::row_component().width(width).height(height).into()
}

fn nav_title_text(title: impl Into<String>, home: bool) -> Element {
    let title = title.into();
    let mut text = arkit::text(title)
        .font_color(shadcn::theme::color::FOREGROUND)
        .text_letter_spacing(TRACKING_TIGHT);

    if home {
        text = text
            .font_size(34.0)
            .font_weight(FontWeight::W700)
            .line_height(40.0);
    } else {
        text = text
            .font_size(17.0)
            .font_weight(FontWeight::W500)
            .line_height(22.0);
    }

    text.into()
}

fn plain_back_button() -> Element {
    shadcn::icon_button("chevron-left")
        .theme(shadcn::ButtonVariant::Ghost)
        .width(36.0)
        .height(36.0)
        .padding(arkit::Padding::ZERO)
        .on_press(Message::Back)
        .into()
}

fn constrained_width(child: Element, width: f32) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .align_items_center()
        .children(vec![arkit::row_component()
            .percent_width(1.0)
            .max_width_constraint(width)
            .justify_content_center()
            .children(vec![child])
            .into()])
        .into()
}

fn showcase_horizontal_padding(value: f32) -> f32 {
    value
}

fn canvas_row(content: Element, center_x: bool) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .justify_content(if center_x {
            JustifyContent::Center
        } else {
            JustifyContent::Start
        })
        .children(vec![content])
        .into()
}

pub(crate) fn fixed_width(child: Element, width: f32) -> Element {
    constrained_width(child, width)
}

pub(crate) fn max_width(child: Element, width: f32) -> Element {
    constrained_width(child, width)
}

pub(crate) fn v_stack(children: Vec<Element>, gap: f32) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .align_items_start()
        .children(
            children
                .into_iter()
                .enumerate()
                .map(|(index, child)| {
                    if index == 0 {
                        child
                    } else {
                        arkit::row_component()
                            .percent_width(1.0)
                            .margin([gap, 0.0, 0.0, 0.0])
                            .children(vec![child])
                            .into()
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into()
}

pub(crate) fn h_stack(children: Vec<Element>, gap: f32) -> Element {
    arkit::row_component()
        .align_items_center()
        .children(
            children
                .into_iter()
                .enumerate()
                .map(|(index, child)| {
                    if index == 0 {
                        child
                    } else {
                        arkit::row_component()
                            .margin([0.0, 0.0, 0.0, gap])
                            .children(vec![child])
                            .into()
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into()
}

#[allow(dead_code)]
pub(crate) fn page_scroll(children: Vec<Element>) -> Element {
    arkit::scroll_component()
        .percent_width(1.0)
        .layout_weight(1.0_f32)
        .background_color(shadcn::theme::color::SURFACE)
        .children(vec![arkit::column_component()
            .percent_width(1.0)
            .padding([
                0.0,
                shadcn::theme::spacing::LG,
                shadcn::theme::spacing::XXL,
                shadcn::theme::spacing::LG,
            ])
            .children(vec![v_stack(children, shadcn::theme::spacing::LG)])
            .into()])
        .into()
}

pub(crate) fn component_canvas(content: Element, centered: bool, padding: f32) -> Element {
    component_canvas_with(
        content,
        centered,
        centered,
        true,
        [
            padding,
            showcase_horizontal_padding(padding),
            padding,
            showcase_horizontal_padding(padding),
        ],
    )
}

pub(crate) fn component_canvas_with(
    content: Element,
    center_x: bool,
    center_y: bool,
    fill_height: bool,
    padding: [f32; 4],
) -> Element {
    let container = if fill_height {
        arkit::column_component()
            .percent_width(1.0)
            .percent_height(1.0)
    } else {
        arkit::column_component().percent_width(1.0)
    };

    arkit::scroll_component()
        .percent_width(1.0)
        .layout_weight(1.0_f32)
        .background_color(shadcn::theme::color::SURFACE)
        .children(vec![container
            .padding([
                padding[0],
                padding[1],
                padding[2] + shadcn::theme::spacing::XXL,
                padding[3],
            ])
            .align_items_start()
            .justify_content(if center_y {
                JustifyContent::Center
            } else {
                JustifyContent::Start
            })
            .children(vec![canvas_row(content, center_x)])
            .into()])
        .into()
}

pub(crate) fn nav_bar(title: impl Into<String>, back: bool) -> Element {
    let title = title.into();

    if !back {
        return arkit::row_component()
            .percent_width(1.0)
            .height(HOME_HEADER_HEIGHT)
            .background_color(shadcn::theme::color::BACKGROUND)
            .padding([
                18.0,
                shadcn::theme::spacing::LG,
                6.0,
                shadcn::theme::spacing::LG,
            ])
            .align_items_bottom()
            .children(vec![nav_title_text(title, true)])
            .into();
    }

    let left = arkit::row_component()
        .layout_weight(1.0_f32)
        .align_items_center()
        .justify_content_start()
        .children(vec![
            if back {
                plain_back_button()
            } else {
                empty_box(36.0, 36.0)
            },
            arkit::row_component()
                .margin([0.0, 0.0, 0.0, 8.0])
                .children(vec![nav_title_text(title, false)])
                .into(),
        ])
        .into();

    arkit::row_component()
        .percent_width(1.0)
        .height(DETAIL_HEADER_HEIGHT)
        .background_color(shadcn::theme::color::BACKGROUND)
        .padding([
            4.0,
            shadcn::theme::spacing::LG,
            4.0,
            shadcn::theme::spacing::LG,
        ])
        .align_items_center()
        .children(vec![left])
        .into()
}

pub(crate) fn component_list_cell(slug: &str, title: &str, first: bool, last: bool) -> Element {
    let slug = slug.to_string();
    let border_width = if last {
        [1.0, 1.0, 1.0, 1.0]
    } else {
        [1.0, 1.0, 0.0, 1.0]
    };
    let radius = shadcn::theme::radius::LG;
    let border_radius = [
        if first { radius } else { 0.0 },
        if first { radius } else { 0.0 },
        if last { radius } else { 0.0 },
        if last { radius } else { 0.0 },
    ];

    arkit::row_component()
        .percent_width(1.0)
        .height(44.0)
        .align_items_center()
        .justify_content(JustifyContent::SpaceBetween)
        .padding([12.0, 16.0, 12.0, 14.0])
        .border_width(border_width)
        .border_color(shadcn::theme::color::BORDER)
        .border_radius(border_radius)
        .background_color(shadcn::theme::color::CARD)
        .on_press(Message::Navigate(Route::Component { slug }))
        .children(vec![
            arkit::text(title)
                .font_size(shadcn::theme::typography::MD)
                .font_color(shadcn::theme::color::FOREGROUND)
                .font_weight(FontWeight::W400)
                .line_height(20.0)
                .into(),
            lucide::icon("chevron-right")
                .size(16.0)
                .stroke_width(1.5)
                .color(shadcn::theme::color::MUTED_FOREGROUND)
                .render(),
        ])
        .into()
}
