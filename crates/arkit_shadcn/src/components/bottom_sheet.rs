//! Bottom sheet — a full-width modal surface anchored to the viewport bottom.
//!
//! The sheet is mounted through the application overlay root, so it is not
//! clipped by the page or showcase canvas. Its native presentation mirrors the
//! React Native Reusables sheet: dimmed dismissible backdrop, rounded top
//! corners, drag indicator, 60vp header, safe-area-aware body padding, and a
//! pan-down handle that dismisses after crossing the drag threshold.

use std::cell::Cell;
use std::rc::Rc;

use super::floating_layer::SHADOW_SM;
use super::{ARKUI_BORDER_STYLE_SOLID, ARKUI_BUTTON_TYPE_NORMAL};
use crate::icon::icon_placeholder;
use crate::theme::*;
use arkit_prelude::*;

const BOTTOM_SHEET_HEADER_HEIGHT: f32 = 60.0;
const BOTTOM_SHEET_HANDLE_HEIGHT: f32 = 24.0;
const BOTTOM_SHEET_MIN_HEIGHT: f32 = 240.0;
const BOTTOM_SHEET_DRAG_DISMISS_THRESHOLD: f32 = 72.0;

fn bottom_sheet_backdrop(theme: Theme) -> u32 {
    match theme.mode {
        ThemeMode::Light => 0x8F000000,
        ThemeMode::Dark => 0x3D000000,
    }
}

fn display_vp_ratio() -> f32 {
    let ratio = ohos_display_binding::default_display_virtual_pixel_ratio();
    if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    }
}

fn use_bottom_sheet_overlay(
    open: bool,
    panel: Element,
    on_dismiss: EventHandler<()>,
    theme: Theme,
) {
    let overlay = arkit_hooks::use_overlay();
    let last_open = use_hook(|| Rc::new(Cell::new(None::<bool>)));
    let changed = last_open.get() != Some(open);
    last_open.set(Some(open));

    let spec = arkit_hooks::ModalOverlaySpec {
        open,
        presentation: arkit_hooks::ModalPresentation::BottomDrawer,
        dismiss_on_backdrop: true,
        backdrop_color: bottom_sheet_backdrop(theme),
        viewport_inset: 0.0,
    };

    let effect_overlay = overlay.clone();
    use_effect(use_reactive((&open,), move |(open,)| {
        if !changed {
            return;
        }

        if open {
            let panel = panel.clone();
            effect_overlay.show_modal_with_dismiss(
                spec,
                move || panel.clone(),
                move || on_dismiss.call(()),
            );
        } else {
            effect_overlay.dismiss();
        }
    }));

    let cleanup_overlay = overlay.clone();
    let cleanup_last_open = last_open.clone();
    use_drop(move || {
        if cleanup_last_open.get() == Some(true) {
            cleanup_overlay.dismiss();
        }
    });
}

/// A controlled or uncontrolled bottom sheet.
///
/// The trigger remains caller-owned, matching the other modal components in
/// this crate. Set `open` from the trigger and handle `on_close` for backdrop,
/// close-button, save-button, and drag dismissal paths.
#[component]
pub fn BottomSheet(
    title: String,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let theme = use_theme();
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(value) => value,
        None => *internal.read(),
    };
    let controlled = open.is_some();

    let close = EventHandler::new(move |_: ()| {
        if !controlled {
            internal.set(false);
        }
        if let Some(handler) = on_close {
            handler.call(());
        }
    });

    let panel = rsx! {
        BottomSheetPanel {
            title,
            on_close: close,
            {children}
        }
    };

    use_bottom_sheet_overlay(current, panel, close, theme);
    rsx! {}
}

#[derive(Clone, Props)]
struct BottomSheetPanelProps {
    title: String,
    on_close: EventHandler<()>,
    children: Element,
}

impl PartialEq for BottomSheetPanelProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[allow(non_snake_case)]
fn BottomSheetPanel(props: BottomSheetPanelProps) -> Element {
    let theme = use_theme();
    let safe_area = arkit_hooks::use_safe_area();
    let mut drag_start = use_signal(|| None::<f32>);
    let mut drag_offset = use_signal(|| 0.0_f32);
    let on_close = props.on_close;
    let top_radius = format!("{0},{0},0,0", theme.radii.xl);
    let body_bottom_padding = safe_area.bottom + spacing::LG;

    rsx! {
        column {
            percent_width: 1.0,
            constraint_size: format!("0,100000,{BOTTOM_SHEET_MIN_HEIGHT},100000"),
            border_radius: top_radius,
            border_width: "1,1,0,1",
            border_color: theme.colors.border,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            background_color: theme.colors.card,
            shadow: SHADOW_SM,
            clip: true,
            on_touch: move |evt| {
                let Some(pointer) = evt.data().pointer else {
                    return;
                };
                let pointer_y = if pointer.has_window_position() {
                    pointer.window_y
                } else {
                    pointer.y
                };
                let ratio = display_vp_ratio();
                match pointer.action {
                    dioxus_elements::event::PointerAction::Down => {
                        drag_start.set(Some(pointer_y));
                        drag_offset.set(0.0);
                    }
                    dioxus_elements::event::PointerAction::Move => {
                        if let Some(start) = drag_start() {
                            let next_offset = ((pointer_y - start) / ratio).max(0.0);
                            if next_offset >= BOTTOM_SHEET_DRAG_DISMISS_THRESHOLD {
                                drag_start.set(None);
                                drag_offset.set(0.0);
                                on_close.call(());
                            } else {
                                drag_offset.set(next_offset);
                            }
                        }
                    }
                    dioxus_elements::event::PointerAction::Up => {
                        let should_dismiss = drag_offset() >= BOTTOM_SHEET_DRAG_DISMISS_THRESHOLD;
                        drag_start.set(None);
                        drag_offset.set(0.0);
                        if should_dismiss {
                            on_close.call(());
                        }
                    }
                    dioxus_elements::event::PointerAction::Cancel => {
                        drag_start.set(None);
                        drag_offset.set(0.0);
                    }
                    dioxus_elements::event::PointerAction::Unknown => {}
                }
            },
            row {
                percent_width: 1.0,
                height: BOTTOM_SHEET_HANDLE_HEIGHT,
                align_items: "center",
                justify_content: "center",
                row {
                    width: 32.0,
                    height: 4.0,
                    border_radius: theme.radii.full,
                    background_color: theme.colors.foreground,
                    opacity: 0.4_f32,
                }
            }
            row {
                percent_width: 1.0,
                height: BOTTOM_SHEET_HEADER_HEIGHT,
                align_items: "center",
                padding_left: spacing::LG,
                border_width: "0,0,1,0",
                border_color: theme.colors.border,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                row {
                    layout_weight: 1.0,
                    text {
                        percent_width: 1.0,
                        font_size: typography::XL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 25.0,
                        text_align: 1_i32,
                        "{props.title}"
                    }
                }
                button {
                    button_type: ARKUI_BUTTON_TYPE_NORMAL,
                    width: 48.0,
                    height: 48.0,
                    padding: 0.0,
                    background_color: 0x00000000,
                    border_width: 0.0,
                    border_style: ARKUI_BORDER_STYLE_SOLID,
                    border_radius: theme.radii.md,
                    clip: true,
                    focusable: false,
                    focus_on_touch: false,
                    alignment: 4_i32,
                    onclick: move |_| on_close.call(()),
                    {icon_placeholder("x", 24.0, theme.colors.muted_foreground)}
                }
            }
            column {
                percent_width: 1.0,
                padding_top: spacing::XXL,
                padding_right: spacing::LG,
                padding_bottom: body_bottom_padding,
                padding_left: spacing::LG,
                {props.children}
            }
        }
    }
}

/// The 56vp, `text-xl` input used by the native RNR bottom-sheet forms.
#[component]
pub fn BottomSheetTextInput(
    placeholder: Option<String>,
    value: Option<String>,
    on_change: Option<EventHandler<String>>,
) -> Element {
    let theme = use_theme();

    rsx! {
        textinput {
            value: if let Some(value) = value { value },
            placeholder: if let Some(placeholder) = placeholder { placeholder },
            placeholder_color: theme.colors.muted_foreground,
            caret_color: theme.colors.primary,
            percent_width: 1.0,
            height: 56.0,
            padding_top: spacing::XXS,
            padding_right: spacing::MD,
            padding_bottom: spacing::XXS,
            padding_left: spacing::MD,
            border_width: 1.0,
            border_color: theme.colors.input,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_radius: theme.radii.md,
            background_color: theme.colors.background,
            font_size: typography::XL,
            font_color: theme.colors.foreground,
            line_height: 25.0,
            on_change: move |evt| {
                if let Some(handler) = on_change {
                    handler.call(evt.data().string_value.clone());
                }
            },
        }
    }
}
