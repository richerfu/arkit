//! Select — trigger showing the selected value + dropdown of options.
//!
//! Ported from the legacy Elm builder `select.rs`. The trigger is mounted in
//! place, while the dropdown panel is published through the Dioxus overlay
//! root and positioned from the measured trigger frame. This preserves the old
//! ArkUI floating-panel semantics without exposing any legacy builder API to
//! callers.

use super::floating_layer::{
    trigger_width_vp, FloatingAlign, FloatingPanelPlacement, FloatingSide, FLOATING_CAPTURE_COLOR,
};
use crate::{i18n::use_component_i18n, theme::*};
use arkit_prelude::*;

const SELECT_PANEL_FALLBACK_WIDTH: f32 = 180.0;
const SELECT_PANEL_SIDE_OFFSET: f32 = spacing::XXS;
const SELECT_PANEL_HEADER_HEIGHT: f32 = 32.0;
const SELECT_PANEL_SCROLL_HEIGHT: f32 = 208.0;
const SELECT_OPTION_HEIGHT: f32 = 36.0;
const SELECT_TEXT_MAX_LINES: i32 = 1;
const SELECT_TEXT_OVERFLOW_ELLIPSIS: &str = "ellipsis";

#[component]
pub fn Select(
    options: Vec<String>,
    placeholder: Option<String>,
    label: Option<String>,
    selected: Option<String>,
    default_selected: String,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<EventHandler<bool>>,
    on_select: Option<EventHandler<String>>,
) -> Element {
    let theme = use_theme();
    let i18n = use_component_i18n();
    let viewport = arkit_hooks::use_overlay_viewport();
    let trigger_ref = arkit_hooks::use_native_element_ref();
    let trigger_frame = use_signal(arkit_hooks::LayoutFrame::default);
    arkit_hooks::use_layout_frame(trigger_ref.clone(), move |frame| {
        let mut trigger_frame = trigger_frame;
        trigger_frame.set(frame);
    });
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
        placeholder.unwrap_or_else(|| i18n.select_placeholder())
    };
    let label_color = if has_value {
        colors.foreground
    } else {
        colors.muted_foreground
    };
    let count = options.len();
    let has_panel_label = label.as_deref() != Some("");

    let frame = *trigger_frame.read();
    let panel_width = trigger_width_vp(frame, viewport, SELECT_PANEL_FALLBACK_WIDTH);
    let panel_height = select_panel_estimated_height(count, has_panel_label);
    let placement = FloatingPanelPlacement::resolve(
        frame,
        viewport,
        panel_width,
        panel_height,
        FloatingSide::Bottom,
        FloatingAlign::Start,
        SELECT_PANEL_SIDE_OFFSET,
    );
    let dismiss = EventHandler::new(move |_: ()| set_open.call(false));

    rsx! {
        row {
            native_ref: trigger_ref,
            width: "100%",
            onclick: move |_| set_open.call(!current_open),
            row {
                width: "100%",
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
                shadow: "sm",
                row {
                    layout_weight: 1.0,
                    clip: true,
                    text {
                        width: "100%",
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
        if current_open {
            arkit_hooks::Portal {
                layer: arkit_hooks::OverlayLayer::Floating,
                {select_overlay_content(SelectOverlayContent {
                    theme,
                    i18n,
                    panel_width,
                    placement,
                    options,
                    label,
                    selected: current_selected,
                    set_selected,
                    on_dismiss: dismiss,
                })}
            }
        }
    }
}

struct SelectOverlayContent {
    theme: Theme,
    i18n: crate::i18n::ComponentI18n,
    panel_width: f32,
    placement: FloatingPanelPlacement,
    options: Vec<String>,
    label: Option<String>,
    selected: String,
    set_selected: EventHandler<String>,
    on_dismiss: EventHandler<()>,
}

fn select_overlay_content(content: SelectOverlayContent) -> Element {
    let SelectOverlayContent {
        theme,
        i18n,
        panel_width,
        placement,
        options,
        label,
        selected,
        set_selected,
        on_dismiss,
    } = content;
    let label = label
        .or_else(|| Some(i18n.select_label()))
        .filter(|label| !label.is_empty());
    let colors = theme.colors;
    let scroll_list = options.len() > 8;
    let top = placement.y.max(0.0);
    let left = placement.x.max(0.0);

    rsx! {
        // Full-screen hit plane; panel uses absolute position so left/top match
        // trigger coordinates in overlay-local vp space (no padding/margin skew).
        stack {
            width: "100%",
            height: "100%",
            background_color: FLOATING_CAPTURE_COLOR,
            hit_test_behavior: "default",
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
                shadow: "sm",
                padding_top: spacing::XXS,
                padding_right: spacing::XXS,
                padding_bottom: spacing::XXS,
                padding_left: spacing::XXS,
                if let Some(label) = label {
                    row {
                        width: "100%",
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
                            {label}
                        }
                    }
                }
                if scroll_list {
                    scroll {
                        width: "100%",
                        height: SELECT_PANEL_SCROLL_HEIGHT,
                        scroll_enabled: true,
                        column {
                            width: "100%",
                            for option in options.iter() {
                                {select_option_row(option, selected.as_str(), &theme, set_selected, on_dismiss)}
                            }
                        }
                    }
                } else {
                    column {
                        width: "100%",
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
            width: "100%",
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
                    width: "100%",
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

fn select_panel_estimated_height(option_count: usize, has_label: bool) -> f32 {
    (spacing::XXS * 2.0)
        + if has_label {
            SELECT_PANEL_HEADER_HEIGHT
        } else {
            0.0
        }
        + if option_count > 8 {
            SELECT_PANEL_SCROLL_HEIGHT
        } else {
            (option_count as f32) * SELECT_OPTION_HEIGHT
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_height_only_reserves_header_space_for_external_label() {
        let without_label = select_panel_estimated_height(3, false);
        let with_label = select_panel_estimated_height(3, true);

        assert_eq!(with_label - without_label, SELECT_PANEL_HEADER_HEIGHT);
    }
}
