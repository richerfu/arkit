//! Home page — grouped demo list driven by [`crate::registry`].

use arkit::prelude::*;
use arkit::router::{use_navigator, RouteProvider, RouteTransition};
use arkit::shadcn::icon::icon_placeholder;

use crate::registry::{DemoSpec, DEMO_GROUPS};
use crate::Route;

/// Card colors for the home list, kept in sync with the shadcn light theme.
const ROW_BACKGROUND: &str = "#fff9f9fa";
const ROW_BORDER: &str = "#14000000";
const ROW_ICON: u32 = 0xcc18181b;
const TITLE_COLOR: &str = "#ff18181b";
const CAPTION_COLOR: &str = "#ff71717a";
const GROUP_COLOR: &str = "#ff94a3b8";

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();

    rsx! {
        RouteTransition::<Route> {
            RouteProvider {
                column {
                    width: "100%",
                    align_items: "start",
                    justify_content: "start",
                    padding_top: 36.0,
                    padding_right: 20.0,
                    padding_bottom: 56.0,
                    padding_left: 20.0,

                    text {
                        font_size: 34.0,
                        font_weight: 700,
                        font_color: TITLE_COLOR,
                        "Arkit Demos"
                    }
                    text {
                        margin_top: 6.0,
                        font_size: 14.0,
                        line_height: 20.0,
                        font_color: CAPTION_COLOR,
                        "14 个示例统一入口 — 点击进入对应页面"
                    }

                    for group in DEMO_GROUPS {
                        column {
                            width: "100%",
                            align_items: "start",
                            justify_content: "start",
                            margin_top: 30.0,
                            text {
                                font_size: 13.0,
                                font_weight: 600,
                                font_color: GROUP_COLOR,
                                "{group.title}"
                            }
                            column {
                                width: "100%",
                                align_items: "start",
                                justify_content: "start",
                                margin_top: 8.0,
                                for (index, spec) in group.demos.iter().enumerate() {
                                    DemoRow {
                                        spec: *spec,
                                        first: index == 0,
                                        last: index + 1 == group.demos.len(),
                                        on_select: move |slug| {
                                            navigator.push(Route::Demo { slug });
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DemoRow(
    spec: DemoSpec,
    first: bool,
    last: bool,
    on_select: EventHandler<String>,
) -> Element {
    let radius = 12.0;
    let top_radius = if first { radius } else { 0.0 };
    let bottom_radius = if last { radius } else { 0.0 };
    let radius_value = format!("{top_radius},{top_radius},{bottom_radius},{bottom_radius}");
    let bottom_border = if last { 1.0 } else { 0.0 };
    let border_width = format!("1,1,{bottom_border},1");

    rsx! {
        row {
            width: "100%",
            height: 68.0,
            align_items: "center",
            justify_content: "start",
            padding_right: 14.0,
            padding_left: 14.0,
            background_color: ROW_BACKGROUND,
            border_width: border_width,
            border_color: ROW_BORDER,
            border_style: "solid",
            border_radius: radius_value,
            clip: true,
            onclick: move |_| on_select.call(spec.slug.to_string()),
            column {
                width: "100%",
                layout_weight: 1.0,
                align_items: "start",
                justify_content: "center",
                text {
                    content: spec.name.to_string(),
                    font_size: 16.0,
                    font_weight: 500,
                    font_color: TITLE_COLOR,
                    line_height: 22.0,
                    max_lines: 1_i32,
                    text_overflow: "ellipsis",
                }
                text {
                    margin_top: 2.0,
                    content: spec.description.to_string(),
                    font_size: 12.0,
                    line_height: 16.0,
                    font_color: CAPTION_COLOR,
                    max_lines: 1_i32,
                    text_overflow: "ellipsis",
                }
            }
            {icon_placeholder("chevron-right", 20.0, ROW_ICON)}
        }
    }
}
