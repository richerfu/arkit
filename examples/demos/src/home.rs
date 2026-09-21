//! Home page — grouped demo list driven by [`crate::registry`].

use arkit::prelude::*;
use arkit::router::{use_navigator, RouteProvider, RouteTransition};
use arkit::shadcn::icon::icon_placeholder;

use crate::registry::{DemoSpec, DEMO_GROUPS};
use crate::Route;

/// Card colors for the home list, kept in sync with the shadcn light theme.
const ROW_BACKGROUND: &str = "#fff9f9fa";
const ROW_HOVER_BACKGROUND: &str = "#fff4f4f5";
const ROW_BORDER: &str = "#14000000";
const ROW_ICON: u32 = 0xcc18181b;
const TITLE_COLOR: &str = "#ff18181b";
const CAPTION_COLOR: &str = "#ff71717a";
const GROUP_COLOR: &str = "#ff94a3b8";

#[component]
pub fn Home() -> Element {
    let adaptive = use_adaptive_layout();
    let page_padding = adaptive.select(16.0, 32.0);
    let max_width = adaptive.select(640.0, 1200.0);
    let columns = if adaptive.width_vp >= 1200.0 { 3 } else { 2 };
    let resolved_style = if adaptive.is_pc() { "PC" } else { "Phone" };
    let mode = match adaptive.mode {
        AdaptiveMode::Auto => "Auto",
        AdaptiveMode::Phone => "Phone",
        AdaptiveMode::Pc => "PC",
    };

    rsx! {
        RouteTransition::<Route> {
            RouteProvider {
                column {
                    width: "100%",
                    align_items: "center",
                    justify_content: "start",
                    padding_top: 20.0,
                    padding_right: page_padding,
                    padding_bottom: 32.0,
                    padding_left: page_padding,

                    column {
                        width: "100%",
                        max_width: max_width,
                        align_items: "start",
                        text {
                            font_size: 28.0,
                            font_weight: 700,
                            font_color: TITLE_COLOR,
                            "Arkit Demos"
                        }
                        text {
                            margin_top: 4.0,
                            font_size: 13.0,
                            line_height: 18.0,
                            font_color: CAPTION_COLOR,
                            "16 个示例统一入口 · {mode} → {resolved_style} · {adaptive.width_vp:.0}vp"
                        }

                        for group in DEMO_GROUPS {
                            DemoGroupSection {
                                title: group.title.to_string(),
                                demos: group.demos.to_vec(),
                                pc: adaptive.is_pc(),
                                columns,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DemoGroupSection(title: String, demos: Vec<DemoSpec>, pc: bool, columns: usize) -> Element {
    let navigator = use_navigator();

    rsx! {
        column {
            width: "100%",
            align_items: "start",
            justify_content: "start",
            margin_top: 20.0,
            text {
                font_size: 12.0,
                font_weight: 600,
                font_color: GROUP_COLOR,
                "{title}"
            }
            if pc {
                column {
                    width: "100%",
                    margin_top: 8.0,
                    for (row_index, row_specs) in demos.chunks(columns).enumerate() {
                        row {
                            width: "100%",
                            height: 64.0,
                            margin_top: if row_index == 0 { 0.0 } else { 12.0 },
                            for (column_index, spec) in row_specs.iter().enumerate() {
                                column {
                                    height: "100%",
                                    layout_weight: 1.0,
                                    margin_left: if column_index == 0 { 0.0 } else { 12.0 },
                                    DemoRow {
                                        spec: *spec,
                                        first: true,
                                        last: true,
                                        standalone: true,
                                        default_focus: row_index == 0 && column_index == 0,
                                        on_select: move |slug| {
                                            navigator.push(Route::Demo { slug });
                                        },
                                    }
                                }
                            }
                            for column_index in row_specs.len()..columns {
                                column {
                                    height: "100%",
                                    layout_weight: 1.0,
                                    margin_left: if column_index == 0 { 0.0 } else { 12.0 },
                                }
                            }
                        }
                    }
                }
            } else {
                column {
                    width: "100%",
                    align_items: "start",
                    justify_content: "start",
                    margin_top: 6.0,
                    for (index, spec) in demos.iter().enumerate() {
                        DemoRow {
                            spec: *spec,
                            first: index == 0,
                            last: index + 1 == demos.len(),
                            standalone: false,
                            default_focus: false,
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

#[component]
fn DemoRow(
    spec: DemoSpec,
    first: bool,
    last: bool,
    standalone: bool,
    default_focus: bool,
    on_select: EventHandler<String>,
) -> Element {
    let adaptive = use_adaptive_layout();
    let mut hovering = use_signal(|| false);
    let mut focused = use_signal(|| false);
    let radius = 12.0;
    let top_radius = if first || standalone { radius } else { 0.0 };
    let bottom_radius = if last || standalone { radius } else { 0.0 };
    let radius_value = format!("{top_radius},{top_radius},{bottom_radius},{bottom_radius}");
    let bottom_border = if last || standalone { 1.0 } else { 0.0 };
    let border_width = format!("1,1,{bottom_border},1");

    rsx! {
        button {
            button_type: "normal",
            width: "100%",
            accessibility_role: "button",
            accessibility_text: spec.name,
            accessibility_description: spec.description,
            accessibility_group: true,
            accessibility_actions: "click",
            focusable: true,
            focus_on_touch: true,
            height: if standalone { 64.0 } else { 56.0 },
            padding_right: 12.0,
            padding_left: 12.0,
            background_color: if adaptive.is_pc() && (hovering() || focused()) { ROW_HOVER_BACKGROUND } else { ROW_BACKGROUND },
            border_width: border_width,
            border_color: ROW_BORDER,
            border_style: "solid",
            border_radius: radius_value,
            clip: true,
            focusable: adaptive.is_pc(),
            focus_on_touch: adaptive.is_pc(),
            default_focus: adaptive.is_pc() && default_focus,
            onhover: move |event| {
                if adaptive.is_pc() {
                    hovering.set(event.data().is_hovering);
                }
            },
            onfocus: move |_| focused.set(true),
            onblur: move |_| focused.set(false),
            onclick: move |_| on_select.call(spec.slug.to_string()),
            row {
                width: "100%",
                align_items: "center",
                justify_content: "start",
                column {
                    width: "100%",
                    layout_weight: 1.0,
                    align_items: "start",
                    justify_content: "center",
                    text {
                        content: spec.name.to_string(),
                        font_size: 15.0,
                        font_weight: 500,
                        font_color: TITLE_COLOR,
                        line_height: 20.0,
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
                {icon_placeholder("chevron-right", 16.0, ROW_ICON)}
            }
        }
    }
}
