//! Device-side accessibility acceptance page.
//!
//! Stable `A11Y` labels are intentional: QEMU smoke tests inspect the ArkUI
//! accessibility tree and invoke the same actions a screen reader uses.

use arkit::prelude::*;
use arkit::shadcn::components::{Button, Checkbox, Progress, Slider, Switch};

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
    let progress = ((actions() % 5) * 25) as f32;

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
                                "A11Y clicks {} accessibility events {}",
                                actions(),
                                accessibility_events(),
                            ),
                            font_size: 16.0,
                            font_weight: 600,
                            font_color: FOREGROUND,
                            "A11Y clicks={actions} events={accessibility_events}"
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
                    }
                }
            }
        }
    }
}
