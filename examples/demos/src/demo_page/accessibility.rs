//! Device-side accessibility acceptance page.
//!
//! Stable `A11Y` labels are intentional: QEMU smoke tests inspect the ArkUI
//! accessibility tree and invoke the same actions a screen reader uses.

use arkit::barcode::{Barcode, BarcodeFormat};
use arkit::canvas::{Canvas, CanvasRenderer};
use arkit::echarts::{ChartOption, ECharts};
use arkit::prelude::*;
use arkit::shadcn::components::{
    Badge, Button, Card, CardHeader, Checkbox, Code, ContextMenu, Field, FieldError, FieldLabel,
    IndexBar, Input, MenuEntry, Progress, Slider, Switch, Timeline, TimelineItem,
};

const BACKGROUND: &str = "#FFF8FAFC";
const CARD: &str = "#FFFFFFFF";
const BORDER: &str = "#FFE2E8F0";
const FOREGROUND: &str = "#FF0F172A";
const MUTED: &str = "#FF475569";

#[component]
pub(super) fn AccessibilityPage() -> Element {
    let mut actions = use_signal(|| 0_u32);
    let mut accessibility_events = use_signal(|| 0_u32);
    let mut checked = use_signal(|| false);
    let mut enabled = use_signal(|| true);
    let mut menu_actions = use_signal(|| 0_u32);
    let mut selected_index = use_signal(|| "A".to_string());
    let canvas_renderer = use_hook(|| CanvasRenderer::new(|_| {}));
    let progress = ((actions() % 5) * 25) as f32;
    let context_items = vec![MenuEntry::action("A11Y menu action")
        .on_select(EventHandler::new(move |_| menu_actions += 1))];

    rsx! {
        column {
            width: "100%",
            height: "100%",
            background_color: BACKGROUND,
            accessibility_role: "group",
            accessibility_text: "Accessibility acceptance",

            scroll {
                width: "100%",
                height: "100%",
                scroll_bar: "auto",
                column {
                    width: "100%",
                    padding: 20.0,

                    text {
                        accessibility_role: "heading",
                        accessibility_text: "Accessibility acceptance",
                        font_size: 24.0,
                        font_weight: 700,
                        font_color: FOREGROUND,
                        "Accessibility acceptance"
                    }
                    text {
                        margin_top: 4.0,
                        accessibility_text: "Base nodes, custom controls, and shadcn components",
                        font_size: 13.0,
                        font_color: MUTED,
                        "Base nodes, custom controls, and shadcn components"
                    }

                    column {
                        width: "100%",
                        margin_top: 16.0,
                        padding: 16.0,
                        background_color: CARD,
                        border_width: 1.0,
                        border_color: BORDER,
                        border_radius: 12.0,

                        text {
                            accessibility_text: format!(
                                "A11Y clicks {} accessibility events {} menu actions {} index {}",
                                actions(),
                                accessibility_events(),
                                menu_actions(),
                                selected_index(),
                            ),
                            font_size: 16.0,
                            font_weight: 600,
                            font_color: FOREGROUND,
                            "A11Y clicks={actions} events={accessibility_events} menu={menu_actions} index={selected_index}"
                        }

                        button {
                            width: "100%",
                            height: 44.0,
                            margin_top: 12.0,
                            accessibility_text: "A11Y native button",
                            accessibility_description: "Increments the action counter",
                            onclick: move |_| actions += 1,
                            "Native button"
                        }

                        row {
                            width: "100%",
                            height: 44.0,
                            margin_top: 10.0,
                            align_items: "center",
                            justify_content: "center",
                            background_color: "#FFE0F2FE",
                            border_radius: 8.0,
                            accessibility_role: "button",
                            accessibility_text: "A11Y custom row button",
                            accessibility_description: "Custom row exposed as a button",
                            accessibility_group: true,
                            accessibility_actions: "click",
                            focusable: true,
                            focus_on_touch: true,
                            onclick: move |_| actions += 1,
                            onaccessibilityaction: move |_| accessibility_events += 1,
                            text {
                                accessibility_mode: "disabled",
                                font_size: 14.0,
                                font_weight: 600,
                                font_color: "#FF075985",
                                "Custom row button"
                            }
                        }

                        Button {
                            width: "100%",
                            accessibility_label: "A11Y shadcn button",
                            accessibility_description: "Increments the action counter",
                            onclick: move |_| actions += 1,
                            "shadcn Button"
                        }

                        row { height: 10.0 }
                        ContextMenu {
                            items: context_items,
                            default_open: false,
                            accessibility_label: "A11Y context menu trigger",
                            row {
                                width: "100%",
                                height: 38.0,
                                align_items: "center",
                                justify_content: "center",
                                background_color: "#FFF1F5F9",
                                border_radius: 8.0,
                                text {
                                    accessibility_mode: "disabled",
                                    font_size: 14.0,
                                    font_color: FOREGROUND,
                                    "Long press menu"
                                }
                            }
                        }

                        row { height: 12.0 }
                        Checkbox {
                            label: Some("A11Y shadcn checkbox".to_string()),
                            checked: Some(checked()),
                            on_change: Some(EventHandler::new(move |value| checked.set(value))),
                        }
                        row { height: 12.0 }
                        row {
                            width: "100%",
                            align_items: "center",
                            Switch {
                                accessibility_label: "A11Y shadcn switch",
                                checked: Some(enabled()),
                                on_change: Some(EventHandler::new(move |value| enabled.set(value))),
                            }
                            text {
                                margin_left: 10.0,
                                font_size: 14.0,
                                font_color: FOREGROUND,
                                "Feature enabled"
                            }
                        }
                        row { height: 16.0 }
                        Progress {
                            accessibility_label: "A11Y progress",
                            value: progress,
                            total: Some(100.0),
                        }
                        row { height: 16.0 }
                        Slider {
                            accessibility_label: "A11Y slider",
                            value: 40.0,
                            min: Some(0.0),
                            max: Some(100.0),
                        }

                        row { height: 16.0 }
                        Field {
                            invalid: true,
                            accessibility_label: "A11Y account field",
                            FieldLabel {
                                content: "A11Y account name".to_string(),
                                required: true,
                                invalid: true,
                            }
                            Input {
                                accessibility_label: "A11Y account input",
                                value: Some(String::new()),
                                placeholder: Some("Account name".to_string()),
                                width: "100%",
                                required: true,
                                invalid: true,
                            }
                            FieldError {
                                message: Some("A11Y account error".to_string()),
                            }
                        }

                        Badge {
                            content: "A11Y verified badge".to_string(),
                        }
                        row { height: 12.0 }
                        Card {
                            accessibility_label: "A11Y profile card",
                            accessibility_description: "Card descendants remain reachable",
                            CardHeader {
                                title: "A11Y card title".to_string(),
                                description: "A11Y card description".to_string(),
                            }
                        }
                        row { height: 12.0 }
                        Code {
                            source: "let accessible = true;".to_string(),
                            language: Some("rust".to_string()),
                            accessibility_label: "A11Y code sample",
                        }
                        row { height: 12.0 }
                        Canvas {
                            draw: canvas_renderer,
                            width: "100%",
                            height: "48",
                            accessibility_label: "A11Y canvas surface",
                            accessibility_description: "A11Y canvas surface",
                        }
                        row { height: 12.0 }
                        Barcode {
                            contents: "A11Y".to_string(),
                            format: BarcodeFormat::QrCode,
                            size: 64.0,
                            accessibility_label: "A11Y barcode preview",
                            accessibility_description: "A11Y barcode preview",
                        }
                        row { height: 12.0 }
                        ECharts {
                            option: ChartOption::new(),
                            height: "64",
                            accessibility_label: "A11Y native chart",
                            accessibility_description: "A11Y native chart",
                        }
                        row { height: 12.0 }
                        scroll {
                            width: "100%",
                            height: 64.0,
                            scroll_bar: "auto",
                            accessibility_role: "scroll",
                            accessibility_text: "A11Y results region",
                            accessibility_description: "Scrollable component results",
                            column {
                                width: "100%",
                                padding: 8.0,
                                text { "A11Y scroll content" }
                            }
                        }
                        row { height: 12.0 }
                        row {
                            width: "100%",
                            height: 96.0,
                            justify_content: "end",
                            IndexBar {
                                indexes: vec!["A".to_string(), "B".to_string(), "C".to_string()],
                                active: Some(selected_index()),
                                empty: vec!["C".to_string()],
                                on_select: move |value| selected_index.set(value),
                            }
                        }
                        row { height: 12.0 }
                        Timeline {
                            value: Some(1),
                            accessibility_label: "A11Y release timeline",
                            TimelineItem {
                                step: 1,
                                title: Some("A11Y semantics complete".to_string()),
                                description: Some("Current milestone".to_string()),
                            }
                            TimelineItem {
                                step: 2,
                                last: true,
                                title: Some("A11Y QEMU verified".to_string()),
                                description: Some("Next milestone".to_string()),
                            }
                        }
                    }
                }
            }
        }
    }
}
