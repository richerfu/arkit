//! Select — trigger showing the selected value + dropdown of options.
//!
//! Ported from the legacy Elm builder `select.rs`. The trigger is mounted in
//! place, while the dropdown panel is published through the Dioxus overlay
//! root and positioned from the measured trigger frame. This preserves the old
//! ArkUI floating-panel semantics without exposing any legacy builder API to
//! callers.

use super::floating_layer::{
    trigger_width_vp, FloatingAlign, FloatingPanelPlacement, FloatingSide, FLOATING_CAPTURE_COLOR,
    HIT_TEST_DEFAULT, SHADOW_SM,
};
use crate::theme::*;
use arkit_prelude::*;

const SELECT_PANEL_FALLBACK_WIDTH: f32 = 180.0;
const SELECT_PANEL_SIDE_OFFSET: f32 = spacing::XXS;
const SELECT_PANEL_HEADER_HEIGHT: f32 = 32.0;
const SELECT_PANEL_SCROLL_HEIGHT: f32 = 208.0;
const SELECT_OPTION_HEIGHT: f32 = 36.0;
const SELECT_TEXT_MAX_LINES: i32 = 1;
const SELECT_TEXT_OVERFLOW_ELLIPSIS: i32 = 2;

#[component]
pub fn Select(
    options: Vec<String>,
    selected: Option<String>,
    default_selected: String,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<EventHandler<bool>>,
    on_select: Option<EventHandler<String>>,
) -> Element {
    let theme = use_theme();
    let overlay = arkit_hooks::use_overlay();
    let mut trigger_frame = use_signal(arkit_hooks::LayoutFrame::default);
    // Measure only the painted trigger root via `onarea` (full control width).
    // Do not also attach `use_layout_frame` here — a second observer can race
    // and overwrite with a different host frame.
    let mut internal_open = use_signal(|| default_open);
    let mut internal_selected = use_signal(|| default_selected.clone());
    let open_controlled = open.is_some();
    let selected_controlled = selected.is_some();
    let current_open = open.unwrap_or_else(|| *internal_open.read());
    let current_selected = selected
        .clone()
        .unwrap_or_else(|| (*internal_selected.read()).clone());

    let set_open = EventHandler::new(move |value: bool| {
        if !open_controlled {
            internal_open.set(value);
        }
        if let Some(handler) = on_open_change {
            handler.call(value);
        }
    });
    let set_selected = EventHandler::new(move |value: String| {
        if !selected_controlled {
            internal_selected.set(value.clone());
        }
        if let Some(handler) = on_select {
            handler.call(value);
        }
    });

    let colors = theme.colors;
    let md = theme.radii.md;
    let has_value = !current_selected.is_empty();
    let trigger_label = if has_value {
        current_selected.clone()
    } else {
        String::from("Select a fruit")
    };
    let label_color = if has_value {
        colors.foreground
    } else {
        colors.muted_foreground
    };
    let count = options.len();

    let toggle = move |_| {
        if current_open {
            set_open.call(false);
            overlay.dismiss();
            return;
        }

        set_open.call(true);

        let frame = *trigger_frame.read();
        let viewport = overlay.viewport();
        let panel_width = trigger_width_vp(frame, viewport, SELECT_PANEL_FALLBACK_WIDTH);
        let panel_height = select_panel_estimated_height(count);
        let placement = FloatingPanelPlacement::resolve(
            frame,
            viewport,
            panel_width,
            panel_height,
            FloatingSide::Bottom,
            FloatingAlign::Start,
            SELECT_PANEL_SIDE_OFFSET,
        );
        let dismiss_overlay = overlay.clone();
        let dismiss = EventHandler::new(move |_: ()| {
            set_open.call(false);
            dismiss_overlay.dismiss();
        });
        let overlay_options = options.clone();
        let overlay_selected = current_selected.clone();

        overlay.show_floating(move || {
            select_overlay_content(
                theme,
                panel_width,
                placement,
                overlay_options,
                overlay_selected,
                set_selected,
                dismiss,
            )
        });
    };

    rsx! {
        row {
            percent_width: 1.0,
            // Measure the full control (same width as the painted chrome).
            onarea: move |evt: dioxus_core::Event<dioxus_elements::event::AreaData>| {
                let frame = evt.data().frame;
                if frame.is_measured() {
                    trigger_frame.set(arkit_hooks::LayoutFrame {
                        x: frame.x,
                        y: frame.y,
                        width: frame.width,
                        height: frame.height,
                    });
                }
            },
            onclick: move |_| {
                toggle(());
            },
            row {
                percent_width: 1.0,
                height: 40.0,
                background_color: colors.background,
                padding_top: 8.0,
                padding_right: spacing::MD,
                padding_bottom: 8.0,
                padding_left: spacing::MD,
                align_items: "center",
                justify_content: "space_between",
                border_radius: md,
                border_width: 1.0,
                border_color: colors.border,
                shadow: 1i32,
                row {
                    layout_weight: 1.0,
                    clip: true,
                    text {
                        percent_width: 1.0,
                        font_size: typography::SM,
                        font_color: label_color,
                        line_height: 20.0,
                        max_lines: SELECT_TEXT_MAX_LINES,
                        text_overflow: SELECT_TEXT_OVERFLOW_ELLIPSIS,
                        {trigger_label}
                    }
                }
                {crate::icon::icon_placeholder("chevron-down", 16.0, colors.muted_foreground)}
            }
        }
    }
}

fn select_overlay_content(
    theme: Theme,
    panel_width: f32,
    placement: FloatingPanelPlacement,
    options: Vec<String>,
    selected: String,
    set_selected: EventHandler<String>,
    on_dismiss: EventHandler<()>,
) -> Element {
    let colors = theme.colors;
    let scroll_list = options.len() > 8;
    let top = placement.y.max(0.0);
    let left = placement.x.max(0.0);

    rsx! {
        // Full-screen hit plane; panel uses absolute position so left/top match
        // trigger coordinates in overlay-local vp space (no padding/margin skew).
        stack {
            percent_width: 1.0,
            percent_height: 1.0,
            background_color: FLOATING_CAPTURE_COLOR,
            hit_test_behavior: HIT_TEST_DEFAULT,
            onclick: move |_| on_dismiss.call(()),
            column {
                position: format!("{left},{top}"),
                width: panel_width,
                align_items: "start",
                onclick: move |evt| evt.stop_propagation(),
                background_color: colors.popover,
                border_radius: theme.radii.md,
                border_width: 1.0,
                border_color: colors.border,
                shadow: SHADOW_SM,
                padding_top: spacing::XXS,
                padding_right: spacing::XXS,
                padding_bottom: spacing::XXS,
                padding_left: spacing::XXS,
                row {
                    percent_width: 1.0,
                    padding_top: 8.0,
                    padding_right: spacing::SM,
                    padding_bottom: 8.0,
                    padding_left: spacing::SM,
                    text {
                        font_size: typography::XS,
                        font_color: colors.muted_foreground,
                        line_height: 16.0,
                        max_lines: SELECT_TEXT_MAX_LINES,
                        text_overflow: SELECT_TEXT_OVERFLOW_ELLIPSIS,
                        "Fruits"
                    }
                }
                if scroll_list {
                    scroll {
                        percent_width: 1.0,
                        height: SELECT_PANEL_SCROLL_HEIGHT,
                        scroll_enabled: true,
                        column {
                            percent_width: 1.0,
                            for option in options.iter() {
                                {select_option_row(option, selected.as_str(), &theme, set_selected, on_dismiss)}
                            }
                        }
                    }
                } else {
                    column {
                        percent_width: 1.0,
                        for option in options.iter() {
                            {select_option_row(option, selected.as_str(), &theme, set_selected, on_dismiss)}
                        }
                    }
                }
            }
        }
    }
}

fn select_option_row(
    option: &str,
    selected: &str,
    theme: &Theme,
    set_selected: EventHandler<String>,
    on_dismiss: EventHandler<()>,
) -> Element {
    let colors = &theme.colors;
    let active = selected == option;
    let opt = option.to_owned();
    let fg = if active {
        colors.accent_foreground
    } else {
        colors.foreground
    };

    rsx! {
        row {
            percent_width: 1.0,
            height: SELECT_OPTION_HEIGHT,
            align_items: "center",
            justify_content: "space_between",
            padding_top: 8.0,
            padding_right: spacing::SM,
            padding_bottom: 8.0,
            padding_left: spacing::SM,
            border_radius: theme.radii.sm,
            background_color: if active { colors.accent } else { 0x00000000 },
            onclick: move |_: dioxus_core::Event<_>| {
                set_selected.call(opt.clone());
                on_dismiss.call(());
            },
            row {
                layout_weight: 1.0,
                clip: true,
                text {
                    percent_width: 1.0,
                    font_size: typography::SM,
                    font_color: fg,
                    line_height: 20.0,
                    max_lines: SELECT_TEXT_MAX_LINES,
                    text_overflow: SELECT_TEXT_OVERFLOW_ELLIPSIS,
                    {option.to_owned()}
                }
            }
            if active {
                {crate::icon::icon_placeholder("check", 16.0, colors.muted_foreground)}
            } else {
                row { width: 16.0, height: 16.0 }
            }
        }
    }
}

fn select_panel_estimated_height(option_count: usize) -> f32 {
    (spacing::XXS * 2.0)
        + SELECT_PANEL_HEADER_HEIGHT
        + if option_count > 8 {
            SELECT_PANEL_SCROLL_HEIGHT
        } else {
            (option_count as f32) * SELECT_OPTION_HEIGHT
        }
}
