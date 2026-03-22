use super::super::layout::{fixed_width, v_stack, FLEX_ALIGN_CENTER, FLEX_ALIGN_SPACE_BETWEEN};
use super::shared::{top_center_canvas, DemoContext};
use arkit::prelude::*;
use arkit_icon as lucide;
use arkit_shadcn as shadcn;

fn menu_text(content: impl Into<String>, muted: bool) -> Element {
    arkit::text(content)
        .font_size(if muted {
            shadcn::theme::typography::XS
        } else {
            shadcn::theme::typography::SM
        })
        .style(
            ArkUINodeAttributeType::FontColor,
            if muted {
                shadcn::theme::color::MUTED_FOREGROUND
            } else {
                shadcn::theme::color::POPOVER_FOREGROUND
            },
        )
        .style(
            ArkUINodeAttributeType::TextLineHeight,
            if muted { 16.0 } else { 20.0 },
        )
        .into()
}

fn menu_row(
    title: &str,
    shortcut: Option<&str>,
    disabled: bool,
    inset: bool,
    submenu: bool,
) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .height(36.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .style(
            ArkUINodeAttributeType::RowJustifyContent,
            FLEX_ALIGN_SPACE_BETWEEN,
        )
        .style(
            ArkUINodeAttributeType::Padding,
            vec![8.0, 8.0, 8.0, if inset { 32.0 } else { 8.0 }],
        )
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![
                shadcn::theme::radius::SM,
                shadcn::theme::radius::SM,
                shadcn::theme::radius::SM,
                shadcn::theme::radius::SM,
            ],
        )
        .style(ArkUINodeAttributeType::Opacity, if disabled { 0.5_f32 } else { 1.0_f32 })
        .children(vec![
            menu_text(title, false),
            if submenu {
                lucide::icon("chevron-right")
                    .size(16.0)
                    .color(shadcn::theme::color::MUTED_FOREGROUND)
                    .render()
            } else if let Some(shortcut) = shortcut {
                menu_text(shortcut, true)
            } else {
                arkit::row_component().width(0.0).height(0.0).into()
            },
        ])
        .into()
}

fn menu_separator() -> Element {
    arkit::row_component()
        .style(ArkUINodeAttributeType::Margin, vec![4.0, 0.0, 4.0, 0.0])
        .children(vec![shadcn::separator()])
        .into()
}

fn menu_panel() -> Element {
    arkit::column_component()
        .width(192.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![4.0, 4.0, 4.0, 4.0],
        )
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![
                shadcn::theme::radius::MD,
                shadcn::theme::radius::MD,
                shadcn::theme::radius::MD,
                shadcn::theme::radius::MD,
            ],
        )
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![1.0, 1.0, 1.0, 1.0],
        )
        .style(
            ArkUINodeAttributeType::BorderColor,
            vec![shadcn::theme::color::BORDER],
        )
        .style(ArkUINodeAttributeType::Clip, true)
        .style(ArkUINodeAttributeType::Shadow, vec![1_i32])
        .background_color(shadcn::theme::color::POPOVER)
        .children(vec![
            menu_row("New Tab", Some("⌘T"), false, false, false),
            menu_row("New Window", Some("⌘N"), false, false, false),
            menu_row("New Incognito Window", None, true, false, false),
            menu_separator(),
            menu_row("Share", None, false, false, true),
            menu_separator(),
            menu_row("Print...", Some("⌘P"), false, false, false),
        ])
        .into()
}

pub(crate) fn render(_ctx: DemoContext) -> Element {
    top_center_canvas(
        fixed_width(
            v_stack(
                vec![
                    shadcn::menubar(vec![
                        shadcn::menubar_item_active("File"),
                        shadcn::menubar_item("Edit"),
                        shadcn::menubar_item("View"),
                        shadcn::menubar_item("Profiles"),
                    ]),
                    menu_panel(),
                ],
                8.0,
            ),
            384.0,
        ),
        [16.0, 16.0, 16.0, 16.0],
        true,
    )
}
