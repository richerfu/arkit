use arkit::prelude::*;
use arkit_shadcn as shadcn;

use super::constants::component_title;
use super::layout::{
    component_canvas, component_canvas_with, fixed_width, h_stack, nav_bar, v_stack,
    FLEX_ALIGN_CENTER, FLEX_ALIGN_SPACE_BETWEEN,
};

fn top_center_canvas(content: Element, padding: [f32; 4], fill_height: bool) -> Element {
    component_canvas_with(content, true, false, fill_height, padding)
}

fn no_padding_center_canvas(content: Element) -> Element {
    component_canvas_with(content, true, true, true, [0.0, 0.0, 0.0, 0.0])
}

fn top_start_canvas(content: Element, padding: f32) -> Element {
    component_canvas_with(
        content,
        false,
        false,
        true,
        [padding, padding, padding, padding],
    )
}

fn carousel_frame(page: Signal<i32>, count: i32, preview: Element) -> Element {
    let current = page.get().clamp(1, count);
    let prev = page.clone();
    let next = page.clone();

    let prev_button = if current == 1 {
        shadcn::button("‹", shadcn::ButtonVariant::Outline)
            .width(40.0)
            .height(40.0)
            .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
            .style(
                ArkUINodeAttributeType::FontColor,
                shadcn::theme::color::MUTED_FOREGROUND,
            )
    } else {
        shadcn::button("‹", shadcn::ButtonVariant::Outline)
            .width(40.0)
            .height(40.0)
            .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
            .on_click(move || {
                prev.update(|idx| {
                    if *idx > 1 {
                        *idx -= 1;
                    }
                });
            })
    };

    let next_button = if current == count {
        shadcn::button("›", shadcn::ButtonVariant::Outline)
            .width(40.0)
            .height(40.0)
            .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
            .style(
                ArkUINodeAttributeType::FontColor,
                shadcn::theme::color::MUTED_FOREGROUND,
            )
    } else {
        shadcn::button("›", shadcn::ButtonVariant::Outline)
            .width(40.0)
            .height(40.0)
            .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
            .on_click(move || {
                next.update(|idx| {
                    if *idx < count {
                        *idx += 1;
                    }
                });
            })
    };

    arkit::column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .style(
            ArkUINodeAttributeType::ColumnJustifyContent,
            FLEX_ALIGN_SPACE_BETWEEN,
        )
        .children(vec![
            arkit::row_component()
                .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
                .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
                .children(vec![preview])
                .into(),
            arkit::row_component()
                .percent_width(1.0)
                .height(56.0)
                .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
                .children(vec![h_stack(
                    vec![prev_button.into(), next_button.into()],
                    shadcn::theme::spacing::SM,
                )])
                .into(),
        ])
        .into()
}

fn button_preview(variant: &str) -> Element {
    match variant {
        "Destructive" => shadcn::button("Destructive", shadcn::ButtonVariant::Destructive).into(),
        "Ghost" => shadcn::button("Ghost", shadcn::ButtonVariant::Ghost).into(),
        "Link" => shadcn::button("Link", shadcn::ButtonVariant::Link).into(),
        "Loading" => shadcn::button("Please wait", shadcn::ButtonVariant::Default).into(),
        "Outline" => shadcn::button("Outline", shadcn::ButtonVariant::Outline).into(),
        "Secondary" => shadcn::button("Secondary", shadcn::ButtonVariant::Secondary).into(),
        "With Icon" => shadcn::button("Login with Email", shadcn::ButtonVariant::Default).into(),
        "Icon" => shadcn::icon_button("›").into(),
        _ => shadcn::button("Button", shadcn::ButtonVariant::Default).into(),
    }
}

fn button_carousel(page: Signal<i32>) -> Element {
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
    let label = variants[(page.get().clamp(1, count) - 1) as usize];
    carousel_frame(page, count, button_preview(label))
}

fn select_carousel(page: Signal<i32>, choice: Signal<String>) -> Element {
    let count = 2_i32;
    let preview = if page.get().clamp(1, count) == 2 {
        fixed_width(
            shadcn::select(
                vec![
                    String::from("Apple"),
                    String::from("Banana"),
                    String::from("Blueberry"),
                    String::from("Grapes"),
                    String::from("Pineapple"),
                    String::from("Cherry"),
                    String::from("Strawberry"),
                    String::from("Orange"),
                    String::from("Lemon"),
                    String::from("Kiwi"),
                    String::from("Mango"),
                    String::from("Pomegranate"),
                    String::from("Watermelon"),
                    String::from("Peach"),
                ],
                choice,
            ),
            180.0,
        )
    } else {
        fixed_width(
            shadcn::select(
                vec![
                    String::from("Apple"),
                    String::from("Banana"),
                    String::from("Blueberry"),
                    String::from("Grapes"),
                    String::from("Pineapple"),
                ],
                choice,
            ),
            180.0,
        )
    };
    carousel_frame(page, count, preview)
}

fn text_carousel(page: Signal<i32>) -> Element {
    let count = 3_i32;
    let preview = match page.get().clamp(1, count) {
        2 => fixed_width(
            v_stack(
                vec![
                    shadcn::text_variant("The Rainbow Forest Adventure", false),
                    shadcn::text_variant(
                        "Once upon a time, in a magical forest, there lived a curious rabbit named Whiskers.",
                        true,
                    ),
                    shadcn::text_variant("Whiskers' Discovery", false),
                    shadcn::text_variant(
                        "One day, Whiskers found a rainbow-colored flower that transformed the whole forest.",
                        true,
                    ),
                ],
                shadcn::theme::spacing::SM,
            ),
            360.0,
        ),
        3 => v_stack(
            vec![
                shadcn::text_variant("Default: text-foreground", false),
                shadcn::text_variant("Inherited from Parent: text-emerald-500", true),
                shadcn::text_variant("Inherited from NestedParent: text-sky-500", true),
            ],
            shadcn::theme::spacing::SM,
        ),
        _ => shadcn::text_variant("Hello, world!", false),
    };

    carousel_frame(page, count, preview)
}

fn component_body(
    slug: &str,
    active_tab: Signal<usize>,
    page: Signal<i32>,
    choice: Signal<String>,
    query: Signal<String>,
    toggle_state: Signal<bool>,
) -> Element {
    match slug {
        "accordion" => top_center_canvas(
            fixed_width(
                shadcn::accordion(vec![shadcn::accordion_item(
                    "Is it accessible?",
                    toggle_state,
                    vec![shadcn::text_variant(
                        "Yes. It adheres to the WAI-ARIA design pattern.",
                        true,
                    )],
                )]),
                360.0,
            ),
            [0.0, 24.0, 0.0, 24.0],
            false,
        ),
        "alert" => top_center_canvas(
            fixed_width(
                v_stack(
                    vec![
                        shadcn::alert(
                            "Heads up!",
                            "You can add components using the command line.",
                        ),
                        shadcn::alert_destructive(
                            "Error",
                            "Your session has expired. Please sign in again.",
                        ),
                    ],
                    shadcn::theme::spacing::MD,
                ),
                360.0,
            ),
            [24.0, 24.0, 24.0, 24.0],
            false,
        ),
        "alert-dialog" => no_padding_center_canvas(fixed_width(
            shadcn::alert_dialog(
                "Are you absolutely sure?",
                "This action cannot be undone.",
                vec![
                    shadcn::button("Cancel", shadcn::ButtonVariant::Outline).into(),
                    shadcn::button("Continue", shadcn::ButtonVariant::Destructive).into(),
                ],
            ),
            360.0,
        )),
        "aspect-ratio" => component_canvas_with(
            fixed_width(
                shadcn::aspect_ratio(
                    16.0 / 9.0,
                    arkit::row_component()
                        .percent_width(1.0)
                        .percent_height(1.0)
                        .background_color(shadcn::theme::color::MUTED)
                        .style(ArkUINodeAttributeType::RowAlignItems, 2_i32)
                        .style(ArkUINodeAttributeType::RowJustifyContent, 2_i32)
                        .children(vec![shadcn::text_variant("16:9", true)])
                        .into(),
                ),
                320.0,
            ),
            true,
            true,
            true,
            [0.0, 24.0, 0.0, 24.0],
        ),
        "avatar" => component_canvas(
            h_stack(
                vec![
                    shadcn::avatar(None, "CN"),
                    shadcn::avatar(None, "RK"),
                    shadcn::avatar(None, "UI"),
                ],
                shadcn::theme::spacing::LG,
            ),
            true,
            24.0,
        ),
        "badge" => component_canvas(
            fixed_width(
                v_stack(
                    vec![
                        h_stack(
                            vec![
                                shadcn::badge("Badge"),
                                shadcn::badge_with_variant(
                                    "Secondary",
                                    shadcn::BadgeVariant::Secondary,
                                ),
                                shadcn::badge_with_variant(
                                    "Destructive",
                                    shadcn::BadgeVariant::Destructive,
                                ),
                                shadcn::badge_with_variant(
                                    "Outline",
                                    shadcn::BadgeVariant::Outline,
                                ),
                            ],
                            shadcn::theme::spacing::SM,
                        ),
                        h_stack(
                            vec![
                                shadcn::badge_with_variant(
                                    "Verified",
                                    shadcn::BadgeVariant::Secondary,
                                ),
                                shadcn::badge("8"),
                                shadcn::badge_with_variant("99", shadcn::BadgeVariant::Destructive),
                                shadcn::badge_with_variant("20+", shadcn::BadgeVariant::Outline),
                            ],
                            shadcn::theme::spacing::SM,
                        ),
                    ],
                    shadcn::theme::spacing::SM,
                ),
                360.0,
            ),
            true,
            24.0,
        ),
        "button" => no_padding_center_canvas(button_carousel(page)),
        "card" => component_canvas(
            fixed_width(
                shadcn::card(vec![
                    shadcn::card_header("Create project", "Deploy your new project in one-click."),
                    shadcn::card_content(vec![
                        shadcn::form_item(
                            "Name",
                            shadcn::input("shadcn")
                                .bind(query.clone())
                                .percent_width(1.0)
                                .into(),
                        ),
                        shadcn::form_item(
                            "Framework",
                            shadcn::input("React").percent_width(1.0).into(),
                        ),
                    ]),
                    shadcn::card_footer(vec![h_stack(
                        vec![
                            shadcn::button("Cancel", shadcn::ButtonVariant::Outline).into(),
                            shadcn::button("Deploy", shadcn::ButtonVariant::Default).into(),
                        ],
                        shadcn::theme::spacing::SM,
                    )]),
                ]),
                360.0,
            ),
            true,
            24.0,
        ),
        "checkbox" => component_canvas(
            fixed_width(
                shadcn::checkbox("Accept terms and conditions", toggle_state),
                360.0,
            ),
            true,
            32.0,
        ),
        "collapsible" => component_canvas(
            fixed_width(
                shadcn::collapsible(
                    "@peduarte starred 3 repositories",
                    toggle_state,
                    vec![
                        shadcn::text_variant("@radix-ui/primitives", true),
                        shadcn::text_variant("@radix-ui/colors", true),
                    ],
                ),
                360.0,
            ),
            true,
            32.0,
        ),
        "context-menu" => {
            let toggle = toggle_state.clone();
            top_center_canvas(
                fixed_width(
                    shadcn::context_menu(
                        shadcn::button("Open Context Menu", shadcn::ButtonVariant::Outline)
                            .height(36.0)
                            .on_click(move || toggle.update(|open| *open = !*open))
                            .into(),
                        vec![
                            shadcn::dropdown_item("Copy"),
                            shadcn::dropdown_item("Rename"),
                            shadcn::dropdown_item_destructive("Delete"),
                        ],
                        toggle_state,
                    ),
                    360.0,
                ),
                [24.0, 24.0, 24.0, 24.0],
                true,
            )
        }
        "dialog" => {
            let toggle = toggle_state.clone();
            component_canvas(
                fixed_width(
                    v_stack(
                        vec![
                            shadcn::button(
                                if toggle_state.get() {
                                    "Close Dialog"
                                } else {
                                    "Open Dialog"
                                },
                                shadcn::ButtonVariant::Outline,
                            )
                            .on_click(move || toggle.update(|open| *open = !*open))
                            .into(),
                            shadcn::dialog(
                                "Edit profile",
                                toggle_state,
                                vec![
                                    shadcn::form_item(
                                        "Name",
                                        shadcn::input("Pedro Duarte")
                                            .bind(query)
                                            .percent_width(1.0)
                                            .into(),
                                    ),
                                    shadcn::dialog_footer(vec![shadcn::button(
                                        "Save changes",
                                        shadcn::ButtonVariant::Default,
                                    )
                                    .into()]),
                                ],
                            ),
                        ],
                        shadcn::theme::spacing::SM,
                    ),
                    360.0,
                ),
                true,
                24.0,
            )
        }
        "dropdown-menu" => {
            let toggle = toggle_state.clone();
            top_center_canvas(
                fixed_width(
                    shadcn::dropdown_menu(
                        shadcn::button("Open Menu", shadcn::ButtonVariant::Outline)
                            .height(36.0)
                            .on_click(move || toggle.update(|open| *open = !*open))
                            .into(),
                        vec![
                            shadcn::dropdown_item("Profile"),
                            shadcn::dropdown_item("Billing"),
                            shadcn::dropdown_item("Settings"),
                            shadcn::dropdown_item_destructive("Log out"),
                        ],
                        toggle_state,
                    ),
                    360.0,
                ),
                [24.0, 24.0, 24.0, 24.0],
                true,
            )
        }
        "hover-card" => {
            let toggle = toggle_state.clone();
            component_canvas(
                fixed_width(
                    v_stack(
                        vec![
                            shadcn::button(
                                if toggle_state.get() {
                                    "Hide Hover Card"
                                } else {
                                    "Show Hover Card"
                                },
                                shadcn::ButtonVariant::Outline,
                            )
                            .on_click(move || toggle.update(|show| *show = !*show))
                            .into(),
                            shadcn::hover_card(
                                shadcn::button("@peduarte", shadcn::ButtonVariant::Ghost).into(),
                                vec![
                                    shadcn::card_title("@peduarte"),
                                    shadcn::card_description("Joined December 2021"),
                                ],
                                toggle_state.get(),
                            ),
                        ],
                        shadcn::theme::spacing::SM,
                    ),
                    360.0,
                ),
                true,
                24.0,
            )
        }
        "input" => component_canvas(
            fixed_width(
                shadcn::input("Email").bind(query).percent_width(1.0).into(),
                360.0,
            ),
            true,
            24.0,
        ),
        "label" => component_canvas(
            fixed_width(
                v_stack(
                    vec![
                        shadcn::label("Accept terms and conditions").into(),
                        shadcn::checkbox("Terms", toggle_state),
                    ],
                    shadcn::theme::spacing::SM,
                ),
                360.0,
            ),
            true,
            24.0,
        ),
        "menubar" => top_center_canvas(
            fixed_width(
                shadcn::menubar(vec![
                    shadcn::menubar_item_active("File"),
                    shadcn::menubar_item("Edit"),
                    shadcn::menubar_item("View"),
                    shadcn::menubar_item("Profile"),
                ]),
                360.0,
            ),
            [16.0, 16.0, 16.0, 16.0],
            true,
        ),
        "popover" => {
            let toggle = toggle_state.clone();
            component_canvas(
                fixed_width(
                    shadcn::popover(
                        shadcn::button("Open popover", shadcn::ButtonVariant::Outline)
                            .on_click(move || toggle.update(|open| *open = !*open))
                            .into(),
                        vec![
                            shadcn::card_title("Dimensions"),
                            shadcn::form_item(
                                "Width",
                                shadcn::input("100%").bind(query).percent_width(1.0).into(),
                            ),
                        ],
                        toggle_state,
                    ),
                    360.0,
                ),
                true,
                24.0,
            )
        }
        "progress" => component_canvas(
            fixed_width(
                shadcn::progress((page.get().max(1) as f32 * 10.0).min(100.0), 100.0).into(),
                280.0,
            ),
            true,
            24.0,
        ),
        "radio-group" => component_canvas(
            fixed_width(
                shadcn::radio_group(
                    vec![
                        String::from("Default"),
                        String::from("Comfortable"),
                        String::from("Compact"),
                    ],
                    choice,
                ),
                360.0,
            ),
            true,
            24.0,
        ),
        "select" => no_padding_center_canvas(select_carousel(page, choice)),
        "separator" => component_canvas(
            fixed_width(
                v_stack(
                    vec![
                        shadcn::text_variant("Radix Primitives", false),
                        shadcn::separator(),
                        shadcn::text_variant("An open-source UI component library.", true),
                    ],
                    shadcn::theme::spacing::SM,
                ),
                320.0,
            ),
            true,
            24.0,
        ),
        "skeleton" => no_padding_center_canvas(fixed_width(
            v_stack(
                vec![shadcn::skeleton(250.0, 18.0), shadcn::skeleton(180.0, 18.0)],
                shadcn::theme::spacing::SM,
            ),
            250.0,
        )),
        "switch" => component_canvas(
            fixed_width(
                h_stack(
                    vec![
                        shadcn::label("Airplane Mode").into(),
                        shadcn::switch(toggle_state).into(),
                    ],
                    shadcn::theme::spacing::SM,
                ),
                360.0,
            ),
            true,
            24.0,
        ),
        "tabs" => top_start_canvas(
            shadcn::tabs(
                vec![String::from("Feedback"), String::from("Survey")],
                active_tab,
                vec![
                    shadcn::card(vec![
                        shadcn::card_header(
                            "Feedback",
                            "Share your thoughts with us. Click submit when you are ready.",
                        ),
                        shadcn::card_content(vec![
                            shadcn::form_item(
                                "Name",
                                shadcn::input("Michael Scott")
                                    .bind(query.clone())
                                    .percent_width(1.0)
                                    .into(),
                            ),
                            shadcn::form_item(
                                "Message",
                                shadcn::input("Where are the turtles?!")
                                    .percent_width(1.0)
                                    .into(),
                            ),
                        ]),
                        shadcn::card_footer(vec![shadcn::button(
                            "Submit feedback",
                            shadcn::ButtonVariant::Default,
                        )
                        .into()]),
                    ]),
                    shadcn::card(vec![
                        shadcn::card_header(
                            "Quick Survey",
                            "Answer a few quick questions to help improve the demo.",
                        ),
                        shadcn::card_content(vec![
                            shadcn::form_item(
                                "Job Title",
                                shadcn::input("Regional Manager").percent_width(1.0).into(),
                            ),
                            shadcn::form_item(
                                "Favorite feature",
                                shadcn::input("CLI").percent_width(1.0).into(),
                            ),
                        ]),
                        shadcn::card_footer(vec![shadcn::button(
                            "Submit survey",
                            shadcn::ButtonVariant::Default,
                        )
                        .into()]),
                    ]),
                ],
            ),
            24.0,
        ),
        "text" => no_padding_center_canvas(text_carousel(page)),
        "textarea" => component_canvas(
            fixed_width(
                shadcn::textarea("Type your message here...")
                    .percent_width(1.0)
                    .into(),
                360.0,
            ),
            true,
            24.0,
        ),
        "toggle" => component_canvas(shadcn::toggle("Bold", toggle_state).into(), true, 24.0),
        "toggle-group" => component_canvas(
            shadcn::toggle_group(
                vec![
                    String::from("Bold"),
                    String::from("Italic"),
                    String::from("Underline"),
                ],
                choice,
            ),
            true,
            24.0,
        ),
        "tooltip" => component_canvas(shadcn::tooltip("Hover", "Add to library"), true, 24.0),
        "table" => component_canvas(
            shadcn::table(
                vec![
                    String::from("Name"),
                    String::from("Role"),
                    String::from("Status"),
                ],
                vec![
                    vec![
                        String::from("Alice"),
                        String::from("Designer"),
                        String::from("Active"),
                    ],
                    vec![
                        String::from("Bob"),
                        String::from("Engineer"),
                        String::from("Idle"),
                    ],
                    vec![
                        String::from("Carol"),
                        String::from("PM"),
                        String::from("Active"),
                    ],
                ],
            ),
            true,
            24.0,
        ),
        _ => component_canvas(
            shadcn::card(vec![
                shadcn::card_title("Route Not Found"),
                shadcn::card_description("Please return to list and retry."),
            ]),
            true,
            24.0,
        ),
    }
}

#[component]
pub(crate) fn component_page(
    name: String,
    active_tab: Signal<usize>,
    page: Signal<i32>,
    choice: Signal<String>,
    query: Signal<String>,
    toggle_state: Signal<bool>,
) -> Element {
    let title = component_title(&name);

    arkit::column(vec![
        nav_bar(title, true),
        component_body(&name, active_tab, page, choice, query, toggle_state),
    ])
}
