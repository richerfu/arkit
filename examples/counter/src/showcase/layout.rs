use arkit::prelude::*;
use arkit_icon as lucide;
use arkit_shadcn as shadcn;

pub(crate) const FLEX_ALIGN_CENTER: i32 = 2;
pub(crate) const FLEX_ALIGN_END: i32 = 3;
pub(crate) const FLEX_ALIGN_SPACE_BETWEEN: i32 = 6;
pub(crate) const FLEX_ALIGN_START: i32 = 1;
const HOME_HEADER_HEIGHT: f32 = 68.0;
const DETAIL_HEADER_HEIGHT: f32 = 52.0;

fn empty_box(width: f32, height: f32) -> Element {
    arkit::row_component().width(width).height(height).into()
}

fn constrained_width(child: Element, width: f32) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .align_items_center()
        .children(vec![arkit::row_component()
            .percent_width(1.0)
            .max_width_constraint(width)
            .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
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
        .style(
            ArkUINodeAttributeType::RowJustifyContent,
            if center_x {
                FLEX_ALIGN_CENTER
            } else {
                FLEX_ALIGN_START
            },
        )
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
                            .style(ArkUINodeAttributeType::Margin, vec![gap, 0.0, 0.0, 0.0])
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
                            .style(ArkUINodeAttributeType::Margin, vec![0.0, 0.0, 0.0, gap])
                            .children(vec![child])
                            .into()
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into()
}

pub(crate) fn page_scroll(children: Vec<Element>) -> Element {
    arkit::scroll_component()
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
        .background_color(shadcn::theme::color::SURFACE)
        .children(vec![arkit::column_component()
            .percent_width(1.0)
            .style(
                ArkUINodeAttributeType::Padding,
                vec![
                    0.0,
                    shadcn::theme::spacing::LG,
                    shadcn::theme::spacing::XXL,
                    shadcn::theme::spacing::LG,
                ],
            )
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
        .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
        .background_color(shadcn::theme::color::SURFACE)
        .children(vec![container
            .style(
                ArkUINodeAttributeType::Padding,
                vec![
                    padding[0],
                    padding[1],
                    padding[2] + shadcn::theme::spacing::XXL,
                    padding[3],
                ],
            )
            .align_items_start()
            .style(
                ArkUINodeAttributeType::ColumnJustifyContent,
                if center_y {
                    FLEX_ALIGN_CENTER
                } else {
                    FLEX_ALIGN_START
                },
            )
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
            .style(
                ArkUINodeAttributeType::Padding,
                vec![
                    12.0,
                    shadcn::theme::spacing::LG,
                    8.0,
                    shadcn::theme::spacing::LG,
                ],
            )
            .align_items_bottom()
            .children(vec![arkit::text(title)
                .font_size(30.0)
                .style(
                    ArkUINodeAttributeType::FontColor,
                    shadcn::theme::color::FOREGROUND,
                )
                .style(ArkUINodeAttributeType::FontWeight, 5_i32)
                .style(ArkUINodeAttributeType::TextLineHeight, 36.0)
                .into()])
            .into();
    }

    let left = arkit::row_component()
        .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
        .align_items_center()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_START)
        .children(vec![if back {
            shadcn::icon_button_with_variant("chevron-left", shadcn::ButtonVariant::Ghost)
                .width(36.0)
                .height(36.0)
                .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
                .on_click(|| {
                    let _ = back_route();
                })
                .into()
        } else {
            empty_box(36.0, 36.0)
        }])
        .into();

    let right = arkit::row_component()
        .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
        .align_items_center()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_END)
        .children(vec![empty_box(36.0, 36.0)])
        .into();

    let center = arkit::row_component()
        .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
        .align_items_center()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
        .children(vec![arkit::text(title)
            .font_size(shadcn::theme::typography::XL)
            .style(
                ArkUINodeAttributeType::FontColor,
                shadcn::theme::color::FOREGROUND,
            )
            .style(ArkUINodeAttributeType::FontWeight, 4_i32)
            .style(ArkUINodeAttributeType::TextLineHeight, 24.0)
            .into()])
        .into();

    arkit::row_component()
        .percent_width(1.0)
        .height(DETAIL_HEADER_HEIGHT)
        .background_color(shadcn::theme::color::BACKGROUND)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![
                6.0,
                shadcn::theme::spacing::LG,
                6.0,
                shadcn::theme::spacing::LG,
            ],
        )
        .align_items_center()
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![0.0, 0.0, 1.0, 0.0],
        )
        .style(
            ArkUINodeAttributeType::BorderColor,
            vec![shadcn::theme::color::BORDER],
        )
        .children(vec![left, center, right])
        .into()
}

pub(crate) fn component_list_cell(slug: &str, title: &str, first: bool, last: bool) -> Element {
    let path = format!("/components/{slug}");
    let border_width = if last {
        vec![1.0, 1.0, 1.0, 1.0]
    } else {
        vec![1.0, 1.0, 0.0, 1.0]
    };
    let radius = shadcn::theme::radius::LG;
    let border_radius = vec![
        if first { radius } else { 0.0 },
        if first { radius } else { 0.0 },
        if last { radius } else { 0.0 },
        if last { radius } else { 0.0 },
    ];

    arkit::row_component()
        .percent_width(1.0)
        .height(44.0)
        .align_items_center()
        .style(
            ArkUINodeAttributeType::RowJustifyContent,
            FLEX_ALIGN_SPACE_BETWEEN,
        )
        .style(
            ArkUINodeAttributeType::Padding,
            vec![12.0, 16.0, 12.0, 14.0],
        )
        .style(ArkUINodeAttributeType::BorderWidth, border_width)
        .style(
            ArkUINodeAttributeType::BorderColor,
            vec![shadcn::theme::color::BORDER],
        )
        .style(ArkUINodeAttributeType::BorderRadius, border_radius)
        .background_color(shadcn::theme::color::CARD)
        .on_click(move || {
            let _ = push_route(path.clone());
        })
        .children(vec![
            arkit::text(title)
                .font_size(shadcn::theme::typography::MD)
                .style(
                    ArkUINodeAttributeType::FontColor,
                    shadcn::theme::color::FOREGROUND,
                )
                .style(ArkUINodeAttributeType::FontWeight, 3_i32)
                .style(ArkUINodeAttributeType::TextLineHeight, 24.0)
                .into(),
            lucide::icon("chevron-right")
                .size(16.0)
                .stroke_width(1.5)
                .color(shadcn::theme::color::MUTED_FOREGROUND)
                .render(),
        ])
        .into()
}
