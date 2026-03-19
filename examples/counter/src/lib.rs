use arkit::prelude::*;
use arkit_shadcn as shadcn;

const FLEX_ALIGN_CENTER: i32 = 2;
const FLEX_ALIGN_SPACE_BETWEEN: i32 = 6;

const SHOWCASE_COMPONENTS: [(&str, &str); 31] = [
    ("accordion", "Accordion"),
    ("alert", "Alert"),
    ("alert-dialog", "Alert Dialog"),
    ("aspect-ratio", "Aspect Ratio"),
    ("avatar", "Avatar"),
    ("badge", "Badge"),
    ("button", "Button"),
    ("card", "Card"),
    ("checkbox", "Checkbox"),
    ("collapsible", "Collapsible"),
    ("context-menu", "Context Menu"),
    ("dialog", "Dialog"),
    ("dropdown-menu", "Dropdown Menu"),
    ("hover-card", "Hover Card"),
    ("input", "Input"),
    ("label", "Label"),
    ("menubar", "Menubar"),
    ("popover", "Popover"),
    ("progress", "Progress"),
    ("radio-group", "Radio Group"),
    ("select", "Select"),
    ("separator", "Separator"),
    ("skeleton", "Skeleton"),
    ("switch", "Switch"),
    ("tabs", "Tabs"),
    ("text", "Text"),
    ("textarea", "Textarea"),
    ("toggle", "Toggle"),
    ("toggle-group", "Toggle Group"),
    ("tooltip", "Tooltip"),
    ("table", "Table"),
];

#[component]
fn section(
    title: impl Into<String>,
    description: impl Into<String>,
    body: Vec<Element>,
) -> Element {
    shadcn::card(vec![
        shadcn::card_header(title, description),
        shadcn::card_content(body),
    ])
}

fn empty_box(width: f32, height: f32) -> Element {
    arkit::row_component().width(width).height(height).into()
}

#[component]
fn nav_bar(title: impl Into<String>, back: bool) -> Element {
    let left = if back {
        shadcn::button("‹", shadcn::ButtonVariant::Ghost)
            .width(40.0)
            .height(40.0)
            .on_click(|| {
                let _ = back_route();
            })
            .into()
    } else {
        empty_box(40.0, 40.0)
    };

    arkit::row_component()
        .percent_width(1.0)
        .height(56.0)
        .background_color(shadcn::theme::color::BACKGROUND)
        .style(ArkUINodeAttributeType::Padding, vec![8.0, 12.0, 8.0, 12.0])
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![0.0, 0.0, 1.0, 0.0],
        )
        .style(
            ArkUINodeAttributeType::BorderColor,
            vec![shadcn::theme::color::BORDER],
        )
        .children(vec![
            left,
            arkit::row_component()
                .percent_width(1.0)
                .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
                .children(vec![shadcn::text_variant(title, false)])
                .into(),
            empty_box(40.0, 40.0),
        ])
        .into()
}

fn component_list_cell(path: String, title: String, first: bool, last: bool) -> Element {
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
        .height(48.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
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
            shadcn::text_variant(title, false),
            shadcn::text_variant("›", true),
        ])
        .into()
}

#[component]
fn catalog_home(lifecycle_text: String, search: Signal<String>) -> Element {
    let keyword = search.get().to_lowercase();
    let filtered = SHOWCASE_COMPONENTS
        .iter()
        .filter(|(_, name)| keyword.is_empty() || name.to_lowercase().contains(&keyword))
        .cloned()
        .collect::<Vec<_>>();

    let list = if filtered.is_empty() {
        vec![shadcn::card(vec![
            shadcn::card_title("No component found"),
            shadcn::card_description("Try a different keyword"),
        ])]
    } else {
        filtered
            .iter()
            .enumerate()
            .map(|(index, (slug, title))| {
                component_list_cell(
                    format!("/components/{slug}"),
                    String::from(*title),
                    index == 0,
                    index + 1 == filtered.len(),
                )
            })
            .collect::<Vec<_>>()
    };

    arkit::column(vec![
        nav_bar("Showcase", false),
        arkit::scroll_component()
            .percent_width(1.0)
            .percent_height(1.0)
            .background_color(shadcn::theme::color::SURFACE)
            .children(vec![arkit::column_component()
                .percent_width(1.0)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![12.0, 16.0, 16.0, 16.0],
                )
                .children(vec![
                    shadcn::input("Components")
                        .bind(search)
                        .percent_width(1.0)
                        .into(),
                    arkit::row_component()
                        .style(ArkUINodeAttributeType::Margin, vec![12.0, 0.0, 0.0, 0.0])
                        .children(vec![shadcn::badge(lifecycle_text)])
                        .into(),
                    arkit::column_component()
                        .percent_width(1.0)
                        .style(ArkUINodeAttributeType::Margin, vec![12.0, 0.0, 0.0, 0.0])
                        .children(list)
                        .into(),
                ])
                .into()])
            .into(),
    ])
}

#[component]
fn button_demo(toggle_state: Signal<bool>) -> Element {
    let flip = toggle_state.clone();
    section(
        "按钮类型",
        "主按钮、次按钮、边框按钮、危险按钮、幽灵按钮",
        vec![
            arkit::row(vec![
                shadcn::button("主要按钮", shadcn::ButtonVariant::Default).into(),
                shadcn::button("次要按钮", shadcn::ButtonVariant::Secondary).into(),
            ]),
            arkit::row(vec![
                shadcn::button("边框按钮", shadcn::ButtonVariant::Outline).into(),
                shadcn::button("危险按钮", shadcn::ButtonVariant::Destructive).into(),
                shadcn::button("幽灵按钮", shadcn::ButtonVariant::Ghost).into(),
            ]),
            shadcn::button(
                if toggle_state.get() {
                    "已启用"
                } else {
                    "未启用"
                },
                if toggle_state.get() {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Default
                },
            )
            .on_click(move || flip.update(|value| *value = !*value))
            .into(),
        ],
    )
}

#[component]
fn input_demo(
    query: Signal<String>,
    choice: Signal<String>,
    toggle_state: Signal<bool>,
) -> Element {
    section(
        "表单组件",
        "输入、选择、开关、单选组合",
        vec![
            shadcn::form_item(
                "关键词",
                shadcn::input("请输入")
                    .bind(query.clone())
                    .percent_width(1.0)
                    .into(),
            ),
            shadcn::form_item(
                "文本域",
                shadcn::textarea("请输入详细描述").percent_width(1.0).into(),
            ),
            shadcn::form_item(
                "选择器",
                shadcn::select(
                    vec![
                        String::from("Option A"),
                        String::from("Option B"),
                        String::from("Option C"),
                    ],
                    choice.clone(),
                ),
            ),
            shadcn::radio_group(
                vec![
                    String::from("Radio A"),
                    String::from("Radio B"),
                    String::from("Radio C"),
                ],
                choice,
            ),
            shadcn::checkbox("同意条款", toggle_state.clone()),
            shadcn::switch(toggle_state).into(),
        ],
    )
}

#[component]
fn feedback_demo(toggle_state: Signal<bool>, page: Signal<i32>) -> Element {
    let progress = ((page.get().max(1) as f32) * 10.0).min(100.0);
    section(
        "反馈展示",
        "Alert、进度、Toast、Skeleton",
        vec![
            shadcn::alert("Info", "这是信息提示"),
            shadcn::alert_destructive("Warning", "这是危险提示"),
            shadcn::progress(progress, 100.0).into(),
            shadcn::toast(format!(
                "当前模式: {}",
                if toggle_state.get() {
                    "开启"
                } else {
                    "关闭"
                }
            )),
            shadcn::skeleton(220.0, 12.0),
            shadcn::skeleton(140.0, 12.0),
        ],
    )
}

#[component]
fn navigation_demo(active_tab: Signal<usize>, page: Signal<i32>) -> Element {
    section(
        "导航组件",
        "面包屑、菜单、标签页、分页",
        vec![
            shadcn::breadcrumb(vec![
                String::from("Home"),
                String::from("Components"),
                String::from("Navigation"),
            ]),
            shadcn::menubar(vec![
                shadcn::menubar_item("Overview"),
                shadcn::menubar_item("Mobile"),
                shadcn::menubar_item("About"),
            ]),
            shadcn::tabs(
                vec![
                    String::from("Tab 1"),
                    String::from("Tab 2"),
                    String::from("Tab 3"),
                ],
                active_tab,
                vec![
                    shadcn::card(vec![
                        shadcn::card_title("Tab 1"),
                        shadcn::card_description("第一个标签内容"),
                    ]),
                    shadcn::card(vec![
                        shadcn::card_title("Tab 2"),
                        shadcn::card_description("第二个标签内容"),
                    ]),
                    shadcn::card(vec![
                        shadcn::card_title("Tab 3"),
                        shadcn::card_description("第三个标签内容"),
                    ]),
                ],
            ),
            shadcn::pagination(page, 10),
        ],
    )
}

#[component]
fn data_demo() -> Element {
    section(
        "数据展示",
        "表格、图表、徽章、头像",
        vec![
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
            shadcn::chart(vec![20.0, 48.0, 66.0, 32.0, 88.0]),
            arkit::row(vec![
                shadcn::badge("new"),
                shadcn::badge("mobile"),
                shadcn::badge("arkit"),
            ]),
            arkit::row(vec![
                shadcn::avatar(None, "A"),
                shadcn::avatar(None, "B"),
                shadcn::avatar(None, "C"),
            ]),
        ],
    )
}

#[component]
fn layout_demo() -> Element {
    section(
        "布局与样式",
        "卡片、分隔、骨架与组合布局",
        vec![
            shadcn::separator(),
            shadcn::card(vec![
                shadcn::card_title("Card A"),
                shadcn::card_description("主信息区"),
                shadcn::text_variant("用于展示摘要和操作入口", true),
            ]),
            shadcn::card(vec![
                shadcn::card_title("Card B"),
                shadcn::card_description("副信息区"),
                arkit::row(vec![
                    shadcn::skeleton(90.0, 10.0),
                    shadcn::skeleton(120.0, 10.0),
                ]),
            ]),
            shadcn::resizable(
                shadcn::card(vec![
                    shadcn::card_title("Left"),
                    shadcn::card_description("内容区 1"),
                ]),
                shadcn::card(vec![
                    shadcn::card_title("Right"),
                    shadcn::card_description("内容区 2"),
                ]),
            ),
        ],
    )
}

fn route_to_demo(name: &str) -> (&str, &str) {
    match name {
        "button" | "toggle" | "toggle-group" => ("button", "Button"),
        "input" | "select" | "switch" | "radio-group" | "checkbox" | "label" | "textarea" => {
            ("input", "Form")
        }
        "tabs" | "menubar" => ("navigation", "Navigation"),
        "alert" | "alert-dialog" | "dialog" | "popover" | "hover-card" | "tooltip"
        | "context-menu" | "dropdown-menu" => ("feedback", "Feedback"),
        "table" | "avatar" | "badge" | "progress" | "text" => ("data", "Data"),
        "accordion" | "collapsible" | "aspect-ratio" | "card" | "separator" | "skeleton" => {
            ("layout", "Layout")
        }
        _ => ("unknown", "Unknown"),
    }
}

#[component]
fn component_page(
    name: String,
    active_tab: Signal<usize>,
    page: Signal<i32>,
    choice: Signal<String>,
    query: Signal<String>,
    toggle_state: Signal<bool>,
) -> Element {
    let (kind, title) = route_to_demo(&name);
    let body = match kind {
        "button" => button_demo(toggle_state),
        "layout" => layout_demo(),
        "data" => data_demo(),
        "feedback" => feedback_demo(toggle_state, page),
        "input" => input_demo(query, choice, toggle_state),
        "navigation" => navigation_demo(active_tab, page),
        _ => shadcn::card(vec![
            shadcn::card_title("未找到对应组件"),
            shadcn::card_description("请返回列表重新选择"),
        ]),
    };

    arkit::column(vec![
        nav_bar(title.to_string(), true),
        arkit::scroll_component()
            .percent_width(1.0)
            .percent_height(1.0)
            .background_color(shadcn::theme::color::SURFACE)
            .children(vec![arkit::column_component()
                .percent_width(1.0)
                .children(vec![
                    shadcn::card(vec![
                        shadcn::card_title(format!("{title} 组件")),
                        shadcn::card_description(format!("路由: /components/{name}")),
                    ]),
                    body,
                ])
                .into()])
            .into(),
    ])
}

#[entry]
fn app() -> Element {
    use_component_lifecycle(
        || {
            let _ = register_route("/");
            let _ = register_named_route("components", "/components/:name");
            let _ = reset_route("/");
        },
        || {},
    );

    let route = use_route();
    let active_tab = use_signal(|| 0usize);
    let page = use_signal(|| 1_i32);
    let choice = use_signal(|| String::from("Option A"));
    let query = use_signal(String::new);
    let toggle_state = use_signal(|| false);
    let lifecycle = use_signal(|| String::from("lifecycle: waiting"));
    let catalog_search = use_signal(String::new);

    let lifecycle_state = lifecycle.clone();
    use_lifecycle(move |event| {
        if matches!(
            event,
            LifecycleEvent::KeyboardEvent(_)
                | LifecycleEvent::Input(_)
                | LifecycleEvent::UserEvent
        ) {
            return;
        }
        lifecycle_state.set(format!("lifecycle: {}", event.as_str()));
    });

    let screen = if route.path() == "/" {
        catalog_home(lifecycle.get(), catalog_search)
    } else if let Some(name) = route.param("name") {
        component_page(
            name.to_string(),
            active_tab,
            page,
            choice,
            query,
            toggle_state,
        )
    } else {
        arkit::column(vec![
            nav_bar("Not Found", true),
            shadcn::card(vec![
                shadcn::card_title("Route Not Found"),
                shadcn::card_description("无法解析当前路由"),
                shadcn::button("返回首页", shadcn::ButtonVariant::Default)
                    .on_click(|| {
                        let _ = reset_route("/");
                    })
                    .into(),
            ]),
        ])
    };

    arkit::column(vec![screen])
}
