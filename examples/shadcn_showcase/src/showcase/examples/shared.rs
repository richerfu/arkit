use crate::prelude::*;
use arkit_icon as lucide;
use arkit_shadcn as shadcn;

use super::super::layout::{component_canvas_with, fixed_width, h_stack, v_stack};
use crate::Message;

const EMERALD_500: u32 = 0xFF10B981;
const PURPLE_500: u32 = 0xFFA855F7;
const SKY_500: u32 = 0xFF0EA5E9;

#[derive(Clone)]
pub(crate) struct DemoContext {
    pub active_tab: usize,
    pub page: i32,
    pub button_preview_feedback: Option<String>,
    pub radio_choice: String,
    pub select_choice: String,
    pub query: String,
    pub toggle_state: bool,
    pub context_menu_open: bool,
    pub dropdown_menu_open: bool,
    pub popover_open: bool,
    pub tooltip_open: bool,
    pub select_open: bool,
    pub accordion_single_open: Option<String>,
    pub context_bookmarks: bool,
    pub context_full_urls: bool,
    pub context_person: String,
    pub checkbox_first: bool,
    pub checkbox_second: bool,
    pub checkbox_card: bool,
    pub toggle_group_values: Vec<String>,
    pub menubar_active: Option<usize>,
}

fn showcase_horizontal_padding(value: f32) -> f32 {
    value
}

pub(crate) fn top_center_canvas(content: Element, padding: [f32; 4], fill_height: bool) -> Element {
    component_canvas_with(
        content,
        true,
        false,
        fill_height,
        [
            padding[0],
            showcase_horizontal_padding(padding[1]),
            padding[2],
            showcase_horizontal_padding(padding[3]),
        ],
    )
}

pub(crate) fn no_padding_center_canvas(content: Element) -> Element {
    component_canvas_with(content, true, true, true, [0.0, 0.0, 0.0, 0.0])
}

pub(crate) fn top_start_canvas(content: Element, padding: f32) -> Element {
    component_canvas_with(
        content,
        false,
        false,
        true,
        [
            padding,
            showcase_horizontal_padding(padding),
            padding,
            showcase_horizontal_padding(padding),
        ],
    )
}

fn carousel_nav_surface(child: Element, disabled: bool) -> Element {
    // In the RN demo the icon button looks like: `h-10 w-10 rounded-md border bg-background
    // shadow-sm ...`. Keep the border/radius on the wrapper so the corners stay stable across
    // press + re-render (ArkUI may re-apply default button styles after interaction).
    let radius = shadcn::theme::radius::MD;
    arkit::row_component()
        .width(40.0)
        .height(40.0)
        .border_radius([radius, radius, radius, radius])
        .shadow(ShadowStyle::OuterDefaultSm)
        .opacity(if disabled { 0.5_f32 } else { 1.0_f32 })
        .children(vec![arkit::row_component()
            .width(40.0)
            .height(40.0)
            .align_items_center()
            .justify_content_center()
            .background_color(shadcn::theme::color::BACKGROUND)
            .border_style(BorderStyle::Solid)
            .border_width([1.0, 1.0, 1.0, 1.0])
            .border_color(shadcn::theme::color::BORDER)
            .border_radius([radius, radius, radius, radius])
            .clip(true)
            .children(vec![child])
            .into()])
        .into()
}

pub(crate) fn carousel_frame(
    page: i32,
    count: i32,
    preview: Element,
    remove_bottom_safe_area: bool,
) -> Element {
    let current = page.clamp(1, count);

    let prev_disabled = current == 1;
    let prev_button = if prev_disabled {
        shadcn::icon_button("chevron-left")
            .theme(shadcn::ButtonVariant::Ghost)
            .key(format!("carousel-prev:{current}:disabled"))
            .width(40.0)
            .height(40.0)
            .padding(arkit::Padding::ZERO)
            .disabled(true)
    } else {
        shadcn::icon_button("chevron-left")
            .theme(shadcn::ButtonVariant::Ghost)
            .key(format!("carousel-prev:{current}:enabled"))
            .width(40.0)
            .height(40.0)
            .padding(arkit::Padding::ZERO)
            .on_press(Message::SetPage((current - 1).max(1)))
    };

    let next_disabled = current == count;
    let next_button = if next_disabled {
        shadcn::icon_button("chevron-right")
            .theme(shadcn::ButtonVariant::Ghost)
            .key(format!("carousel-next:{current}:disabled"))
            .width(40.0)
            .height(40.0)
            .padding(arkit::Padding::ZERO)
            .disabled(true)
    } else {
        shadcn::icon_button("chevron-right")
            .theme(shadcn::ButtonVariant::Ghost)
            .key(format!("carousel-next:{current}:enabled"))
            .width(40.0)
            .height(40.0)
            .padding(arkit::Padding::ZERO)
            .on_press(Message::SetPage((current + 1).min(count)))
    };

    let mut preview_area = arkit::row_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .align_items_center()
        .justify_content_center()
        .children(vec![preview]);

    if !remove_bottom_safe_area {
        preview_area = preview_area.padding([0.0, 0.0, 48.0 + shadcn::theme::spacing::LG, 0.0]);
    }

    let nav_bar = arkit::column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .justify_content_end()
        .align_items_center()
        .children(vec![arkit::row_component()
            .percent_width(1.0)
            .height(48.0)
            .margin([0.0, 0.0, shadcn::theme::spacing::LG, 0.0])
            .padding([0.0, 16.0, 0.0, 16.0])
            .align_items_center()
            .justify_content_center()
            .children(vec![h_stack(
                vec![
                    carousel_nav_surface(prev_button.into(), prev_disabled),
                    carousel_nav_surface(next_button.into(), next_disabled),
                ],
                shadcn::theme::spacing::SM,
            )])
            .into()])
        .into();

    arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![preview_area.into(), nav_bar])
        .into()
}

pub(crate) fn carousel_frame_fn(
    page: i32,
    count: i32,
    render_preview: impl Fn(i32) -> Element,
    remove_bottom_safe_area: bool,
) -> Element {
    let current = page.clamp(1, count);

    let mut preview_area = arkit::row_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .align_items_center()
        .justify_content_center()
        .children(vec![render_preview(current)]);

    if !remove_bottom_safe_area {
        preview_area = preview_area.padding([0.0, 0.0, 48.0 + shadcn::theme::spacing::LG, 0.0]);
    }

    let nav_bar = arkit::column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .justify_content_end()
        .align_items_center()
        .children(vec![arkit::row_component()
            .percent_width(1.0)
            .height(48.0)
            .margin([0.0, 0.0, shadcn::theme::spacing::LG, 0.0])
            .padding([0.0, 16.0, 0.0, 16.0])
            .align_items_center()
            .justify_content_center()
            .children(vec![{
                let prev_disabled = current == 1;
                let next_disabled = current == count;
                let prev_button = if prev_disabled {
                    shadcn::icon_button("chevron-left")
                        .theme(shadcn::ButtonVariant::Ghost)
                        .key(format!("carousel-prev:{current}:disabled"))
                        .width(40.0)
                        .height(40.0)
                        .padding(arkit::Padding::ZERO)
                        .disabled(true)
                } else {
                    shadcn::icon_button("chevron-left")
                        .theme(shadcn::ButtonVariant::Ghost)
                        .key(format!("carousel-prev:{current}:enabled"))
                        .width(40.0)
                        .height(40.0)
                        .padding(arkit::Padding::ZERO)
                        .on_press(Message::SetPage((current - 1).max(1)))
                };
                let next_button = if next_disabled {
                    shadcn::icon_button("chevron-right")
                        .theme(shadcn::ButtonVariant::Ghost)
                        .key(format!("carousel-next:{current}:disabled"))
                        .width(40.0)
                        .height(40.0)
                        .padding(arkit::Padding::ZERO)
                        .disabled(true)
                } else {
                    shadcn::icon_button("chevron-right")
                        .theme(shadcn::ButtonVariant::Ghost)
                        .key(format!("carousel-next:{current}:enabled"))
                        .width(40.0)
                        .height(40.0)
                        .padding(arkit::Padding::ZERO)
                        .on_press(Message::SetPage((current + 1).min(count)))
                };
                h_stack(
                    vec![
                        carousel_nav_surface(prev_button.into(), prev_disabled),
                        carousel_nav_surface(next_button.into(), next_disabled),
                    ],
                    shadcn::theme::spacing::SM,
                )
            }])
            .into()])
        .into();

    arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![preview_area.into(), nav_bar])
        .into()
}

fn button_preview(variant: &str) -> Element {
    let preview_press = || Message::ButtonPreviewPressed(variant.to_string());
    match variant {
        "Destructive" => shadcn::button("Destructive")
            .theme(shadcn::ButtonVariant::Destructive)
            .on_press(preview_press())
            .into(),
        "Ghost" => shadcn::button("Ghost")
            .theme(shadcn::ButtonVariant::Ghost)
            .on_press(preview_press())
            .into(),
        "Link" => shadcn::button("Link")
            .theme(shadcn::ButtonVariant::Link)
            .on_press(preview_press())
            .into(),
        "Loading" => shadcn::button_with_icon("Please wait", "loader")
            .theme(shadcn::ButtonVariant::Default)
            .disabled(true)
            .into(),
        "Outline" => shadcn::button("Outline")
            .theme(shadcn::ButtonVariant::Outline)
            .on_press(preview_press())
            .into(),
        "Secondary" => shadcn::button("Secondary")
            .theme(shadcn::ButtonVariant::Secondary)
            .on_press(preview_press())
            .into(),
        "With Icon" => shadcn::button_with_icon("Login with Email", "mail")
            .theme(shadcn::ButtonVariant::Default)
            .on_press(preview_press())
            .into(),
        "Icon" => shadcn::icon_button("chevron-right")
            .theme(shadcn::ButtonVariant::Outline)
            .on_press(preview_press())
            .into(),
        _ => shadcn::button("Button")
            .theme(shadcn::ButtonVariant::Default)
            .on_press(preview_press())
            .into(),
    }
}

pub(crate) fn button_carousel(page: i32) -> Element {
    let variants = [
        "Default",
        "Destructive",
        "Ghost",
        "Link",
        "Loading",
        "Outline",
        "Secondary",
        "With Icon",
        "Icon",
    ];
    let count = variants.len() as i32;
    carousel_frame_fn(
        page,
        count,
        move |current| {
            let label = variants[(current - 1) as usize];
            arkit::row_component()
                .key(format!("button-preview:{label}"))
                .children(vec![button_preview(label)])
                .into()
        },
        false,
    )
}

fn icon_tile(name: &str, icon: Element) -> Element {
    arkit::column_component()
        .width(96.0)
        .align_items_center()
        .children(vec![
            arkit::row_component()
                .width(48.0)
                .height(48.0)
                .align_items_center()
                .justify_content_center()
                .border_radius([
                    shadcn::theme::radius::MD,
                    shadcn::theme::radius::MD,
                    shadcn::theme::radius::MD,
                    shadcn::theme::radius::MD,
                ])
                .border_width([1.0, 1.0, 1.0, 1.0])
                .border_color(shadcn::theme::color::BORDER)
                .background_color(shadcn::theme::color::BACKGROUND)
                .children(vec![icon])
                .into(),
            arkit::row_component()
                .margin([8.0, 0.0, 0.0, 0.0])
                .children(vec![shadcn::text_with_variant(
                    name,
                    shadcn::TextVariant::Small,
                )])
                .into(),
        ])
        .into()
}

pub(crate) fn icon_showcase() -> Element {
    fixed_width(
        v_stack(
            vec![
                h_stack(
                    vec![
                        icon_tile(
                            "mail",
                            lucide::icon("mail")
                                .size(20.0)
                                .color(shadcn::theme::color::FOREGROUND)
                                .render(),
                        ),
                        icon_tile(
                            "chevron-right",
                            lucide::icon("chevron-right")
                                .size(20.0)
                                .color(shadcn::theme::color::FOREGROUND)
                                .render(),
                        ),
                        icon_tile(
                            "search",
                            lucide::icon("search")
                                .size(20.0)
                                .color(shadcn::theme::color::FOREGROUND)
                                .render(),
                        ),
                    ],
                    shadcn::theme::spacing::MD,
                ),
                h_stack(
                    vec![
                        icon_tile(
                            "bell-off",
                            lucide::icon("bell-off")
                                .size(20.0)
                                .color(0xFFEF4444)
                                .render(),
                        ),
                        icon_tile(
                            "star",
                            lucide::icon("star").size(24.0).color(0xFFF59E0B).render(),
                        ),
                        icon_tile(
                            "settings-2",
                            lucide::icon("settings-2")
                                .size(20.0)
                                .color(shadcn::theme::color::FOREGROUND)
                                .render(),
                        ),
                    ],
                    shadcn::theme::spacing::MD,
                ),
            ],
            shadcn::theme::spacing::LG,
        ),
        352.0,
    )
}

pub(crate) fn select_carousel(page: i32, selected: String, open: bool) -> Element {
    let default_items = vec!["Apple", "Banana", "Blueberry", "Grapes", "Pineapple"];
    let scrollable_items = vec![
        "Apple",
        "Banana",
        "Blueberry",
        "Grapes",
        "Pineapple",
        "Cherry",
        "Strawberry",
        "Orange",
        "Lemon",
        "Kiwi",
        "Mango",
        "Pomegranate",
        "Watermelon",
        "Peach",
        "Pear",
        "Plum",
        "Raspberry",
        "Tangerine",
    ];

    let count = 2;
    let options = if page.clamp(1, count) == 2 {
        scrollable_items
    } else {
        default_items
    }
    .into_iter()
    .map(|item| item.to_string())
    .collect::<Vec<_>>();

    carousel_frame(
        page,
        count,
        fixed_width(
            shadcn::select(
                options,
                selected,
                open,
                Message::SetSelectOpen,
                Message::SetSelectChoice,
            ),
            180.0,
        ),
        false,
    )
}

pub(crate) fn text_carousel(page: i32) -> Element {
    let count = 3;
    let preview = match page.clamp(1, count) {
        2 => {
            fn spacer(height: f32) -> Element {
                arkit::row_component()
                    .percent_width(1.0)
                    .height(height)
                    .into()
            }

            let content = arkit::column_component()
                .percent_width(1.0)
                .children(vec![
                    shadcn::text_with_variant(
                        "The Rainbow Forest Adventure",
                        shadcn::TextVariant::H1,
                    ),
                    spacer(12.0),
                    shadcn::text_with_variant(
                        "Once upon a time, in a magical forest, there lived a curious rabbit named Whiskers. Whiskers loved exploring and discovering new things every day.",
                        shadcn::TextVariant::P,
                    ),
                    spacer(24.0),
                    shadcn::text_with_variant("Whiskers' Discovery", shadcn::TextVariant::H2),
                    shadcn::text_with_variant(
                        "One day, while hopping through the forest, Whiskers stumbled upon a mysterious rainbow-colored flower. The flower had the power to make the forest come alive with vibrant colors and happy creatures.",
                        shadcn::TextVariant::P,
                    ),
                    shadcn::text_with_variant(
                        "\"Oh, what a wonderful discovery!\" exclaimed Whiskers. \"I must share this magic with all my forest friends!\"",
                        shadcn::TextVariant::Blockquote,
                    ),
                    spacer(32.0),
                    shadcn::text_with_variant(
                        "The Colorful Transformation",
                        shadcn::TextVariant::H3,
                    ),
                    spacer(4.0),
                    shadcn::text_with_variant(
                        "Whiskers excitedly gathered all the animals in the forest and showed them the magical rainbow flower. The animals were amazed and decided to plant more of these flowers to make their home even more magical.",
                        shadcn::TextVariant::P,
                    ),
                    spacer(12.0),
                    shadcn::text_with_variant(
                        "As the rainbow flowers bloomed, the entire forest transformed into a kaleidoscope of colors. Birds chirped in harmony, butterflies danced in the air, and even the trees swayed to the rhythm of the wind.",
                        shadcn::TextVariant::P,
                    ),
                    spacer(24.0),
                    shadcn::text_with_variant(
                        "The Enchanted Celebration",
                        shadcn::TextVariant::H3,
                    ),
                    spacer(4.0),
                    shadcn::text_with_variant(
                        "The animals decided to celebrate their enchanted forest with a grand feast. They gathered nuts, berries, and fruits from the colorful trees and shared stories of their adventures. The joyous laughter echoed through the Rainbow Forest.",
                        shadcn::TextVariant::P,
                    ),
                    spacer(12.0),
                    shadcn::text_with_variant(
                        "And so, the Rainbow Forest became a place of wonder and happiness, where Whiskers and all the animals lived together in harmony.",
                        shadcn::TextVariant::Lead,
                    ),
                    spacer(24.0),
                    shadcn::text_with_variant(
                        "The Never-ending Magic",
                        shadcn::TextVariant::H3,
                    ),
                    spacer(4.0),
                    shadcn::text_with_variant(
                        "The magic of the rainbow flowers continued to spread, reaching other parts of the world. Soon, forests everywhere became vibrant and alive, thanks to the discovery of Whiskers and the enchanted Rainbow Forest.",
                        shadcn::TextVariant::P,
                    ),
                    spacer(12.0),
                    shadcn::text_with_variant(
                        "The moral of the story is: embrace the magic of discovery, share joy with others, and watch as the world transforms into a colorful and beautiful place.",
                        shadcn::TextVariant::Large,
                    ),
                    spacer(24.0),
                ])
                .into();

            fixed_width(
                arkit::scroll_component()
                    .percent_width(1.0)
                    .percent_height(1.0)
                    .children(vec![arkit::column_component()
                        .percent_width(1.0)
                        .padding([24.0, 24.0, 72.0, 24.0])
                        .children(vec![content])
                        .into()])
                    .into(),
                512.0,
            )
        }
        3 => {
            fn colored_body(content: impl Into<String>, color: u32) -> Element {
                arkit::text(content)
                    .font_size(shadcn::theme::typography::MD)
                    .font_color(color)
                    .line_height(24.0)
                    .into()
            }

            fn code_chip(content: impl Into<String>, color: u32) -> Element {
                arkit::row_component()
                    .background_color(shadcn::theme::color::MUTED)
                    .border_radius([
                        shadcn::theme::radius::SM,
                        shadcn::theme::radius::SM,
                        shadcn::theme::radius::SM,
                        shadcn::theme::radius::SM,
                    ])
                    .padding([3.0, 5.0, 3.0, 5.0])
                    .children(vec![arkit::text(content)
                        .font_size(shadcn::theme::typography::SM)
                        .font_family("monospace")
                        .font_weight(FontWeight::W600)
                        .font_color(color)
                        .line_height(18.0)
                        .into()])
                    .into()
            }

            fn v_stack_center(children: Vec<Element>, gap: f32) -> Element {
                arkit::column_component()
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
                                        .margin([gap, 0.0, 0.0, 0.0])
                                        .children(vec![child])
                                        .into()
                                }
                            })
                            .collect::<Vec<_>>(),
                    )
                    .into()
            }

            fixed_width(
                v_stack_center(
                    vec![
                        h_stack(
                            vec![
                                shadcn::text("Default:"),
                                shadcn::text_with_variant(
                                    "text-foreground",
                                    shadcn::TextVariant::Code,
                                ),
                            ],
                            4.0,
                        ),
                        v_stack_center(
                            vec![
                                h_stack(
                                    vec![
                                        colored_body("Inherited from Parent:", EMERALD_500),
                                        code_chip("text-emerald-500", EMERALD_500),
                                    ],
                                    4.0,
                                ),
                                h_stack(
                                    vec![
                                        colored_body("Overridden:", PURPLE_500),
                                        code_chip("text-purple-500", PURPLE_500),
                                    ],
                                    4.0,
                                ),
                                h_stack(
                                    vec![
                                        colored_body("Inherited from NestedParent:", SKY_500),
                                        code_chip("text-sky-500", SKY_500),
                                    ],
                                    4.0,
                                ),
                            ],
                            8.0,
                        ),
                    ],
                    8.0,
                ),
                352.0,
            )
        }
        _ => shadcn::text("Hello, world!"),
    };

    carousel_frame(page, count, preview, true)
}
